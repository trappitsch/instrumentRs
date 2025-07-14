//! This module provides the implementation for an instrument controlled via a serial port.
//!
//! It includes a blocking implementation of the `Instrument` trait using the `serialport` crate
//! and an asynchronous implementation using the `tokio_serial` crate.

use std::time::Instant;

use serialport::{SerialPort, SerialPortBuilder};
use thiserror::Error;

use crate::{InstrumentError, InstrumentInterface};

/// The errors that SerialInstrument might return.
#[derive(Clone, Debug, Error)]
#[non_exhaustive]
pub enum SerialInstrumentError {
    /// Serialport errors can occur when opening a serial interface. See the [`serialport::Error`]
    /// documentation for more information.
    #[error(transparent)]
    Serialport(#[from] serialport::Error),
}

/// A blocking serial port implementation using the `serialport` crate.
#[derive(Debug)]
pub struct SerialInstrument {
    port: Box<dyn SerialPort>,
    terminator: String,
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
    pub fn try_new(spb: SerialPortBuilder) -> Result<Self, SerialInstrumentError> {
        Ok(SerialInstrument {
            port: spb.open()?,
            terminator: "\n".to_string(),
        })
    }
}

impl InstrumentInterface for SerialInstrument {
    fn set_terminator(&mut self, terminator: &str) {
        self.terminator = terminator.to_string();
    }

    fn sendcmd(&mut self, cmd: &str) -> Result<(), InstrumentError> {
        let cmd = format!("{}{}", cmd, self.terminator);
        self.port.write_all(cmd.as_bytes())?;
        self.port.flush()?;
        Ok(())
    }

    fn query(&mut self, cmd: &str) -> Result<String, InstrumentError> {
        self.sendcmd(cmd)?;
        let mut response = String::new();
        let mut single_buf = [0u8];

        let tic = Instant::now();
        let mut timeout_occured = true;

        while (Instant::now() - tic) < self.port.timeout() {
            self.port.read_exact(&mut single_buf)?;
            if let Ok(val) = str::from_utf8(&single_buf) {
                response.push_str(val);
            } else {
                eprintln!("Received invalid UTF-8 data: {single_buf:?}");
            }
            if response.ends_with(&self.terminator) {
                timeout_occured = false;
                break;
            }
        }

        if timeout_occured {
            Err(InstrumentError::TimeoutQuery {
                query: cmd.to_string(),
                timeout: self.port.timeout(),
            })
        } else {
            let retval = response.trim();
            Ok(retval.to_string())
        }
    }
}
