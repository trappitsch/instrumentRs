//! A rust driver for the Pfeiffer/Inficon TPG36x vacuum gauge controller.
//!
//! This driver is by default set up to work with the dual gauge model (TPG362), but can also be
//! set up to be used with the single gauge model (TPG361).
//!
//! # Example
//!
//! This example shows the usage via the TCP/IP interface. An example for the serial port is shown
//! in the [`Tpg36x`] documentation.
//!
//! ```no_run
//! use instrumentrs::TcpIpInterface;
//! use pfeiffer_tpg36x::{SensorStatus, Tpg36x};
//!
//! // IP address and port of the instrument. Adjust to your setup!
//! let addr = "192.168.1.10:8000";
//!
//! // Create a new TCP/IP instrument interface and use it to create a new Tpg36x instance.
//! let tcpip_inst = TcpIpInterface::simple(addr).unwrap();
//! let mut inst = Tpg36x::try_new(tcpip_inst).unwrap();
//!
//! // Check if the the first pressure sensor is on and if so, read the pressure and print it.
//! let mut sensor1 = inst.get_channel(0).unwrap();
//! let status = sensor1.get_status().unwrap();
//!
//! if status == SensorStatus::On {
//!     println!("Current pressure: {}", sensor1.get_pressure().unwrap());
//! }

#![deny(warnings, missing_docs)]

mod ethernet_conf;
mod status;
mod units;

pub use ethernet_conf::{DhcpConfig, EthernetConfig};
pub use status::SensorStatus;
pub use units::{PressureUnit, Tpg36xMeasurement};

use std::sync::{Arc, Mutex};

use instrumentrs::{InstrumentError, InstrumentInterface};

use status::PressMsrDatStat;

/// A rust driver for the TPG36x.
///
/// This driver provides functionality to control the Pfeiffer/Inficon TPG361 and TPG362 vacuum gauge
/// controllers.
///
/// # Example via serial port connection
/// ```no_run
/// use instrumentrs::SerialInterface;
/// use pfeiffer_tpg36x::Tpg36x;
///
/// let port = "/dev/ttyACM0";
/// let baud = 9600;
/// let inst_interface = SerialInterface::simple(port, baud).unwrap();
/// let mut inst = Tpg36x::try_new(inst_interface).unwrap();
///
/// println!("Instrument name: {}", inst.get_name().unwrap());
/// ```
///
/// This would print the type of unit, model number, serial number, firmware, and hardware version
/// of the vacuum gauge controller to `stdout`.
pub struct Tpg36x<T: InstrumentInterface> {
    interface: Arc<Mutex<T>>,
    unit: Arc<Mutex<PressureUnit>>,
    num_channels: usize,
}

impl<T: InstrumentInterface> Tpg36x<T> {
    /// Create a new TPG36x instance with the given instrument interface.
    ///
    /// This function can fail if the instrument is not answering, as the function queries the
    /// instrument upon initialization in order to set the correct pressure unit that is currently
    /// displayed.
    ///
    /// # Arguments
    /// - `interface`: An instrument interface that implements the [`InstrumentInterface`] trait.
    pub fn try_new(interface: T) -> Result<Self, InstrumentError> {
        let mut intf = interface;
        intf.set_terminator("\r\n");
        let interface = Arc::new(Mutex::new(intf));
        let mut instrument = Tpg36x {
            interface,
            unit: Arc::new(Mutex::new(PressureUnit::default())),
            num_channels: 2, // Default for the standard DigOutBox
        };
        instrument.update_unit()?;
        Ok(instrument)
    }

    /// Get a new channel with a given index for the Channel.
    ///
    /// Please note that channels are zero-indexed.
    pub fn get_channel(&mut self, idx: usize) -> Result<Channel<T>, InstrumentError> {
        if idx >= self.num_channels {
            return Err(InstrumentError::ChannelIndexOutOfRange {
                idx,
                nof_channels: self.num_channels,
            });
        }
        Ok(Channel::new(
            idx,
            Arc::clone(&self.interface),
            Arc::clone(&self.unit),
        ))
    }

    /// Get the ethernet configuration of the TPG36x.
    ///
    /// This returns the current ethernet configuration of the TPG36x as an [`EthernetConfig`]
    pub fn get_ethernet_config(&mut self) -> Result<EthernetConfig, InstrumentError> {
        let response = self.query("ETH")?;
        EthernetConfig::from_cmd_str(response.as_str())
            .map_err(|_| InstrumentError::ResponseParseError(response))
    }

