//! A rust driver for the [DigOutBox](https://digoutbox.rtfd.io/)
//!
//! This driver provides all functionalities of the DigOutBox.
//!
//! # Example
//!
//! TODO:

#![warn(missing_docs)]

use std::{
    fmt::Display,
    sync::{Arc, Mutex},
};

use instrumentrs::{InstrumentError, InstrumentInterface};

/// Enum representing the current interlock state of the device.
#[derive(Debug, PartialEq)]
pub enum InterlockStatus {
    /// Status that is returned when the box is ready for operation (interlock not triggered).
    Ready,
    /// Status that is returned when the box's interlock was triggered.
    Interlocked,
}

impl Display for InterlockStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InterlockStatus::Ready => write!(f, "Instrument is ready"),
            InterlockStatus::Interlocked => write!(
                f,
                "Instrument is interlocked and not ready to activate any channel."
            ),
        }
    }
}

impl From<&str> for InterlockStatus {
    fn from(value: &str) -> Self {
        match value {
            "0" => InterlockStatus::Ready,
            _ => InterlockStatus::Interlocked,
        }
    }
}

/// Enum representing the current software lockout state of the device.
#[derive(Debug, PartialEq)]
pub enum SoftwareControlStatus {
    /// Status when software can be used to operate the device
    Ready,
    /// Status when the software is currently locked out from operating the device.
    LockedOut,
}

impl Display for SoftwareControlStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SoftwareControlStatus::Ready => write!(f, "Software control is possible."),
            SoftwareControlStatus::LockedOut => {
                write!(f, "Software is locked out from controlling the instrument.")
            }
        }
    }
}

impl From<&str> for SoftwareControlStatus {
    fn from(value: &str) -> Self {
        match value {
            "0" => SoftwareControlStatus::Ready,
            _ => SoftwareControlStatus::LockedOut,
        }
    }
}

/// A rust driver for the DigOutBox.
///
/// To talk to the DigOutBox, you have to first define what interface you want to use. For example,
/// you can use a blocking serial interface using [`serialport`]. Assuming the DigOutBox is
/// available as `/dev/ttyACM0`, you could initialize this driver as following.
///
/// ```no_run
/// use std::time::Duration;
/// use instrumentrs::SerialInstrument;
/// use digoutbox::DigOutBox;
///
/// let spb = serialport::new("/dev/ttyACM0", 9600).timeout(Duration::from_secs(3));
/// let interface = SerialInstrument::try_new(spb).unwrap();
/// let mut inst = DigOutBox::new(interface);
///
/// println!("Instrument name: {}", inst.get_name().unwrap());
/// ```
///
/// This would print the name, harware, and software version of the instrument to stdout.
pub struct DigOutBox<T: InstrumentInterface> {
    interface: Arc<Mutex<T>>,
    num_channels: usize,
}

impl<T: InstrumentInterface> DigOutBox<T> {
    /// Create a new DigOutBox instance with the given instrument interface.
    pub fn new(interface: T) -> Self {
        DigOutBox {
            interface: Arc::new(Mutex::new(interface)),
            num_channels: 16, // Default for the standard DigOutBox
        }
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
        Ok(Channel::new(idx, Arc::clone(&self.interface)))
    }

    /// Turn all channels off.
    pub fn all_off(&mut self) -> Result<(), InstrumentError> {
        self.sendcmd("ALLOFF")?;
        Ok(())
    }

    /// Get the status of all channels as a vector of booleans.
    ///
    /// The vector will contain `true` for channels that are on and `false` for channels that are
    /// off. Channels are zero-indexed and returned in order.
    pub fn get_all_outputs(&mut self) -> Result<Vec<bool>, InstrumentError> {
        let resp = self.query("ALLDO?")?;
        let outputs: Vec<bool> = resp.split(',').map(|s| s.trim() == "1").collect();
        Ok(outputs)
    }

    /// Get the current interlock status of the instrument.
    pub fn get_interlock_status(&mut self) -> Result<InterlockStatus, InstrumentError> {
        let resp = self.query("INTERLOCKS?")?;
        Ok(InterlockStatus::from(resp.as_ref()))
    }

