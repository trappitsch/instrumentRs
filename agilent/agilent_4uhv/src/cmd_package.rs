//! Module to deal with the command packages for the Agilent 4UHV.

use instrumentrs::InstrumentError;

use crate::utils::calculate_crc;

/// What type of command is being sent?
enum CommandType {
    /// Read from the controller.
    Read = 0x30,
    /// Write to the controller.
    Write = 0x31,
}

/// Data package to be sent to the controller.
///
/// Three options are possible:
/// * Logic (L): 1 bye long, either '0' (False/Off) or '1' (True/On)
/// * Numberic (N): 6 bytes long, valud characters, valid characters: '-', '.', '0', ..., '9'
/// * Alphanumeric (A): 48 bytes long, valid characters are ASCII from blank to '_'
///
/// The following `from`, `try_from` implementations are provided:
/// * `From<bool>` for logic data packages
/// * `TryFrom<i64>` for numeric data packages
/// * `TryFrom<f64>` for numeric data packages
/// * `TryFrom<&str>` for alphanumeric data packages
#[derive(Debug, Clone)]
pub struct Data {
    data_vec: Vec<u8>,
}

impl From<bool> for Data {
    fn from(value: bool) -> Self {
        let data_vec = if value { vec![b'1'] } else { vec![b'0'] };
        Data { data_vec }
    }
}

impl TryFrom<i64> for Data {
    type Error = InstrumentError;

    /// Create a numeric data package from an integer value.
    ///
    /// As the data package can in this case at most be 6 bytes long, this raises an
    /// [`InstrumentError`] if the value is out of range.
    fn try_from(value: i64) -> Result<Self, InstrumentError> {
        if value >= 1_000_000 || value <= -100_000 {
            return Err(InstrumentError::IntValueOutOfRange {
                value,
                min: -10_000,
                max: 100_000,
            });
        }
        let data_vec = format!("{:06}", value).into_bytes();
        Ok(Data { data_vec })
    }
}

impl TryFrom<f64> for Data {
    type Error = InstrumentError;

    /// Create a numeric data package from a floating point value.
    ///
    /// As the data package can in this case at most be 6 bytes long, this raises an
    /// [`InstrumentError`] if the value is out of range.
    fn try_from(value: f64) -> Result<Self, InstrumentError> {
        if value >= 1_000_000.0 || value <= -100_000.0 {
            return Err(InstrumentError::FloatValueOutOfRange {
                value,
                min: -10_000.0,
                max: 100_000.0,
            });
        }
        let mut data_vec = format!("{:06}", value).into_bytes();
        data_vec.truncate(6);
        if data_vec.last() == Some(&b'.') {
            // Remove trailing dot if present
            data_vec.pop();
            data_vec.insert(0, b'0'); // Add leading zero if needed
        }
        Ok(Data { data_vec })
    }
}

impl TryFrom<&str> for Data {
    type Error = InstrumentError;

    /// Try to create an alphanumeric data package from a string slice. Checks that the string is
    /// not longer than 48 bytes and that all characters are in the valid ASCII range: from space
    /// (`0x20`) to `_` (`0x5F`). Note that this ASCII range does not include lowercase letters.
    fn try_from(value: &str) -> Result<Self, InstrumentError> {
        if value.len() > 48 {
            return Err(InstrumentError::InvalidArgument(format!(
                "String with length {} is too long. Maximum allowed length is 48.",
                value.len()
            )));
        }
        let data_vec = value.as_bytes().to_vec();

        // Check that all characters are in the valid ASCII range
        if !data_vec.iter().all(|&b| (0x20..=0x5F).contains(&b)) {
            return Err(InstrumentError::InvalidArgument("String contains an invalid character. Only characters from ASCII space to '_' are allowed.".into()));
        }

        Ok(Data { data_vec })
    }
}

