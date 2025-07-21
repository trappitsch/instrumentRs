//! This module provides the implementation for an instrument controlled via a serial port.
//!
//! It includes a blocking implementation of the `Instrument` trait using the [`serialport`] crate.

use serialport::{SerialPort, SerialPortBuilder};

use crate::{Instrument, InstrumentError};

/// A blocking serial port implementation using the [`serialport`] crate.
#[derive(Debug)]
pub struct SerialInstrument {}

impl SerialInstrument {
    /// Try to create a new instance of `SerialInstrument`.
    ///
    /// The terminator is by default set to `"\n"`, but can be changed using the `set_terminator`
    /// function. Not that the terminator is automatically appended to commands and reading
    /// responses will read until the terminator is found.
    ///
    /// # Arguments
    /// * `spb` - A `SerialPortBuilder` to configure the serial port.
    ///   [`serialport::SerialPortBuilder`] and the [`serialport::new`] function for more details.
    pub fn try_new(
        spb: SerialPortBuilder,
    ) -> Result<Instrument<Box<dyn SerialPort>>, InstrumentError> {
        let port = spb.open()?;
        let timeout = port.timeout();
        Ok(Instrument::new(port, timeout))
    }
}
