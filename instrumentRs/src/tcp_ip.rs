//! This module provides the implementation for an instrument controlled via TCP/IP.
//!
//! It includes a blocking implementation of the `Instrument` trait using the
//! [`std::net::TcpStream`] struct.

use std::{
    net::{TcpStream, ToSocketAddrs},
    time::Duration,
};

use crate::{Instrument, InstrumentError};

/// A blocking TCP/IP implementation using the [`std::net::TcpStream`] struct.
#[derive(Debug)]
pub struct TcpIpInstrument {}

impl TcpIpInstrument {
    /// Try to create a new instance of `TcpIpInstrument`.
    ///
    /// The terminator is by default set to `"\n"`, but can be changed using the `set_terminator`
    /// function. Not that the terminator is automatically appended to commands and reading
    /// responses will read until the terminator is found.
    ///
    /// If no read timeout is set, which is possible for the `TcpStream`, we set a manual timeout
    /// of three seconds. This can of course be adjusted with the `set_timeout` function. The
    /// reason for this is that we do not want to infinitely block, as this is not wanted for
    /// instrument communications, especially when they are blocking.
    ///
    /// # Arguments
    /// * `sock_addr` - Socket address.
    pub fn try_new<A: ToSocketAddrs>(
        sock_addr: A,
    ) -> Result<Instrument<TcpStream>, InstrumentError> {
        let stream = TcpStream::connect(sock_addr)?;
        let timeout = Duration::from_secs(3);
        stream.set_write_timeout(Some(timeout))?;
        stream.set_read_timeout(Some(timeout))?;
        Ok(Instrument::new(stream, timeout))
    }
}