    /// Query the name, hard, and firmware version of the device as a string.
    pub fn get_name(&mut self) -> Result<String, InstrumentError> {
        Ok(self.query("*IDN?")?.trim().to_string())
    }

    /// Set the number of channels for the DigOutBox.
    pub fn set_num_channels(&mut self, num: usize) {
        self.num_channels = num;
    }

    /// Get the current software control status of the instrument.
    pub fn get_software_control_status(
        &mut self,
    ) -> Result<SoftwareControlStatus, InstrumentError> {
        let resp = self.query("SWL?")?;
        Ok(SoftwareControlStatus::from(resp.as_ref()))
    }

    /// Send a command to the instrument.
    fn sendcmd(&mut self, cmd: &str) -> Result<(), InstrumentError> {
        {
            self.interface
                .lock()
                .expect("Mutext should not be poisoned")
                .sendcmd(cmd)?;
        }
        Ok(())
    }

    /// Query the instrument with a command and return the response as a String.
    fn query(&mut self, cmd: &str) -> Result<String, InstrumentError> {
        self.interface
            .lock()
            .expect("Mutex should not be poisoned")
            .query(cmd)
    }
}

/// Channel structure representing a single channel of the DigOutBox.
///
/// All commands to the channel must be sent through this structure. However, the channel itself
/// can only be created through the `DigOutBox` struct. This is to ensure that the channel is
/// always initialized with a valid interface.
pub struct Channel<T: InstrumentInterface> {
    idx: usize,
    interface: Arc<Mutex<T>>,
}

impl<T: InstrumentInterface> Channel<T> {
    /// Get the output of this channel as a boolean.
    ///
    /// Returns `true` if the channel output is on, otherwise `false`.
    pub fn get_output(&mut self) -> Result<bool, InstrumentError> {
        let val = self.query("DO")?;
        Ok(val == "1")
    }

    /// Set the output of this channel to a value.
    ///
    /// # Arguments
    /// * `value` - The boolean value to set the output to (true for high, false for low).
    pub fn set_output(&mut self, value: bool) -> Result<(), InstrumentError> {
        let value_send = if value { "1" } else { "0" };
        self.sendcmd("DO", value_send)
    }

    /// Get a new channel for the given instrument interface.
    ///
    /// This function can only be called from inside of the `DigOutBox` struct.
    fn new(idx: usize, interface: Arc<Mutex<T>>) -> Self {
        Channel { idx, interface }
    }

    /// Send a command to this channel of the instrument.
    ///
    /// All channel commands require the following formatting: `{CMD}{IDX} {ARG}`, where {CMD} is
    /// the command, {IDX} the channel number, and {ARG} the argument to send to the channel.
    ///
    /// # Arguments:
    /// - `cmd`: Command to send to the channel
    /// - `value`: Argument to send along with this command.
    fn sendcmd(&mut self, cmd: &str, value: &str) -> Result<(), InstrumentError> {
        {
            self.interface
                .lock()
                .expect("Mutex should not be poisoned")
                .sendcmd(&format!("{cmd}{0} {value}", self.idx))?;
        }
        Ok(())
    }

    /// Send a query to this channel of the instrument.
    ///
    /// Only the command to query must be provided as the channel number and question mark are
    /// automatically appended.
    ///
    /// # Arguments:
    /// - `cmd`: Command to send to the channel
    fn query(&mut self, cmd: &str) -> Result<String, InstrumentError> {
        self.interface
            .lock()
            .expect("Mutex should not be poisoned")
            .query(&format!("{cmd}{0}?", self.idx))
    }
}

// Tests
#[cfg(test)]
mod tests {

    use std::vec;

    use super::*;
    use instrumentrs::LoopbackInterface;

    // Tests for the instrument itself.

    #[test]
    fn test_terminator() {
        let empty_vec: Vec<&str> = Vec::new();
        let loopback = LoopbackInterface::new(empty_vec, vec![]);
        let inst = DigOutBox::new(loopback);
        {
            inst.interface
                .lock()
                .expect("Mutex should not be poisoned")
                .test_terminator("\n");
        }
    }

