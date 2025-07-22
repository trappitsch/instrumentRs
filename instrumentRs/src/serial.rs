//! This module provides implementations generating a serial port interface.
//!
//! This module is only available when the `serial` feature is enabled. It uses the [`serialport`]
//! crate in order to create a blocking connection to the specified port.

#![cfg(feature = "serial")]

use std::time::Duration;

use serialport::{SerialPort, SerialPortBuilder};

use crate::{Instrument, InstrumentError};

/// A blocking serial port implementation using the [`serialport`] crate.
///
/// You have the possibility to create an instrument interface from a simple serial port
/// configuration (port and baud rate) or a full featured serial port configuration using a
/// [`serialport::SerialPortBuilder`] structure.
///
/// # Returns
/// Returns a [`Result`] containing an [`Instrument`] with the serial interface if successful,
/// or an [`InstrumentError`] if there was an error opening the port.
#[derive(Debug)]
pub struct SerialInterface {}

impl SerialInterface {
    /// Try to create a Instrument interface with a simple serial port configuration.
    ///
    /// The timeout is by default set to 3 seconds.
    ///
    /// # Arguments
    /// * `port` - The name of the serial port, e.g., `"/dev/ttyUSB0"` or `"COM3"`.
    /// * `baud` - The baud rate for the serial communication, e.g., `9600`.
    ///   [`serialport::SerialPortBuilder`] and the [`serialport::new`] function for more details.
    pub fn simple(
        port: &str,
        baud: u32,
    ) -> Result<Instrument<Box<dyn SerialPort>>, InstrumentError> {
        let timeout = Duration::from_secs(3);
        let port = serialport::new(port, baud).timeout(timeout).open()?;
        Ok(Instrument::new(port, timeout))
    }

    /// Try to create a new Instrument interface with a full featured serial port interface.
    ///
    /// Here, you can specify any additional parameters that is accepted by the [`serialport`]
    /// crate. You also have to specify the timeout inside your [`serialport::SerialPortBuilder`].
    /// structure. This timeout is then passed on to the [`Instrument`] interface.
    ///
    /// This function simply takes your [`serialport::SerialPortBuilder`], opens the port, and reads the set
    /// timeout from the port to pass on to the [`Instrument`] interface.
    ///
    /// # Arguments
    /// * `builder` - A [`serialport::SerialPortBuilder`] that contains all the parameters for the
    ///   connection to the serial port.
    pub fn full(
        builder: SerialPortBuilder,
    ) -> Result<Instrument<Box<dyn SerialPort>>, InstrumentError> {
        let port = builder.open()?;
        let timeout = port.timeout();
        Ok(Instrument::new(port, timeout))
    }
}
