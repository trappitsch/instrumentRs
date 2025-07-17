//! InstrumentRs: Talk to your (scientific) equipment from with Rust
//!
//! The InstrumentRs library provides standardized interfaces to talk to scientific equipment via
//! various different ports. To do so, it provides an `InstrumentInterface` trait and its
//! implementations. Furthermore, we also provide an `InstrumentError` error type that instrument
//! drivers should return.
//!
//! # Currently implemented interfaces are:
//! - Serial (blocking) using the [`serialport`] crate.
//!
//! We are planning to soon also support the following interfaces:
//! - Async serial
//! - TCP/IP blocking
//! - TCP/IP async
//!
//! # Goals and non-goals of this project
//!
//! InstrumentRs shall provide a simple framework that allows you write your own instrument driver
//! and share it with the community. It should allow you to focus on the driver design itself and
//! take care of the interfacing for you. This allows your driver to be flexible, i.e., a serial
//! device can be connected to a computer via RS232, but can also be connected via an ethernet to
//! serial interface. InstrumentRs will take care of sending the correct commands for a specified
//! instrument in the background.
//!
//! While InstrumentRs is not a collection of drivers and only allows you to write a simplified
//! driver, we will host drivers here as well and maintain them. This means: If you would like to
//! write a driver but do not want to maintain it, please raise an issue in the InstrumentRs
//! repository on GitHub in order to get your driver added here. This means that we will take
//! over maintainership of the driver and release them as bugs get squished, etc. In order for this
//! to work, all functionality of your instrument driver must be tested with hardware, but also
//! with tests using the provided `LoopbackInterface`.
//!
//! # Inspiration
//!
//! This project is heavily inspired by the fantastic
//! [`instrumentkit`](https://github.com/instrumentkit/InstrumentKit) library that allows for
//! instrument control from python.
//!
//! # Status
//!
//! This project is currently under active development and (breaking) changes might occure fast. If
//! you are interested in using this project and/or contributing, please get in touch by raising an
//! issue on GitHub. This would also be super valuable as we would learn how it is used, what the
//! need is, etc.
//!
//! # License
//!
//!
//! Licensed under either of
//!
//! - Apache License, Version 2.0 ([LICENSE-APACHE](http://www.apache.org/licenses/LICENSE-2.0))
//! - MIT license ([LICENSE-MIT](http://opensource.org/licenses/MIT))
//!
//! at your option.
//!
//! # Contribution
//!
//! Unless you explicitly state otherwise, any contribution intentionally submitted
//! for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
//! dual licensed as above, without any additional terms or conditions.

#![warn(missing_docs)]

mod loopback;
mod serial;
mod tcp_ip;

use std::time::{Duration, Instant};

pub use loopback::LoopbackInterface;
pub use serial::SerialInstrument;
pub use tcp_ip::TcpIpInstrument;

use thiserror::Error;

