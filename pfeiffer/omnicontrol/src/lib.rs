//! A rust driver for pfeiffer_omnicontrol
//!
//! TODO: Short description of what the driver does
//!
//! # Example
//!
//! TODO: High-level example
//! ```no_run
//! ```

//TODO: Uncomment the following line to enable warnings and missing docs
//#![deny(warnings, missing_docs)]

use std::sync::{Arc, Mutex};

use instrumentrs::{InstrumentError, InstrumentInterface};

mod cmd_package;
mod lib_utils;
mod package_utils;
mod read_package;

pub use lib_utils::BaseAddress;
use measurements::Pressure;

use crate::{cmd_package::CommandPackage, package_utils::DataType, read_package::ReadPackage};

/// A rust driver for the Omnicontrol.
///
/// This driver provides functionality to control the Pfeiffer/Omnicontrol.
///
/// # Example
/// TODO
/// ```no_run
/// ```
pub struct Omnicontrol<T: InstrumentInterface> {
    interface: Arc<Mutex<T>>,
    num_channels: usize,
    base_address: BaseAddress,
    base_device: usize,
}

impl<T: InstrumentInterface> Omnicontrol<T> {
    /// Create a new Omnicontrol instance with the given instrument interface.
    ///
    /// # Arguments
    /// * `interface` - An instrument interface that implements the [`InstrumentInterface`] trait.
    pub fn new(interface: T, base_address: BaseAddress) -> Self {
        let mut intf = interface;
        intf.set_terminator("\r");
        let interface = Arc::new(Mutex::new(intf));

        Omnicontrol {
            interface,
            num_channels: 4,
            base_address,
            base_device: 1, // Base device is always 1 for Omnicontrol
        }
    }

    /// Get a new channel with a given index for the Channel.
    ///
    /// Please note that channels are zero indexed.
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
            self.base_address,
        ))
    }

    /// Set the number of channels for the Omnicontrol.
    pub fn set_num_channels(&mut self, num: usize) -> Result<(), InstrumentError> {
        if !(1..5).contains(&num) {
            let num: i64 = num.try_into().unwrap_or(i64::MAX);
            return Err(InstrumentError::IntValueOutOfRange {
                value: num,
                min: 1,
                max: 4,
            });
        }
        self.num_channels = num;
        Ok(())
    }

    /// Query the name of the instrument
    ///
    /// Returns the designation of the instrument as a String.
    pub fn get_name(&mut self) -> Result<String, InstrumentError> {
        let cmd = CommandPackage::get_read_pkg(self.base_address, self.base_device, 349);
        let res = self.query(&cmd)?;
        Ok(ReadPackage::try_new(&res)?.get_data_string())
    }

    /// Send a command for this instrument to an interface.
    fn sendcmd(&mut self, cmd: &str) -> Result<(), InstrumentError> {
        sendcmd(Arc::clone(&self.interface), cmd)
    }

    /// Query the instrument with a command and return the response as a String.
    fn query(&mut self, cmd: &str) -> Result<String, InstrumentError> {
        query(Arc::clone(&self.interface), cmd)
    }
}

impl<T: InstrumentInterface> Clone for Omnicontrol<T> {
    fn clone(&self) -> Self {
        Self {
            interface: self.interface.clone(),
            num_channels: self.num_channels,
            base_address: self.base_address,
            base_device: self.base_device,
        }
    }
}

/// Channel structure representing a single channel of the Omnicontrol.
///
/// **This structure can only be created through the [`Omnicontrol`] struct.**
///
/// Implementation of an individual channel and commands that go to it.
pub struct Channel<T: InstrumentInterface> {
    idx: usize,
    interface: Arc<Mutex<T>>,
    base_address: BaseAddress,
}

impl<T: InstrumentInterface> Channel<T> {
    /// Get a new channel for the given instrument interface.
    ///
    /// This function can only be called from inside of the [`Omnicontrol`] struct.
    fn new(idx: usize, interface: Arc<Mutex<T>>, base_address: BaseAddress) -> Self {
        Channel {
            idx,
            interface,
            base_address,
        }
    }

    /// Get pressure from this channel as a [`measurements::Pressure`] value.
    pub fn get_pressure(&mut self) -> Result<Pressure, InstrumentError> {
        let base_device = (self.idx + 1) * 10 + 2; // Pressure sensor device address
        let cmd = CommandPackage::get_read_pkg(self.base_address, base_device, 740);
        let res = self.query(&cmd)?;
        let data = ReadPackage::try_new(&res)?.get_data_string();
        let pressure = DataType::UExpoNew.parse_to_f64(&data)?;
        println!("Pressure read from channel {}: {} hPa", self.idx, pressure);
        Ok(Pressure::from_hectopascals(pressure))
    }

    /// Send a command for this instrument to an interface.
    fn sendcmd(&mut self, cmd: &str) -> Result<(), InstrumentError> {
        sendcmd(Arc::clone(&self.interface), cmd)
    }

    /// Query the instrument with a command and return the response as a String.
    fn query(&mut self, cmd: &str) -> Result<String, InstrumentError> {
        query(Arc::clone(&self.interface), cmd)
    }
}

impl<T: InstrumentInterface> Clone for Channel<T> {
    fn clone(&self) -> Self {
        Self {
            idx: self.idx,
            interface: self.interface.clone(),
            base_address: self.base_address,
        }
    }
}

/// Send a command for this instrument to the given interface.
fn sendcmd<T: InstrumentInterface>(intf: Arc<Mutex<T>>, cmd: &str) -> Result<(), InstrumentError> {
    let mut intf = intf.lock().expect("Mutex should not be poisoned");
    intf.sendcmd(cmd)
}

/// Query this instrument via the given interface.
fn query<T: InstrumentInterface>(
    intf: Arc<Mutex<T>>,
    cmd: &str,
) -> Result<String, InstrumentError> {
    {
        sendcmd(Arc::clone(&intf), cmd)?;
    }
    let mut intf = intf.lock().expect("Mutex should not be poisoned");
    intf.read_until_terminator()
}
