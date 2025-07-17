//! This module provides the implementation for an instrument controlled via a serial port.
//!
//! It includes a blocking implementation of the `Instrument` trait using the [`serialport`] crate.

use std::time::Duration;

use serialport::{SerialPort, SerialPortBuilder};

use crate::{InstrumentError, InstrumentInterface};

/// A blocking serial port implementation using the [`serialport`] crate.
#[derive(Debug)]
pub struct SerialInstrument {
    port: Box<dyn SerialPort>,
    terminator: String,
    timeout: Duration,
}

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
    pub fn try_new(spb: SerialPortBuilder) -> Result<Self, InstrumentError> {
        let port = spb.open()?;
        let timeout = port.timeout();
        Ok(SerialInstrument {
            port,
            terminator: "\n".to_string(),
            timeout,
        })
    }
}

impl InstrumentInterface for SerialInstrument {
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), InstrumentError> {
        self.port.read_exact(buf)?;
        Ok(())
    }

    fn get_terminator(&self) -> &str {
        self.terminator.as_str()
    }

    fn set_terminator(&mut self, terminator: &str) {
        self.terminator = terminator.to_string();
    }

    fn get_timeout(&self) -> Duration {
        self.port.timeout()
    }

    fn set_timeout(&mut self, timeout: Duration) -> Result<(), InstrumentError> {
        self.port.set_timeout(timeout)?;
        self.timeout = timeout;
        Ok(())
    }

    fn write_raw(&mut self, data: &[u8]) -> Result<(), InstrumentError> {
        self.port.write_all(data)?;
        self.port.flush()?;
        Ok(())
    }
}