/// The error enum for all instruments.
///
/// For any command sending or querying, your instrument should return either an empty result or a
/// result with the query where this Error is the alternative. `InstrumentError` makes it easy to
/// propagate all the sending commands, querying errors forward with the `?` operator such that
/// errors propagate nicely. If this is not possible, it is considered a bug and should be
/// reported.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum InstrumentError {
    /// The instrument did not acknowledge the command that was sent. The response received is
    /// returned in the error as a String.
    #[error("Instrument did not acknowledge the command sent, but responded with: {0}")]
    NotAcknowledged(String),
    /// The channel index requested is out of range. The error contains the index requested and
    /// the number of channels that are currently configured.
    #[error(
        "Channel with index {idx} is out of range. Number of channels available: {nof_channels}"
    )]
    ChannelIndexOutOfRange {
        /// Index of the channel that is out of range.
        idx: usize,
        /// Total number of channels.
        nof_channels: usize,
    },
    /// A given float value is out of the specified range. The error contains the value that was
    /// sent, the minimum value that is allowed, and the maximum value that is allowed.
    #[error("Float value {value} is out of range. Allowed range is [{min}, {max}]")]
    FloatValueOutOfRange {
        /// The value that is out of range.
        value: f64,
        /// The minimum value that is allowed.
        min: f64,
        /// The maximum value that is allowed.
        max: f64,
    },
    /// A given integer value is out of the specified range. The error contains the value that was
    /// sent, the minimum value that is allowed, and the maximum value that is allowed.
    #[error("Integer value {value} is out of range. Allowed range is [{min}, {max}]")]
    IntValueOutOfRange {
        /// The value that is out of range.
        value: i64,
        /// The minimum value that is allowed.
        min: i64,
        /// The maximum value that is allowed.
        max: i64,
    },
    /// Error when reading from/writing to an interface. See [`std::io::Error`] for more details.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// Instrument status is not okay, e.g., a response from the instrument did not succeed with a
    /// given error message. This error contains a string with the error message that is intended
    /// to be displayed for the user, i.e., "Sensor not calibrated". Note that the string is
    /// directly displayed without any further formatting, so you need to ensure that it is
    /// descriptive enough for the user.
    #[error("{0}")]
    InstrumentStatus(String),
    /// Instrument response could not be parsed becuase it was unexpected by the driver. This error
    /// contains the response that was received from the instrument.
    #[error("Response from instrument could not be parsed. Response was: {0}")]
    ResponseParseError(String),
    /// Serial port errors can occur when opening a serial interface. See the [`serialport::Error`]
    /// documentation for more information.
    #[error(transparent)]
    Serialport(#[from] serialport::Error),
    /// Timeout occurred while waiting for a response from the instrument. The error contains the
    /// timeout that was exceeded.
    #[error(
        "Timeout occured while waiting for a response from the instrument. Timeout was set to {0:?}."
    )]
    Timeout(Duration),
    /// Timeout occurred while waiting for a response to a query. The error contains the query
    /// that was sent and the timeout that was exceeded.
    #[error(
        "Timeout occured while waiting for a response to query: {query}. Timeout was set to {timeout:?}."
    )]
    TimeoutQuery {
        /// The query that timed out.
        query: String,
        /// The timeout that was set.
        timeout: Duration,
    },
}

/// The `InstrumentInterface` trait defines the interface for controlling instruments.
///
/// It currently contains a method for sending commands and querying responses from the instrument.
/// A blocking implementation for these methods should probably always be required.
///
/// Furthermore, additional methods for reading and writing data in blocking mode and
/// asynchronously can be provided, however, are not currently required as part of the trait.
pub trait InstrumentInterface {
    /// Check if an acknowledgment is received from the instrument.
    ///
    /// This function checks if the instrument acknowledges the command sent to it with the correct
    /// return value or not. If no acknowledgment is received, it returns an
    /// `InstrumentError::NotAcknowledged` error with the incorrect response received in the error
    /// message.
    ///
    /// # Arguments:
    /// - `_ack` - A string slice that contains the expected acknowledgment response.
    fn check_acknowledgment(&mut self, ack: &str) -> Result<(), InstrumentError> {
        let response = self.read_until_terminator()?;
        if response == ack {
            Ok(())
        } else {
            Err(InstrumentError::NotAcknowledged(response))
        }
    }

    /// Query the instrument with a command and return the response as a String.
    ///
    /// This function uses `sendcmd` to send the command and then reads the response character by
    /// character until the response string ends with the terminator. If no terminator is
    /// encountered, the function will block until the timeout is reached. If a non-UTF-8 byte is
    /// received, an error is printed to `stderr` and the byte is skipped.
    ///
    /// This function has a default implementation, as it uses other interface specific methods in
    /// order to query the instrument.
    ///
    /// # Arguments
    /// * `_cmd` - The command to send to the instrument for which we expect a response.
    fn query(&mut self, cmd: &str) -> Result<String, InstrumentError> {
        self.sendcmd(cmd)?;
        match self.read_until_terminator() {
            Ok(response) => Ok(response),
            Err(InstrumentError::Timeout(tout)) => Err(InstrumentError::TimeoutQuery {
                query: cmd.to_string(),
                timeout: tout,
            }),
            e => e, // should be unreachable
        }
    }

