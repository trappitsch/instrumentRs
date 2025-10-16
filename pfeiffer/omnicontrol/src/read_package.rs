//! Module to read and decipher a package.
//!
//! All string slicing in here is done assuming that the package is valid, which means it only
//! contains ASCII characters! Validity of the package is checked before passing it to this module,
//! otherwise, it is a bug and should be reported.

use instrumentrs::InstrumentError;

use crate::package_utils::{DataType, calculate_checksum};

/// The read package structure that holds the message itself.
pub struct ReadPackage {
    message: String,
}

impl ReadPackage {
    /// Try to create a new ReadPackage.
    ///
    /// Returns `Ok(Self)` if the package is valid, otherwise returns an error.
    ///
    /// The following possible errors are checked:
    /// * CRC is invalid
    /// * Package is one of three error messages.
    pub fn try_new(msg: &str) -> Result<Self, InstrumentError> {
        // Check that package is at least 13 characters long.
        if msg.len() < 13 {
            return Err(InstrumentError::InstrumentStatus(format!(
                "The package received from the instrument is too short: {}",
                msg
            )));
        }

        // Check that the CRC is valid.
        let (msg, crc_exp) = msg.split_at(msg.len() - 3);
        let crc_calc = calculate_checksum(msg);
        if crc_calc != crc_exp {
            return Err(InstrumentError::ChecksumInvalid);
        }

        // dump the first part of the message
        let (_, msg) = msg.split_at(8);

        // get length of data and the actual data
        let (len_str, data) = msg.split_at(2);

        let len = len_str.parse::<usize>().map_err(|_| {
            InstrumentError::ResponseParseError(format!("Data length is not a number: {msg}"))
        })?;

        if data.len() != len {
            return Err(InstrumentError::InstrumentStatus(format!(
                "Data length does not match length field: {} != {} in message: {}",
                data.len(),
                len,
                msg
            )));
        }

        Ok(Self {
            message: data.to_string(),
        })
    }

    /// Get a string back from an expected string response.
    pub fn get_data_string(&self) -> String {
        self.message.trim().to_string()
    }
}
