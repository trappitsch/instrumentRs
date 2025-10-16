//! Library utilities, re-exported in `lib.rs`.
//!
//! Helps to keep the main file cleaner.

use std::fmt::{Display, Formatter};

use instrumentrs::InstrumentError;

/// RS-485 base address of the controller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaseAddress {
    /// Address 0
    Zero = 0,
    /// Address 100
    OneHundred = 1,
    /// Address 200
    TwoHundred = 2,
}

impl Display for BaseAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BaseAddress::Zero => write!(f, "0"),
    BaseAddress::OneHundred => write!(f, "100"),
            BaseAddress::TwoHundred => write!(f, "200"),
        }
    }
}

impl TryFrom<usize> for BaseAddress {
    type Error = InstrumentError;

    fn try_from(value: usize) -> Result<Self, InstrumentError> {
        let res = match value {
            0 => BaseAddress::Zero,
            1 => BaseAddress::OneHundred,
            100 => BaseAddress::OneHundred,
            2 => BaseAddress::TwoHundred,
            200 => BaseAddress::TwoHundred,
            _ => { 
                return Err(InstrumentError::InvalidArgument(
                    "Base address must be one of 0, 100, or 200".to_string(),
                ));
                
            }
        };
        Ok(res)
    }
}

impl From<BaseAddress> for usize {
    fn from(addr: BaseAddress) -> Self {
        match addr {
            BaseAddress::Zero => 0,
            BaseAddress::OneHundred => 100,
            BaseAddress::TwoHundred => 200,
        }
    }
}
