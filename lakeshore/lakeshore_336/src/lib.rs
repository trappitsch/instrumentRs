//! A rust driver for Lakeshore_336 temperature controller.
//!
//! This driver provides functionality for the Lakeshore336 temperature controller.
//! Currently, only reading the temperature from the four channels is implemented.
//!
//! # Example
//!
//! This example shows the usage via the serial interface.
//! ```no_run
//! use lakeshore_336::{Lakeshore336, SerialInterfaceLakeshore};
//!
//! // The port where the Lakeshore336 is connected to
//! let port = "/dev/ttyUSB0";
//!
//! // Get the serial interface for the Lakeshore336 and open it. This interface already sets the
//! // correct parity, stop bits, and data bits for communication with the Lakeshore336.
//! let serial_inst = SerialInterfaceLakeshore::simple(port).expect("Failed to open serial port");
//! let mut inst = Lakeshore336::try_new(serial_inst).unwrap();
//!
//! // Query the name of the instrument
//! println!("{}", inst.get_name().unwrap());
//!
//! // Print the temperature values of channels A and C
//! let mut cha = inst.get_channel(0).unwrap();
//! let mut chc = inst.get_channel(2).unwrap();
//! println!("Channel A temperature: {:?}", cha.get_temperature());
//! println!("Channel C temperature: {:?}", chc.get_temperature());
//! ```

#![deny(warnings, missing_docs)]

use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use instrumentrs::{Instrument, InstrumentError, InstrumentInterface, SerialInterface};

use measurements::Temperature;

use serialport::SerialPort;

/// A SerialInterface for the Lakeshore336.
///
/// Builds an InstrumentRs SerialInterface with the correct parity, stop bits, and data bits for
/// communication with the Lakeshore336.
#[derive(Debug)]
pub struct SerialInterfaceLakeshore {}

impl SerialInterfaceLakeshore {
    /// Try to create an Instrument interface with a simple serial port configuration.
    ///
    /// This is analog to the `simple` method of the `SerialInterface` struct in `InstrumentRs`,
    /// however, it sets the correct parity, stop bits, and data bits for communication with
    /// Lakeshore 336. The default timeout is set to 3 seconds.
    ///
    /// Arguments:
    /// * `port` - The name of the serial port, e.g., `"/dev/ttyUSB0"` or `"COM3"`.
    pub fn simple(port: &str) -> Result<Instrument<Box<dyn SerialPort>>, InstrumentError> {
        let timeout = Duration::from_secs(3);
        let port = serialport::new(port, 57600)
            .timeout(timeout)
            .parity(serialport::Parity::Odd)
            .data_bits(serialport::DataBits::Seven)
            .stop_bits(serialport::StopBits::One);
        SerialInterface::full(port)
    }
}

/// A rust driver for the Lakeshore336.
///
/// This driver provides functionality to control the Lakeshore/Lakeshore336.
/// See the top-level documentation for an example on how to use this driver.
pub struct Lakeshore336<T: InstrumentInterface> {
    interface: Arc<Mutex<T>>,
    num_channels: usize,
}

impl<T: InstrumentInterface> Lakeshore336<T> {
    /// Create a new Lakeshore336 instance with the given instrument interface.
    ///
    /// # Arguments
    /// * `interface` - An instrument interface that implements the [`InstrumentInterface`] trait.
    pub fn try_new(interface: T) -> Result<Self, InstrumentError> {
        let interface = Arc::new(Mutex::new(interface));

        Ok(Lakeshore336 {
            interface,
            num_channels: 4,
        })
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
        Ok(Channel::new(idx, Arc::clone(&self.interface)))
    }

    /// Query the name of the instrument
    ///
    /// Returns a comma-separated string of:
    /// * Manufacturer ID
    /// * Model number
    /// * Instrument serial number / Option card serial number
    /// * Firmware version
    pub fn get_name(&mut self) -> Result<String, InstrumentError> {
        self.query("*IDN?")
    }

    /// Send a command to the instrument.
    fn sendcmd(&mut self, cmd: &str) -> Result<(), InstrumentError> {
        let mut intf = self.interface.lock().expect("Mutex should not be poisoned");
        intf.sendcmd(cmd)
    }

    /// Query the instrument with a command and return the response as a String.
    fn query(&mut self, cmd: &str) -> Result<String, InstrumentError> {
        self.sendcmd(cmd)?;
        let mut intf = self.interface.lock().expect("Mutex should not be poisoned");
        intf.read_until_terminator()
    }
}

impl<T: InstrumentInterface> Clone for Lakeshore336<T> {
    fn clone(&self) -> Self {
        Self {
            interface: self.interface.clone(),
            num_channels: self.num_channels,
        }
    }
}

/// Channel structure representing a single channel of the Lakeshore336.
///
/// **This structure can only be created through the [`Lakeshore336`] struct.**
///
/// Implementation of an individual channel and commands that go to it.
pub struct Channel<T: InstrumentInterface> {
    idx: usize,
    interface: Arc<Mutex<T>>,
}

impl<T: InstrumentInterface> Channel<T> {
    /// Get a new channel for the given instrument interface.
    ///
    /// This function can only be called from inside of the [`Lakeshore336`] struct.
    fn new(idx: usize, interface: Arc<Mutex<T>>) -> Self {
        Channel { idx, interface }
    }

    /// Get the current temperature reading of this channel.
    ///
    /// Note: If no sensor is connected, the input it disabled, etc., the instrument returns a
    /// reading of zero kelvin. In this case, we return a sensor error.
    pub fn get_temperature(&mut self) -> Result<Temperature, InstrumentError> {
        let resp = self.query("KRDG?")?;
        let val = resp
            .trim()
            .parse::<f64>()
            .map_err(|_| InstrumentError::ResponseParseError(resp))?;
        if val == 0.0 {
            return Err(InstrumentError::SensorError(format!(
                "Channel {} returned 0 K, no sensor connected or input disabled",
                self.idx_mapper()
            )));
        }
        Ok(Temperature::from_kelvin(val))
    }

    /// IDX mapper
    ///
    /// Map the zero-indexed channel number to the letter indexed channel number.
    fn idx_mapper(&self) -> char {
        match self.idx {
            0 => 'A',
            1 => 'B',
            2 => 'C',
            3 => 'D',
            _ => unreachable!("Channel index out of range"),
        }
    }

    /// Send a command for this instrument to an interface.
    fn sendcmd(&mut self, cmd: &str) -> Result<(), InstrumentError> {
        let mut intf = self.interface.lock().expect("Mutex should not be poisoned");
        intf.sendcmd(format!("{}{}", cmd, self.idx_mapper()).as_str())
    }

    /// Query the instrument with a command and return the response as a String.
    fn query(&mut self, cmd: &str) -> Result<String, InstrumentError> {
        self.sendcmd(cmd)?;
        let mut intf = self.interface.lock().expect("Mutex should not be poisoned");
        intf.read_until_terminator()
    }
}

impl<T: InstrumentInterface> Clone for Channel<T> {
    fn clone(&self) -> Self {
        Self {
            idx: self.idx,
            interface: self.interface.clone(),
        }
    }
}
