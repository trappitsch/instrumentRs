//! Module to read packages that are returnd by the Agilent4Uhv instrument.

use instrumentrs::InstrumentError;

use crate::utils::calculate_crc;

/// Read package structure that can decipher a given `&[u8]`.
pub struct ReadPackage {
    /// The package data, stripped of STX, ADDR, ETX, and CRC.
    data: Vec<u8>,
}

impl ReadPackage {
    /// Create a new instance of the [`ReadPackage`] struct.
    ///
    /// This will already check if the CRC of the package is valid. If not, it will return an
    /// [`InstrumentError`].
    ///
    /// # Arguments
    /// * `data`: The byte slice containing the package data (full package from STX to CRC).
    pub fn try_new(data: &[u8]) -> Result<Self, InstrumentError> {
        if data.len() < 6 {
            return Err(InstrumentError::PackageInvalid(format!(
                "Package received from instrument is too short: {:?}",
                data
            )));
        }

        let crc_rec = &data[data.len() - 2..];
        let crc_exp = calculate_crc(&data[1..data.len() - 2]);
        if crc_rec != crc_exp {
            return Err(InstrumentError::ChecksumInvalid);
        }

        Ok(Self {
            data: data[2..data.len() - 3].to_vec(),
        })
    }

    /// Evaluate the package as an ackowledgement package.
    pub fn ack_pkg(&self) -> Result<(), InstrumentError> {
        // Slicing is okay as the package must be at least 1 byte long
        let chk = self.data[0];
        if chk != 0x06 {
            let err_str = match chk {
                0x15 => "NACK",
                0x32 => "Unknown Window",
                0x33 => "Data Type Error",
                0x35 => "Win disabled",
                _ => "Unknown Error",
            };
            return Err(InstrumentError::NotAcknowledged(err_str.into()));
        }
        Ok(())
    }

    /// Get an integer from the numeric package.
    pub fn int_pkg(&self) -> Result<i64, InstrumentError> {
        let s = str::from_utf8(&self.data[4..])
            .map_err(|_| InstrumentError::ResponseParseError("Invalid UTF8".into()))?;
        let val = s
            .trim()
            .parse::<i64>()
            .map_err(|_| InstrumentError::ResponseParseError(s.into()))?;
        Ok(val)
    }

    /// Get a float from the numeric package.
    pub fn float_pkg(&self) -> Result<f64, InstrumentError> {
        let s = str::from_utf8(&self.data[4..])
            .map_err(|_| InstrumentError::ResponseParseError("Invalid UTF8".into()))?;
        let val = s
            .trim()
            .parse::<f64>()
            .map_err(|_| InstrumentError::ResponseParseError(s.into()))?;
        Ok(val)
    }

    /// Get a string from an alphanumeric package.
    pub fn alphanumeric_pkg(&self) -> Result<String, InstrumentError> {
        let s = str::from_utf8(&self.data[4..])
            .map_err(|_| InstrumentError::ResponseParseError("Invalid UTF8".into()))?;
        Ok(s.trim().to_string())
    }
}
