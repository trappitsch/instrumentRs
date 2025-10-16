//! Utilities for handling packages.

use instrumentrs::InstrumentError;

/// Calculate the checksum for the command package.
///
/// Sum of ASCII values from start (address) to end of data field, modulo 256.
///
/// Return: A three digit string with leading zeros if necessary.
pub fn calculate_checksum(cmd: &str) -> String {
    let sum: u8 = cmd.bytes().fold(0u8, |acc, b| acc.wrapping_add(b));
    format!("{:03}", sum)
}

/// Data type structure, as described in section 2.4 of the manual.
///
/// FIXME: We should refractor the parsing of the values into the read package function!
/// The names are as given in the table, but in Camel Case instead of snake_case.
///
/// Transformations for all data types are implemented, even if they are currently unused in the
/// driver. This is to keep the driver easily extendable in the future.
#[derive(Debug, PartialEq, Eq)]
pub enum DataType {
    /// Old logical value (true or false) of length 6 (000_000 is false, 111_111 is true).
    BooleanOld,
    /// Positive hole number of length 6 (000_000 to 999_999).
    UInteger,
    /// Fixed point number, unsigned, length 6, last two digits are decimal (i.e., 001571 => 15.71)
    UReal,
    /// String, any ASCII codes between 32 and 127, length 6.
    String,
    /// New logical value (true or false) of length 1 (0 is false, 1 is true).
    BooleanNew,
    /// Positive short number of length 3 (000 to 999).
    UShortInt,
    /// Positive exponential number. The last two digits are exponent -20. Length 6 (i.e., 1000023
    /// => 1.0e23).
    UExpoNew,
    /// Length 16 string with ASCII codes between 32 and 127.
    String16,
    /// Length 8 string with ASCII codes between 32 and 127.
    String8,
}

impl DataType {
    /// Get the correct datatype from the datatype number as given in the manual.
    ///
    /// Panics if the number is not valid. This is not user exposed, only a problem for the lib.
    pub fn from_type_number(type_number: usize) -> Self {
        match type_number {
            0 => DataType::BooleanOld,
            1 => DataType::UInteger,
            2 => DataType::UReal,
            4 => DataType::String,
            6 => DataType::BooleanNew,
            7 => DataType::UShortInt,
            10 => DataType::UExpoNew,
            11 => DataType::String16,
            12 => DataType::String8,
            _ => panic!("Invalid datatype number, please file a bug report."),
        }
    }

    /// Parse a boolean datatype and return a bool.
    pub fn parse_to_bool(&self, data: &str) -> Result<bool, InstrumentError> {
        match self {
            DataType::BooleanOld => match data {
                "000000" => Ok(false),
                "111111" => Ok(true),
                _ => Err(InstrumentError::ResponseParseError(data.to_string())),
            },
            DataType::BooleanNew => match data {
                "0" => Ok(false),
                "1" => Ok(true),
                _ => Err(InstrumentError::ResponseParseError(data.to_string())),
            },
            _ => panic!(
                "This should never be called for non-boolean datatypes, please file a bug report."
            ),
        }
    }

    /// Parse a whole number datatype and return a `usize`.
    pub fn parse_to_usize(&self, data: &str) -> Result<usize, InstrumentError> {
        match self {
            DataType::UInteger | DataType::UShortInt => data
                .trim()
                .parse::<usize>()
                .map_err(|_| InstrumentError::ResponseParseError(data.to_string())),
            _ => panic!(
                "This should never be called for non-integer datatypes, please file a bug report."
            ),
        }
    }

    /// Parse a fixed point or exponential number datatype and return a `f64`.
    pub fn parse_to_f64(&self, data: &str) -> Result<f64, InstrumentError> {
        match self {
            DataType::UReal => {
                let int_value = data
                    .trim()
                    .parse::<u64>()
                    .map_err(|_| InstrumentError::ResponseParseError(data.to_string()))?;
                Ok(int_value as f64 / 100.0)
            }
            DataType::UExpoNew => {
                if data.len() != 6 {
                    return Err(InstrumentError::ResponseParseError(data.to_string()));
                }
                let mantissa_str = &format!("{}.{}", &data[0..1], &data[1..4]);
                let exponent_str = &data[4..];
                let mantissa = mantissa_str
                    .trim()
                    .parse::<f64>()
                    .map_err(|_| InstrumentError::ResponseParseError(data.to_string()))?;
                let exponent = exponent_str
                    .trim()
                    .parse::<i32>()
                    .map_err(|_| InstrumentError::ResponseParseError(data.to_string()))?;
                Ok(mantissa * 10f64.powi(exponent - 20))
            }
            _ => panic!(
                "This should never be called for non-float datatypes, please file a bug report."
            ),
        }
    }
}
