//! Handles constructing command packages for the instrument.

use crate::{BaseAddress, package_utils::calculate_checksum};

/// Type of the command and it's respective value (noted as * in manual).
enum CommandType {
    Read,
    Write,
}

impl CommandType {
    /// Get the string representation of the command type.
    fn as_str(&self) -> &str {
        match self {
            CommandType::Read => "00",
            CommandType::Write => "10",
        }
    }
}

/// Construct a command package to send to the Omnicontrol.
pub struct CommandPackage {
    /// Sum of base address and indicator address.
    device_address: usize,
    /// The parameter to set or read.
    parameter: usize,
}

impl CommandPackage {
    /// Create a new read command package. Terminator will be added by interface.
    pub fn get_read_pkg(
        base_address: BaseAddress,
        indicator_address: usize,
        parameter: usize,
    ) -> String {
        let pkg = CommandPackage::new(base_address, indicator_address, parameter);
        let mut cmd = format!(
            "{:03}{}{:03}02=?",
            pkg.device_address,
            CommandType::Read.as_str(),
            pkg.parameter
        );
        cmd.push_str(&calculate_checksum(&cmd));
        cmd
    }

    /// Create a new command package.
    ///
    /// Panics if device address or parameter are larger than three digits.
    fn new(base_address: BaseAddress, indicator_address: usize, parameter: usize) -> Self {
        let device_address = usize::from(base_address) + indicator_address;

        assert!(
            device_address < 1000,
            "Device address must be less than 1000, please file a bug report."
        );
        assert!(
            parameter < 1000,
            "Parameter must be less than 1000, please file a bug report."
        );

        CommandPackage {
            device_address,
            parameter,
        }
    }
}
