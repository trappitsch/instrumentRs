//! InstrumentRs: Talk to your (scientific) equipment from with Rust
//!
//! The InstrumentRs library provides standardized interfaces to talk to scientific equipment via
//! various different ports. To do so, it provides an [`InstrumentInterface`] trait and its
//! implementations. Furthermore, we also provide an [`InstrumentError`] error type that instrument
//! drivers should return. Any connection type that implements the [`std::io::Read`] and
//! [`std::io::Write`] traits can be used as an instrument interface. Furthermore, we also provide
//! simplified access to the following interfaces:
//!
//! - TCP/IP (blocking) using the [`std::net`] module.
//! - Serial (blocking) using the [`serialport`] crate (feature `"serial"`).
//!
//! We are planning in the future to also support asynchronous interfaces.
//!
//! # Example
//!
//! The following shows a simple example on how to get an [`Instrument`] interface using a simple
//! socket address.
//!
//! ```no_run
//! use std::net::SocketAddr;
//! use instrumentrs::TcpIpInterface;
//!
//! let address = "192.168.1.10:8000";
//! let inst_interface = TcpIpInterface::simple(address);
//! ```
//!
//! You can now take this instrument interface and pass it to any of the instrument drivers, of
//! course assuming that the actual instrument is connected to this interface.
//!
//! # Goals and non-goals of this project
//!
//! InstrumentRs shall provide a simple framework that allows you write your own instrument driver
//! and share it with the community. It should allow you to focus on the driver design itself and
//! take care of the interfacing for you. This allows your driver to be flexible, i.e., a serial
//! device can be connected to a computer via RS232, but can also be connected via an Ethernet to
//! serial interface. InstrumentRs will take care of sending the correct commands for a specified
//! instrument in the background.
//!
//! While InstrumentRs is not a collection of drivers and only allows you to write a simplified
//! driver, we will host drivers here as well and maintain them. This means: If you would like to
//! write a driver but do not want to maintain it, please raise an issue in the InstrumentRs
//! repository on GitHub in order to get your driver added here. This means that we will take
//! over maintainership of the driver and release them as bugs get squished, etc. In order for this
//! to work, all functionality of your instrument driver must be tested with hardware, but also
//! with tests using the provided [`LoopbackInterfaceString`].
//!
//! # Inspiration
//!
//! This project is heavily inspired by the fantastic
//! [`instrumentkit`](https://github.com/instrumentkit/InstrumentKit) library that allows to
//! control instruments from Python.
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
//! Licensed under either
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

#![deny(warnings, missing_docs)]

mod instrument;
mod loopback;
mod serial;
mod tcp_ip;

use std::time::{Duration, Instant};

pub use instrument::{Instrument, InstrumentError};
pub use loopback::LoopbackInterfaceString;
pub use tcp_ip::TcpIpInterface;

#[cfg(feature = "serial")]
pub use serial::SerialInterface;

/// The [`InstrumentInterface`] trait defines the interface for controlling instruments.
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
    /// [`InstrumentError::NotAcknowledged`] error with the incorrect response received in the error
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
            Err(e) => Err(e),
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
                panic!(
                    "Received invalid UTF-8 data: {single_buf:?}. This should be unreachable, as read exact always returns a `u8`. Please report this as a bug."
                );
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
    /// Returns the current timeout of the interface as a [`Duration`]. The default timeout, if not]
    /// implemented, is set to three seconds.
    fn get_timeout(&self) -> Duration {
        Duration::from_secs(3)
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

    /// Write a byte slice to the instrument and flush it after.
    ///
    /// This function takes a byte slice and writes it to the interface. It does NOT append the
    /// terminator. After writing, the interface should be flushed.
    fn write_raw(&mut self, _data: &[u8]) -> Result<(), InstrumentError>;
}