/// Represents a read or write command package for the Agilent 4UHV.
pub struct CommandPackage {
    /// Start of transmission byte: 0x02
    stx: u8,
    /// Address of the device byte: 0 - 32
    addr: u8,
    /// Window for command: `000` - `999`, encoded as 3-digit ASCII
    win: Vec<u8>,
    /// Command code byte: 0x30 for read, 0x31 for write
    com: u8,
    /// Data payload, if write command, as a vector of characters
    data: Option<Data>,
    /// End of transmission byte: 0x03
    etx: u8,
    /// Checksum 2 bytes: XOR of <ADDR>, <WIN>, <COM>, <DATA>, <ETX>
    crc: [u8; 2],
    /// Full vector
    vec: Vec<u8>,
}

impl CommandPackage {
    /// Create a new read command package for the given window number.
    ///
    /// This function will panic if the window number is not between 0..=999.
    pub fn new_read(win: u16, addr: u8) -> Self {
        Self::new(CommandType::Read, win, None, addr)
    }

    /// Create a new write command package for the given window number and data package.
    ///
    /// This function will panic if the window number is not between 0..=999.
    pub fn new_write(win: u16, data: Data, addr: u8) -> Self {
        Self::new(CommandType::Write, win, Some(data), addr)
    }

    fn new(cmd_type: CommandType, win: u16, data: Option<Data>, addr: u8) -> Self {
        if win > 999 {
            panic!("Window number must be between 0 and 999");
        }
        let stx = 0x02;
        let addr = 0x80 + addr; // Does not matter when using serial.
        let win = format!("{:03}", win).into_bytes();
        let com = cmd_type as u8;
        let etx = 0x03;
        let crc = [0x00, 0x00]; // Placeholder, will be calculated later.

        let mut command_package = CommandPackage {
            stx,
            addr,
            win,
            com,
            data,
            etx,
            crc,
            vec: Vec::new(),
        };
        command_package.calculate_crc();
        command_package.build_vec();
        command_package
    }

    /// Get the command package as a byte slice, ready to be sent to the controller.
    pub fn as_bytes(&self) -> &[u8] {
        self.vec.as_slice()
    }

    /// Calculate the checksum (CRC) for the command package and update the internal vector.
    ///
    /// Take the XOR of all bytes from <ADDR> to <ETX> (inclusive) represent it as 1 byte HEX, then
    /// take ASCII of turn that into a 2-byte ASCII
    fn calculate_crc(&mut self) {
        self.build_vec();
        let crc_vec = &self.vec[1..self.vec.len() - 2]; // slice without STX and CRC
        self.crc = calculate_crc(crc_vec);
    }

