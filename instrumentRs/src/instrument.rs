//! This module provides the main implementation for the Instrument Interface trait.
//!
//! It can be called with any type that implements [`std::io::Read`] and [`std::io::Write`],
//! such as [`std::net::TcpStream`] or [`serialport::SerialPort`].

use std::time::Duration;

use thiserror::Error;

use crate::InstrumentInterface;

/// A general instrument interface that can be built with any interface that implements
/// [`std::io::Read`] and [`std::io::Write`].
///
/// This struct can be used to communicate with instruments over the various interfaces. Handy
/// shortcuts for creating various interfaces are provides as well. However, this general
/// implementation can also be used with any other types that are not provided by `InstrumentRs`.
///
/// # Example
///
/// The following shows a simple example on how to create an [`Instrument`] interface from your own
/// interface that implements [`std::io::Read`] and [`std::io::Write`]. Of course, to just use a
/// simple [`std::net::TcpStream`] as shown here, you can also use the
/// [`crate::TcpIpInterface`] interface.
///
/// ```no_run
/// use std::{net::TcpStream, time::Duration};
///
/// use instrumentrs::Instrument;
///
/// let my_interface = TcpStream::connect("192.168.10.1:8000").unwrap();
/// let inst_interface = Instrument::new(my_interface, Duration::from_secs(3));
/// ```
pub struct Instrument<P: std::io::Read + std::io::Write> {
    port: P,
    terminator: String,
    timeout: Duration,
}

impl<P: std::io::Read + std::io::Write> Instrument<P> {
    /// Try to create a new instance of [`Instrument`] with a given interface.
    pub fn new(port: P, timeout: Duration) -> Self {
        Self {
            port,
            terminator: "\n".to_string(),
            timeout,
        }
    }
}

impl<P: std::io::Read + std::io::Write> InstrumentInterface for Instrument<P> {
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
        self.timeout
    }

    fn write_raw(&mut self, data: &[u8]) -> Result<(), InstrumentError> {
        self.port.write_all(data)?;
        self.port.flush()?;
        Ok(())
    }
}

/// The error enum for all instruments.
///
/// For any command sending or querying, your instrument should return either an empty result or a
/// result with the query where this Error is the alternative. [`InstrumentError`] makes it easy to
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
    /// Error when an invalid argument is passed to a function. This error contains only an error
    /// message, but no arguments. It is intended for the user.
    #[error("{0}")]
    InvalidArgument(String),
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
    #[cfg(feature = "serial")]
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