    #[test]
    pub fn test_all_off() {
        let loopback = LoopbackInterface::new(vec!["ALLOFF"], vec![]);
        let mut inst = DigOutBox::new(loopback);

        inst.all_off().unwrap();

        {
            inst.interface
                .lock()
                .expect("Mutex should not be poisoned")
                .finalize();
        }
    }

    #[test]
    fn test_get_all_outputs() {
        let loopback =
            LoopbackInterface::new(vec!["ALLDO?"], vec!["1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0"]);
        let mut inst = DigOutBox::new(loopback);

        assert_eq!(
            inst.get_all_outputs().unwrap(),
            vec![
                true, false, true, false, true, false, true, false, true, false, true, false, true,
                false, true, false
            ]
        );

        {
            inst.interface
                .lock()
                .expect("Mutex should not be poisoned")
                .finalize();
        }
    }

    #[test]
    fn test_get_interlock_status() {
        let loopback = LoopbackInterface::new(vec!["INTERLOCKS?", "INTERLOCKS?"], vec!["0", "1"]);
        let mut inst = DigOutBox::new(loopback);

        let interlock_status = inst.get_interlock_status().unwrap();
        assert_eq!(interlock_status, InterlockStatus::Ready);
        assert!(format!("{interlock_status}").contains("is ready"));

        let interlock_status = inst.get_interlock_status().unwrap();
        assert_eq!(interlock_status, InterlockStatus::Interlocked);
        assert!(format!("{interlock_status}").contains("is interlocked and not ready"));

        {
            inst.interface
                .lock()
                .expect("Mutex should not be poisoned")
                .finalize();
        }
    }

    #[test]
    fn test_get_name() {
        let loopback = LoopbackInterface::new(vec!["*IDN?"], vec!["Inst Name"]);
        let mut inst = DigOutBox::new(loopback);

        assert_eq!(inst.get_name().unwrap(), "Inst Name");

        {
            inst.interface
                .lock()
                .expect("Mutex should not be poisoned")
                .finalize();
        }
    }

    #[test]
    fn test_get_software_control_status() {
        let loopback = LoopbackInterface::new(vec!["SWL?", "SWL?"], vec!["0", "1"]);
        let mut inst = DigOutBox::new(loopback);

        let scs = inst.get_software_control_status().unwrap();
        assert_eq!(scs, SoftwareControlStatus::Ready);
        assert!(format!("{scs}").contains("Software control is possible."));

        let scs = inst.get_software_control_status().unwrap();
        assert_eq!(scs, SoftwareControlStatus::LockedOut);
        assert!(format!("{scs}").contains("is locked out"));

        {
            inst.interface
                .lock()
                .expect("Mutex should not be poisoned")
                .finalize();
        }
    }

    // Tests for the channels
    #[test]
    fn test_get_channel() {
        let empty_vec: Vec<&str> = Vec::new();
        let loopback = LoopbackInterface::new(empty_vec, vec![]);
        let mut inst = DigOutBox::new(loopback);

        // Get a channel and check if it is created correctly
        let channel = inst.get_channel(0).unwrap();
        assert_eq!(channel.idx, 0);

        // Try to get a channel that is out of range
        match inst.get_channel(17) {
            Err(InstrumentError::ChannelIndexOutOfRange { idx, nof_channels }) => {
                assert_eq!(idx, 17);
                assert_eq!(nof_channels, 16);
            }
            _ => panic!("Expected ChannelIndexOutOfRange error"),
        }

        // Now set the box up so it has only 6 channels
        inst.set_num_channels(6);
        // Try to get a channel that is out of range
        assert!(inst.get_channel(6).is_err());
    }

    #[test]
    fn test_channel_output() {
        let loopback =
            LoopbackInterface::new(vec!["DO0 1", "DO0?", "DO1 0", "DO1?"], vec!["1", "0"]);
        let mut inst = DigOutBox::new(loopback);

        let mut ch0 = inst.get_channel(0).unwrap();
        ch0.set_output(true).unwrap();
        assert!(ch0.get_output().unwrap());

        let mut ch1 = inst.get_channel(1).unwrap();
        ch1.set_output(false).unwrap();
        assert!(!ch1.get_output().unwrap());

        {
            inst.interface
                .lock()
                .expect("Mutex should not be poisoned")
                .finalize();
        }
    }
}
