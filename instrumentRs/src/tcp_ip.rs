//! This module provides the implementation for an instrument controlled via TCP/IP.
//!
//! It includes a blocking implementation of the `Instrument` trait using the
//! [`std::net::TcpStream`] struct.

use std::{
    io::{Read as _, Write},
    net::{TcpStream, ToSocketAddrs},
    time::Duration,
};

use crate::{InstrumentError, InstrumentInterface};

/// A blocking TCP/IP implementation using the [`std::net::TcpStream`] struct.
#[derive(Debug)]
pub struct TcpIpInstrument {
    port: std::net::TcpStream,
    terminator: String,
    timeout: Duration,
}

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
    pub fn try_new<A: ToSocketAddrs>(sock_addr: A) -> Result<Self, InstrumentError> {
        let stream = TcpStream::connect(sock_addr)?;
        let timeout = Duration::from_secs(3);
        stream.set_write_timeout(Some(timeout))?;
        stream.set_read_timeout(Some(timeout))?;
        Ok(Self {
            port: stream,
            terminator: "\n".to_string(),
            timeout,
        })
    }
}

impl InstrumentInterface for TcpIpInstrument {
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

    fn set_timeout(&mut self, timeout: Duration) -> Result<(), InstrumentError> {
        self.port.set_read_timeout(Some(timeout))?;
        self.port.set_write_timeout(Some(timeout))?;
        self.timeout = timeout;
        Ok(())
    }

    fn write_raw(&mut self, data: &[u8]) -> Result<(), InstrumentError> {
        self.port.write_all(data)?;
        self.port.flush()?;
        Ok(())
    }
}
