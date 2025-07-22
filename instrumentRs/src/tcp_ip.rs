//! This module provides the implementation for an instrument controlled via TCP/IP.
//!
//! It includes a blocking implementation of the `Instrument` trait using the
//! [`std::net::TcpStream`] struct. As this is part of the standard library, this interface is
//! always available as long as the standard library is available.

use std::{
    net::{TcpStream, ToSocketAddrs},
    time::Duration,
};

use crate::{Instrument, InstrumentError};

/// A blocking TCP/IP implementation using [`std::net::TcpStream`].
///
/// You have the possibility to create an instrument interface from a simple socket address, or to
/// create a full featured TCP/IP interface with additional parameters and pass the `full` method an
/// open `TcpStream`.
///
/// # Returns
/// Returns a [`std::result`] containing an `Instrument` with the TCP/IP interface if successful,
/// or an `InstrumentError` if there was an error opening the port.
#[derive(Debug)]
pub struct TcpIpInterface {}

impl TcpIpInterface {
    /// Try to create a new Instrument interface with of TCP/IP interface.
    ///
    /// The timeout in the simple implementation is set to 3 seconds for both reading and writing.
    ///
    /// # Arguments
    /// * `sock_addr` - Socket address.
    pub fn simple<A: ToSocketAddrs>(
        sock_addr: A,
    ) -> Result<Instrument<TcpStream>, InstrumentError> {
        let stream = TcpStream::connect(sock_addr)?;
        let timeout = Duration::from_secs(3);
        stream.set_write_timeout(Some(timeout))?;
        stream.set_read_timeout(Some(timeout))?;
        Ok(Instrument::new(stream, timeout))
    }

    /// Try to create a new Instrument interface from an open TCP/IP stream.
    ///
    /// This allows you to specify timeouts, etc. For the internal `Instrument` timeout, we will
    /// use the `read_timeout` of the `TcpStream`. If this is `None`, we will use a default
    /// timeout of 3 seconds.
    ///
    /// # Arguments
    /// * `stream` - An already open `TcpStream`.
    pub fn full(stream: TcpStream) -> Result<Instrument<TcpStream>, InstrumentError> {
        let timeout = stream.read_timeout()?.unwrap_or(Duration::from_secs(3));
        Ok(Instrument::new(stream, timeout))
    }
}
