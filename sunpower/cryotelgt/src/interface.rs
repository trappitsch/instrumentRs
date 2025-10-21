//! Provide a serial interface for the CryoTel GT.

use std::time::Duration;

use instrumentrs::{Instrument, InstrumentError, SerialInterface};
use serialport::SerialPort;

/// A SerialInterface for the CryoTel GT.
///
/// Builds an InstrumentRs SerialInterface with the correct parity, stop bits, and data bits for
/// communication with the CryoTel GT.
#[derive(Debug)]
pub struct SerialInterfaceCryoTelGt {}

impl SerialInterfaceCryoTelGt {
    /// Try to create an Instrument interface with a simple serial port configuration.
    ///
    /// This is analog to the `simple` method of the `SerialInterface` struct in `InstrumentRs`,
    /// however, it sets the correct parity, stop bits, and data bits for communication with
    /// CryoTel GT. The default timeout is set to 3 seconds.
    ///
    /// Arguments:
    /// * `port` - The name of the serial port, e.g., `"/dev/ttyUSB0"` or `"COM3"`.
    pub fn simple(port: &str) -> Result<Instrument<Box<dyn SerialPort>>, InstrumentError> {
        let timeout = Duration::from_secs(3);
        let port = serialport::new(port, 4800)
            .timeout(timeout)
            .data_bits(serialport::DataBits::Eight)
            .stop_bits(serialport::StopBits::One);
        SerialInterface::full(port)
    }
}
