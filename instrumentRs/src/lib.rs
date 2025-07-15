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
//! repository on GitHub in order to get your driver added here. This will mean that we can take
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

// mod async_serial;
mod loopback;
mod serial;

// pub use async_serial::AsyncSerialInstrument;
pub use loopback::LoopbackInterface;
pub use serial::{SerialInstrument, SerialInstrumentError};

use thiserror::Error;

/// The error enum for all instruments.
///
/// For any command sending or querying, your instrument should return either an empty result or a
/// result with the query where this Error is the alternative. `InstrumentError` makes it easy to
/// propagate all the sending commands, querying errors forward with the ? operator such that
/// errors propagate nicely. If this is not possible, it is considered a bug and should be
/// reported.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum InstrumentError {
    /// The channel index requested is out of range.
    #[error(
        "Channel with index {idx} is out of range. Number of channels available: {nof_channels}"
    )]
    ChannelIndexOutOfRange {
        /// Index of the channel that is out of range.
        idx: usize,
        /// Total number of channels.
        nof_channels: usize,
    },
    /// A given float value is out of the specified range.
    #[error("Float value {value} is out of range. Allowed range is [{min}, {max}]")]
    FloatValueOutOfRange {
        /// The value that is out of range.
        value: f64,
        /// The minimum value that is allowed.
        min: f64,
        /// The maximum value that is allowed.
        max: f64,
    },
    /// The called command is not supported by this interface.
    #[error("This command is not supported by this interface.")]
    InterfaceCommandNotSupported,
    /// A given integer value is out of the specified range.
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
    /// Timeout occured while waiting for a response to a query.
    #[error(
        "Timeout occured while waiting for a response to query: {query}. Timeout was set to {timeout:?}."
    )]
    TimeoutQuery {
        /// The query that timed out.
        query: String,
        /// The timeout that was set.
        timeout: std::time::Duration,
    },
}

/// The `InstrumentInterface` trait defines the interface for controlling instruments.
///
/// It currently contains a method for sending commands and querying responsed from the instrument.
/// A blocking implmentation for these methods should probably always be requried.
///
/// Furthermore, additional methods for reading and writing data in blocking mode and
/// asynchronously can be provided, however, are not currently required as part of the trait.
pub trait InstrumentInterface {
    /// Send a command to the instrument.
    ///
    /// This function takes the command, appends the terminator, and writes it to the instrument.
    /// The interface is then flushed to ensure that the command is sent immediately.
    /// instrument.
    ///
    /// # Arguments:
    /// - `_cmd` - A string slice that will be sent to the instrument.
    fn sendcmd(&mut self, _cmd: &str) -> Result<(), InstrumentError> {
        Err(InstrumentError::InterfaceCommandNotSupported)
    }

    /// Query the instrument with a command and return the response as a String.
    ///
    /// This function uses `sendcmd` to send the command and then reads the response character by
    /// character until the response string ends with the terminator. If no terminator is
    /// encountered, the function will block until the timeout is reached. If a non-UTF-8 byte is
    /// received, an error is printed to stderr and the byte is skipped.
    ///
    /// # Arguments
    /// * `_cmd` - The command to send to the instrument for which we expect a response.
    fn query(&mut self, _cmd: &str) -> Result<String, InstrumentError> {
        Err(InstrumentError::InterfaceCommandNotSupported)
    }

    /// Set the terminator of an interface from a `&str`.
    ///
    /// # Arguments:
    /// - `_terminator` - A string slice that will be used as the terminator for commands
    fn set_terminator(&mut self, _terminator: &str) {}
}