    /// Read an exact number of bytes from the instrument.
    ///
    /// You must provide a mutable buffer that this function will read into. The function will
    /// read as many bytes as the buffer can hold.
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), InstrumentError>;

    /// Read until the terminator is found or the timeout is reached.
    ///
    /// This function reads from the instrument until the terminator is found or the timeout is
    /// reached and returns the read data as a String.
    fn read_until_terminator(&mut self) -> Result<String, InstrumentError> {
        let mut response = String::new();
        let mut single_buf = [0u8];

        let tic = Instant::now();
        let mut timeout_occured = true;

        while (Instant::now() - tic) < self.get_timeout() {
            self.read_exact(&mut single_buf)?;
            if let Ok(val) = str::from_utf8(&single_buf) {
                response.push_str(val);
            } else {
                eprintln!("Received invalid UTF-8 data: {single_buf:?}");
            }
            if response.ends_with(&self.get_terminator()) {
                timeout_occured = false;
                break;
            }
        }

        if timeout_occured {
            Err(InstrumentError::Timeout(self.get_timeout()))
        } else {
            let retval = response.trim();
            Ok(retval.to_string())
        }
    }

    /// Send a command to the instrument.
    ///
    /// This function takes the command, appends the terminator, and writes it to the instrument.
    /// The interface is then flushed to ensure that the command is sent immediately to the
    /// instrument.
    ///
    /// This function has a default implementation, as it uses other interface specific methods in
    /// order to query the instrument.
    ///
    /// # Arguments:
    /// - `_cmd` - A string slice that will be sent to the instrument.
    fn sendcmd(&mut self, cmd: &str) -> Result<(), InstrumentError> {
        let cmd = format!("{}{}", cmd, self.get_terminator());
        self.write(&cmd)
    }

    ///
    /// Get the current terminator of the interface.
    ///
    /// If not implemented, this function will return a default value of `"\n"`.
    fn get_terminator(&self) -> &str {
        "\n"
    }

    /// Set the terminator of an interface from a `&str`.
    ///
    /// # Arguments:
    /// - `_terminator` - A string slice that will be used as the terminator for commands
    fn set_terminator(&mut self, _terminator: &str) {}

    /// Get the current timeout of the interface.
    ///
    /// Returns the current timeout of the interface as a `Duration`. The default timeout, if not
    /// implemented, is set to three seconds.
    fn get_timeout(&self) -> Duration {
        Duration::from_secs(3)
    }

    /// Set the timeout of the interface.
    ///
    /// The default implementation does nothing and just returns `Ok(())`.
    ///
    /// # Arguments:
    /// - `_timeout` - A `Duration` that will be used as the timeout for the interface.
    fn set_timeout(&mut self, _timeout: Duration) -> Result<(), InstrumentError> {
        Ok(())
    }

    /// Write a string to the instrument.
    ///
    /// This function takes a string slice and writes it to the instrument. It does NOT append the
    /// terminator. If you prefer a command that appends the terminator, use `sendcmd`.
    ///
    /// # Arguments:
    /// - `_data` - A string slice that will be written to the instrument.
    fn write(&mut self, data: &str) -> Result<(), InstrumentError> {
        self.write_raw(data.as_bytes())
    }

    /// Write a byte slice to the instrument and flush it afterwards.
    ///
    /// This function takes a byte slice and writes it to the interface. It does NOT append the
    /// terminator. After writing, the interface should be flushed.
    fn write_raw(&mut self, _data: &[u8]) -> Result<(), InstrumentError>;
}