    /// Set the ethernet configuration for the TPG36x.
    ///
    /// # Arguments
    /// - `ethernet_config`: An ethernet configuration.
    pub fn set_ethernet_config(
        &mut self,
        ethernet_config: EthernetConfig,
    ) -> Result<(), InstrumentError> {
        self.sendcmd(&ethernet_config.to_command_string())
    }

    /// Query the name, hard, and firmware version of the device as a string.
    ///
    /// This returns, separated by commas, the following information as a string:
    /// - Type of the unit, e.g. TPG362
    /// - Model No. of the unit, e.g. PTG28290
    /// - Serial No. of the unit, e.g. 44990000
    /// - Firmware version of the unit, e.g.. 010100
    /// - Hardware version of the unit, e.g. 010100
    pub fn get_name(&mut self) -> Result<String, InstrumentError> {
        Ok(self.query("AYT")?.trim().to_string())
    }

    /// Set the number of channels for the DigOutBox.
    pub fn set_num_channels(&mut self, num: usize) -> Result<(), InstrumentError> {
        if num >= 2 {
            let num: i64 = num.try_into().unwrap_or(i64::MAX);
            return Err(InstrumentError::IntValueOutOfRange {
                value: num,
                min: 0,
                max: 1,
            });
        }
        self.num_channels = num;
        Ok(())
    }

    /// Get the MAC address of the instrument.
    ///
    /// This returns a string that you can put into your own mac address converter if you like.
    /// However, as this is a niche feature and MAC address handling is not in `std`, we decided
    /// to return a String instead.
    pub fn get_mac_address(&mut self) -> Result<String, InstrumentError> {
        self.query("MAC")
    }

    /// Get the current unit from the instrument.
    ///
    /// This updates the internally kept unit and returns a copy of it.
    pub fn get_unit(&mut self) -> Result<PressureUnit, InstrumentError> {
        self.update_unit()?;
        let unit = self.unit.lock().expect("Mutex should not be poisoned");
        Ok(*unit)
    }

    /// Set the unit for the instrument.
    ///
    /// This sets a new unit for the instrument and, if successful, updates the internal unit
    /// representation to match the new unit.
    ///
    /// # Arguments
    /// - `unit`: The new unit to set for the instrument.
    pub fn set_unit(&mut self, unit: PressureUnit) -> Result<(), InstrumentError> {
        self.sendcmd(&format!("UNI,{}", unit.as_str()))?;
        {
            let mut current_unit = self.unit.lock().expect("Mutex should not be poisoned");
            *current_unit = unit;
        }
        Ok(())
    }

    /// Update the unit by querying the instrument for the current unit setting.
    pub fn update_unit(&mut self) -> Result<(), InstrumentError> {
        let response = self.query("UNI")?;
        {
            let mut unit = self.unit.lock().expect("Mutex should not be poisoned");
            *unit = PressureUnit::from_cmd_str(response.as_str())?;
        }
        Ok(())
    }

    /// Send a command to the instrument.
    fn sendcmd(&mut self, cmd: &str) -> Result<(), InstrumentError> {
        let mut intf = self.interface.lock().expect("Mutex should not be poisoned");
        intf.sendcmd(cmd)?;
        intf.check_acknowledgment("\u{6}") // check for "ACK"
    }

    fn query(&mut self, cmd: &str) -> Result<String, InstrumentError> {
        self.sendcmd(cmd)?;
        let mut intf = self.interface.lock().expect("Mutex should not be poisoned");
        intf.write("\u{5}")?; // send "ENQ"
        intf.read_until_terminator()
    }
}

/// Channel structure representing a single channel of the TPG36x.
///
/// **This structure can only be created through the [`Tpg36x`] struct.**
///
/// Implementation of an individual channel and commands that go to it.
pub struct Channel<T: InstrumentInterface> {
    idx: usize,
    interface: Arc<Mutex<T>>,
    unit: Arc<Mutex<PressureUnit>>,
}