    /// Build the internal vector representation of the command package.
    fn build_vec(&mut self) {
        let mut vec = Vec::new();
        vec.push(self.stx);
        vec.push(self.addr);
        vec.extend_from_slice(&self.win);
        vec.push(self.com);
        if let Some(data) = &self.data {
            vec.extend_from_slice(&data.data_vec);
        }
        vec.push(self.etx);
        vec.extend_from_slice(&self.crc);
        self.vec = vec;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_logic() {
        let data = Data::from(true);
        assert_eq!(data.data_vec, vec![0x31]);

        let data = Data::from(false);
        assert_eq!(data.data_vec, vec![0x30]);
    }

    #[test]
    fn test_data_numeric_i64_ok() {
        let data = Data::try_from(12345).unwrap();
        assert_eq!(data.data_vec, vec![b'0', b'1', b'2', b'3', b'4', b'5']);

        let data = Data::try_from(-12345).unwrap();
        assert_eq!(data.data_vec, vec![b'-', b'1', b'2', b'3', b'4', b'5']);

        let data = Data::try_from(0).unwrap();
        assert_eq!(data.data_vec, vec![b'0', b'0', b'0', b'0', b'0', b'0']);

        let data = Data::try_from(42).unwrap();
        assert_eq!(data.data_vec, vec![b'0', b'0', b'0', b'0', b'4', b'2']);

        let data = Data::try_from(99_999).unwrap();
        assert_eq!(data.data_vec, vec![b'0', b'9', b'9', b'9', b'9', b'9']);

        let data = Data::try_from(999_999).unwrap();
        assert_eq!(data.data_vec, vec![b'9', b'9', b'9', b'9', b'9', b'9']);

        let data = Data::try_from(-99_999).unwrap();
        assert_eq!(data.data_vec, vec![b'-', b'9', b'9', b'9', b'9', b'9']);
    }

    #[test]
    fn test_data_numeric_i64_err() {
        let data = Data::try_from(1_000_000);
        assert!(data.is_err());

        let data = Data::try_from(-100_000);
        assert!(data.is_err());
    }

    #[test]
    fn test_data_numeric_f64_ok() {
        let data = Data::try_from(123.45).unwrap();
        assert_eq!(data.data_vec, vec![b'1', b'2', b'3', b'.', b'4', b'5']);

        let data = Data::try_from(123.456789).unwrap();
        assert_eq!(data.data_vec, vec![b'1', b'2', b'3', b'.', b'4', b'5']);

        let data = Data::try_from(-123.456789).unwrap();
        assert_eq!(data.data_vec, vec![b'-', b'1', b'2', b'3', b'.', b'4']);

        let data = Data::try_from(0.0).unwrap();
        assert_eq!(data.data_vec, vec![b'0', b'0', b'0', b'0', b'0', b'0']);

        let data = Data::try_from(42.0).unwrap();
        assert_eq!(data.data_vec, vec![b'0', b'0', b'0', b'0', b'4', b'2']);

        let data = Data::try_from(42.3).unwrap();
        assert_eq!(data.data_vec, vec![b'0', b'0', b'4', b'2', b'.', b'3']);

        let data = Data::try_from(999_999.9999999).unwrap();
        assert_eq!(data.data_vec, vec![b'9', b'9', b'9', b'9', b'9', b'9']);

        let data = Data::try_from(-99_999.9999999).unwrap();
        assert_eq!(data.data_vec, vec![b'-', b'9', b'9', b'9', b'9', b'9']);

        let data = Data::try_from(99_999.9).unwrap();
        assert_eq!(data.data_vec, vec![b'0', b'9', b'9', b'9', b'9', b'9']);
    }

    #[test]
    fn test_data_numeric_f64_err() {
        let data = Data::try_from(1_000_000.);
        assert!(data.is_err());

        let data = Data::try_from(-100_000.);
        assert!(data.is_err());
    }

    #[test]
    fn test_alphanumeric_ok() {
        let data = Data::try_from("HELLO_123").unwrap();
        assert_eq!(data.data_vec, b"HELLO_123");

        let data = Data::try_from("123456789_123456789_123456789_123456789_12345678").unwrap();
        assert_eq!(
            data.data_vec,
            b"123456789_123456789_123456789_123456789_12345678"
        );

        let data = Data::try_from(" !\"#$%&'()*+,-./0123456789:;<=>?@").unwrap();
        assert_eq!(data.data_vec, b" !\"#$%&'()*+,-./0123456789:;<=>?@");

        let data = Data::try_from("ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_").unwrap();
        assert_eq!(data.data_vec, b"ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_");
    }

    #[test]
    fn test_alphanumeric_err() {
        let data = Data::try_from("123456789_123456789_123456789_123456789_123456789");
        assert!(data.is_err());

        let data = Data::try_from("lower case letters are bad");
        assert!(data.is_err());
    }

    /// Example from manual: Turn ON Channel 1
    #[test]
    fn test_ex_turn_on_channel_1() {
        let expected: &[u8] = &[0x02, 0x80, 0x30, 0x31, 0x31, 0x31, 0x31, 0x03, 0x42, 0x33];

        let data = Data::from(true);
        let win = 11;
        let cmd_package = CommandPackage::new_write(win, data, 0x00);

        assert_eq!(cmd_package.as_bytes(), expected);
    }

    /// Example from manual: Read Pressure (win 812) Channel 1
    #[test]
    fn test_ex_read_pressure_channel_1() {
        let expected: &[u8] = &[0x02, 0x80, 0x38, 0x31, 0x32, 0x30, 0x03, 0x38, 0x38];

        let win = 812;
        let cmd_package = CommandPackage::new_read(win, 0x00);

        assert_eq!(cmd_package.as_bytes(), expected);
    }
}