impl<T: InstrumentInterface> Channel<T> {
    /// Get the pressure of this channel in the given unit.
    ///
    /// This will return a [`Tpg36xMeasurement`] struct containing the value either as a pressure or
    /// as a voltage, depending on the setup of the unit.
    ///
    /// **Note**: If the unit on the instrument was changed manually, this may not return the
    /// correct value! In this case, make sure that the `update_unit` function on the [`Tpg36x`]
    /// struct prior to calling this function!
    pub fn get_pressure(&mut self) -> Result<Tpg36xMeasurement, InstrumentError> {
        let resp = self.query(&format!("PR{}", self.idx + 1))?;
        println!("Response: {resp}");
        let parts = resp.split(',').collect::<Vec<&str>>();
        if parts.len() != 2 {
            return Err(InstrumentError::ResponseParseError(resp));
        }

        let status = PressMsrDatStat::from_cmd_str(parts[0])?;
        if status != PressMsrDatStat::Ok {
            return Err(InstrumentError::InstrumentStatus(format!("{status}")));
        }

        let val = parts[1]
            .parse::<f64>()
            .map_err(|_| InstrumentError::ResponseParseError(resp.to_string()))?;
        let ret_val = {
            let unit = self.unit.lock().expect("Mutex should not be poisoned");
            units::from_value_unit(val, &unit)
        };
        Ok(ret_val)
    }

    /// Get the status of the channel.
    ///
    /// This routine returns the status of the channel, i.e., whether the channel is on, off, or in
    /// a stat that cannot be changed.
    pub fn get_status(&mut self) -> Result<SensorStatus, InstrumentError> {
        let resp = self.query("SEN")?;
        let parts = split_check_resp(&resp, 2)?;
        // This should be infallible for two reasons:
        // - We check the length of the vector before in the `split_check_resp` function.
        // - If it's a one channel gauge, `self.idx = 1` cannot be accessed from the get go.
        // So if this panics, it is a bug in the code!
        SensorStatus::from_cmd_str(parts[self.idx])
    }

    /// Set the status of the channel.
    ///
    /// This routine sets the status of the channel, i.e., whether the channel should be on, off,
    /// or left unchanged.
    ///
    /// Note: The manual does not specify different commands for the one or two channel models,
    /// even though it does for other commands. We thus assume that sending two channels always is
    /// not a problem, as the second channel on the one channel model is simply ignored. This is an
    /// assumption, as we currently have no one channel model to test this with.
    pub fn set_status(&mut self, status: SensorStatus) -> Result<(), InstrumentError> {
        let mut to_send = [SensorStatus::NoChange, SensorStatus::NoChange];
        to_send[self.idx] = status; // infallible, `self.idx` can at most be 1
        self.sendcmd(&format!(
            "SEN,{},{}",
            to_send[0].to_cmd_str(),
            to_send[1].to_cmd_str()
        ))?;
        Ok(())
    }

    /// Get a new channel for the given instrument interface.
    ///
    /// This function can only be called from inside of the [`Tpg36x`] struct.
    fn new(idx: usize, interface: Arc<Mutex<T>>, unit: Arc<Mutex<PressureUnit>>) -> Self {
        Channel {
            idx,
            interface,
            unit,
        }
    }

    /// Send a command for this instrument to an interface.
    fn sendcmd(&mut self, cmd: &str) -> Result<(), InstrumentError> {
        let mut intf = self.interface.lock().expect("Mutex should not be poisoned");
        intf.sendcmd(cmd)?;
        intf.check_acknowledgment("\u{6}") // check for "ACK"
    }

    /// Query the instrument with a command and return the response as a String.
    fn query(&mut self, cmd: &str) -> Result<String, InstrumentError> {
        self.sendcmd(cmd)?;
        let mut intf = self.interface.lock().expect("Mutex should not be poisoned");
        intf.write("\u{5}")?; // send "ENQ"
        intf.read_until_terminator()
    }
}

/// Split a string slice into its parts by commas, check if of correct length, and return the parts
/// as a vector.
fn split_check_resp(resp: &str, exp_len: usize) -> Result<Vec<&str>, InstrumentError> {
    let parts = resp.split(',').collect::<Vec<&str>>();
    if parts.len() != exp_len {
        return Err(InstrumentError::ResponseParseError(resp.to_string()));
    }
    Ok(parts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    /// Ensure that the split really splits by commas and checks the length.
    #[rstest]
    fn test_split_check_resp() {
        let resp = "part1,part2,part3";
        let parts = split_check_resp(resp, 3).unwrap();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0], "part1");
        assert_eq!(parts[1], "part2");
        assert_eq!(parts[2], "part3");

        // Test with incorrect length
        assert!(split_check_resp(resp, 2).is_err());
        assert!(split_check_resp(resp, 4).is_err());
    }

    /// Ensure that any response without comma returns one part, which is the response itself.
    #[rstest]
    #[case("")]
    #[case("asdf")]
    fn test_split_check_resp_empty(#[case] resp: &str) {
        let parts = split_check_resp(resp, 1).unwrap();

        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0], resp);

        // Test with incorrect length
        assert!(split_check_resp(resp, 0).is_err());
        assert!(split_check_resp(resp, 2).is_err());
    }
}
