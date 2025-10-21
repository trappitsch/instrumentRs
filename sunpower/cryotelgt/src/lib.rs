//! A rust driver for the Sunpower CryoTel GT.
//!
//! This driver provides functionality to control a Sunpower CryoTel GT Gen II cryocooler via,
//! e.g., RS-232, from Rust.
//!
//! Note that the CryoTel GT always returns the actually set value. This driver does not check if
//! the set value is the same as the requested value and it is up the the user to verify that it
//! is, i.e., by querying the value again after setting it. This is a current limitation of this
//! driver, however, is hopefully acceptable for now. If you need this functionality, please file
//! an issue in the GitHub repository.
//!
//! # Example
//!
//! This example shows the usage of the CryoTel GT driver with a serial connection.
//!
//! ```no_run
//! use sunpower_cryotelgt::{CryoTelGt, SerialInterfaceCryoTelGt};
//!
//! // Create a serial interface and connect to the CryoTel GT
//! let interface = SerialInterfaceCryoTelGt::simple("/dev/ttyUSB0").unwrap();
//! let mut cryotel = CryoTelGt::try_new(interface).unwrap();
//!
//! // Get the current temperature
//! let temperature = cryotel.get_temperature().unwrap();
//!```

#![deny(warnings, missing_docs)]

use std::{
    fmt::Display,
    sync::{Arc, Mutex},
};

use instrumentrs::{InstrumentError, InstrumentInterface};
use measurements::{Power, Temperature};

pub use interface::SerialInterfaceCryoTelGt;

mod interface;

/// Status of the CryoTel GT.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoolerState {
    /// Cooler is off or shutting down.
    Disabled,
    /// Cooler is running.
    Enabled,
}

impl Display for CoolerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoolerState::Disabled => write!(f, "Disabled"),
            CoolerState::Enabled => write!(f, "Enabled"),
        }
    }
}

/// Control modes for the CryoTel GT.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControlMode {
    /// Controller will maintain constant power as set by `set_power_setpoint`.
    Power = 0,
    /// Controller will maintain constant temperature as set by `set_temperature_setpoint`.
    Temperature = 2,
}

impl Display for ControlMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ControlMode::Power => write!(f, "Power"),
            ControlMode::Temperature => write!(f, "Temperature"),
        }
    }
}

/// Thermostat mode for the CryoTel GT.
///
/// This functionality allows the user to add a thermostat to the system which can be used to shut
/// down the cryocooler. See the manual for more information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThermostatMode {
    /// Thermostat functionality disabled.
    Disabled = 0,
    /// Thermostat functionality enabled.
    Enabled = 1,
}

impl Display for ThermostatMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThermostatMode::Disabled => write!(f, "Disabled"),
            ThermostatMode::Enabled => write!(f, "Enabled"),
        }
    }
}

/// Stop modes for the CryoTel GT.
///
/// This determines what stop commands the cooler will listen to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StopMode {
    /// Allows the cooler to be started / stopped via software commands.
    Remote = 0,
    /// Controls the cooler via the the digital input on the hardware I/O connector.
    DigitalInput = 1,
}

impl Display for StopMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StopMode::Remote => write!(f, "Remote"),
            StopMode::DigitalInput => write!(f, "Digital Input"),
        }
    }
}

/// A rust driver for the CryoTelGt.
///
/// This driver provides functionality to control the Sunpower/CryoTelGt.
///
/// # Example
///
/// ```no_run
/// use sunpower_cryotelgt::{CryoTelGt, SerialInterfaceCryoTelGt};
///
/// // Create a serial interface and connect to the CryoTel GT
/// let interface = SerialInterfaceCryoTelGt::simple("/dev/ttyUSB0").unwrap();
/// let mut cryotel = CryoTelGt::try_new(interface).unwrap();
///
/// // Get the current temperature
/// let temperature = cryotel.get_temperature().unwrap();
///```
pub struct CryoTelGt<T: InstrumentInterface> {
    interface: Arc<Mutex<T>>,
}

impl<T: InstrumentInterface> CryoTelGt<T> {
    /// Create a new CryoTelGt instance with the given instrument interface.
    ///
    /// # Arguments
    /// * `interface` - An instrument interface that implements the [`InstrumentInterface`] trait.
    pub fn try_new(interface: T) -> Result<Self, InstrumentError> {
        let mut intf = interface;
        intf.set_terminator("\r");
        let interface = Arc::new(Mutex::new(intf));
        let instrument = CryoTelGt { interface };
        Ok(instrument)
    }

    /// Get the temperature band of the CryoTel GT in Kelvin.
    ///
    /// Returns the temperature band within which the green LED and "At temperature pin" on the I/O
    /// connector are activated.
    pub fn get_at_temperature_band(&mut self) -> Result<Temperature, InstrumentError> {
        let response = self.query("SET TBAND")?;
        let tband_k: f64 = response.trim().parse().map_err(|e| {
            InstrumentError::ResponseParseError(format!(
                "Failed to parse temperature band from response '{}': {}",
                response, e
            ))
        })?;
        Ok(Temperature::from_kelvin(tband_k))
    }

    /// Set the temperature band of the CryoTel GT.
    ///
    /// # Arguments
    /// * `tband` - The temperature band at which the green LED and "At temperature pin" on the I/O
    ///   connector are activated.
    pub fn set_at_temperature_band(&mut self, tband: Temperature) -> Result<(), InstrumentError> {
        let tband_k = tband.as_kelvin();
        let cmd = format!("SET TBAND={:.2}", tband_k);
        let _ = self.query(&cmd)?;
        Ok(())
    }

    /// Get the control mode of the CryoTel GT.
    pub fn get_control_mode(&mut self) -> Result<ControlMode, InstrumentError> {
        let response = self.query("SET PID")?;
        let mode_num: u8 = response.trim().parse().map_err(|e| {
            InstrumentError::ResponseParseError(format!(
                "Failed to parse control mode from response '{}': {}",
                response, e
            ))
        })?;
        match mode_num {
            0 => Ok(ControlMode::Power),
            2 => Ok(ControlMode::Temperature),
            _ => Err(InstrumentError::ResponseParseError(format!(
                "Unknown control mode number: {}",
                mode_num
            ))),
        }
    }

    /// Set the control mode of the CryoTel GT.
    ///
    /// Note: The control mode will be reset after a power cycle, unless you call the
    /// `save_control_mode` function.
    ///
    /// # Arguments
    /// * `mode` - The control mode to set.
    pub fn set_control_mode(&mut self, mode: ControlMode) -> Result<(), InstrumentError> {
        let cmd = format!("SET PID={}", mode as u8);
        let _ = self.query(&cmd)?;
        Ok(())
    }

    /// Get the error codes from the CryoTel GT as human readable messages.
    ///
    /// Only error codes that are currently active will be returned.
    pub fn get_errors(&mut self) -> Result<Option<Vec<String>>, InstrumentError> {
        let response = self.query("ERROR")?;
        let err_code = u8::from_str_radix(response.trim(), 2).map_err(|e| {
            InstrumentError::ResponseParseError(format!(
                "Failed to parse error code from response '{}': {}",
                response, e
            ))
        })?;

        let mut ret = Vec::new();
        if err_code & 1 != 0 {
            ret.push("Over Current".to_string());
        };
        if err_code & 2 != 0 {
            ret.push("Jumper Error".to_string());
        };
        if err_code & 4 != 0 {
            ret.push("Serial Error".to_string());
        };
        if err_code & 8 != 0 {
            ret.push("Non-volatile Memory Error".to_string());
        };
        if err_code & 16 != 0 {
            ret.push("Watchdog Error".to_string());
        };
        if err_code & 32 != 0 {
            ret.push("Temperature Sensor Error".to_string());
        };

        match ret.is_empty() {
            true => Ok(None),
            false => Ok(Some(ret)),
        }
    }

    /// Get KI, the integral constant of the temperature control loop.
    ///
    /// The default value of the KI parameter is 1.0, see `reset_ki`.
    pub fn get_ki(&mut self) -> Result<f64, InstrumentError> {
        let response = self.query("SET KI")?;
        let ki: f64 = response.trim().parse().map_err(|e| {
            InstrumentError::ResponseParseError(format!(
                "Failed to parse KI from response '{}': {}",
                response, e
            ))
        })?;
        Ok(ki)
    }

    /// Set KI, the integral constant of the temperature control loop.
    ///
    /// The default value of the KI parameter is 1.0, see `reset_ki`.
    ///
    /// # Arguments
    /// * `ki` - The KI value to set.
    pub fn set_ki(&mut self, ki: f64) -> Result<(), InstrumentError> {
        let cmd = format!("SET KI={:.5}", ki);
        let _ = self.query(&cmd)?;
        Ok(())
    }

    /// Reset KI to its default value of 1.0.
    pub fn reset_ki(&mut self) -> Result<(), InstrumentError> {
        self.set_ki(1.0)
    }

    /// Get KP, the proportional constant of the temperature control loop.
    ///
    /// The default value of the KP parameter is 50.0, see `reset_kp`.
    pub fn get_kp(&mut self) -> Result<f64, InstrumentError> {
        let response = self.query("SET KP")?;
        let kp: f64 = response.trim().parse().map_err(|e| {
            InstrumentError::ResponseParseError(format!(
                "Failed to parse KP from response '{}': {}",
                response, e
            ))
        })?;
        Ok(kp)
    }

    /// Set KP, the proportional constant of the temperature control loop.
    ///
    /// The default value of the KP parameter is 50.0, see `reset_kp`.
    ///
    /// # Arguments
    /// * `kp` - The KP value to set.
    pub fn set_kp(&mut self, kp: f64) -> Result<(), InstrumentError> {
        let cmd = format!("SET KP={:.5}", kp);
        let _ = self.query(&cmd)?;
        Ok(())
    }

    /// Reset KP to its default value of 50.0.
    pub fn reset_kp(&mut self) -> Result<(), InstrumentError> {
        self.set_kp(50.0)
    }

    /// Get the current power of the CryoTel GT.
    pub fn get_power(&mut self) -> Result<Power, InstrumentError> {
        let response = self.query("P")?;
        let pow_val = response.trim().parse::<f64>().map_err(|e| {
            InstrumentError::ResponseParseError(format!(
                "Failed to parse power from response '{}': {}",
                response, e
            ))
        })?;

        Ok(Power::from_watts(pow_val))
    }

    /// Get the current power limits and current power of the CryoTel GT.
    ///
    /// Returns a tuple of (max_power, min_power, current_power). The limits here represent the
    /// maximum and minimum allowable power for the current temperature. The current power
    /// represent the currently commanded power.
    pub fn get_power_limits_current(&mut self) -> Result<(Power, Power, Power), InstrumentError> {
        let responses = self.query_multiline("E", 3)?;
        let max_power_w: f64 = responses[0].trim().parse().map_err(|e| {
            InstrumentError::ResponseParseError(format!(
                "Failed to parse max power from response '{}': {}",
                responses[0], e
            ))
        })?;
        let min_power_w: f64 = responses[1].trim().parse().map_err(|e| {
            InstrumentError::ResponseParseError(format!(
                "Failed to parse min power from response '{}': {}",
                responses[1], e
            ))
        })?;
        let current_power_w: f64 = responses[2].trim().parse().map_err(|e| {
            InstrumentError::ResponseParseError(format!(
                "Failed to parse current power from response '{}': {}",
                responses[2], e
            ))
        })?;
        Ok((
            Power::from_watts(max_power_w),
            Power::from_watts(min_power_w),
            Power::from_watts(current_power_w),
        ))
    }

    /// Get the user-set maximum power of the CryoTel GT.
    pub fn get_power_max(&mut self) -> Result<Power, InstrumentError> {
        let response = self.query("SET MAX")?;
        let max_power_w: f64 = response.trim().parse().map_err(|e| {
            InstrumentError::ResponseParseError(format!(
                "Failed to parse max user power from response '{}': {}",
                response, e
            ))
        })?;
        Ok(Power::from_watts(max_power_w))
    }

    /// Set the user-set maximum power of the CryoTel GT.
    ///
    /// If this exceeds the maximum power allowed for the current temperature, the CryoTel GT will
    /// limit its output power automatically to the maximum safe power.
    /// # Arguments
    /// * `max_power` - The maximum power to set.
    pub fn set_power_max(&mut self, max_power: Power) -> Result<(), InstrumentError> {
        let max_power_w = max_power.as_watts();
        let cmd = format!("SET MAX={:.2}", max_power_w);
        let _ = self.query(&cmd)?;
        Ok(())
    }

    /// Get the user-set minimum power of the CryoTel GT.
    pub fn get_power_min(&mut self) -> Result<Power, InstrumentError> {
        let response = self.query("SET MIN")?;
        let min_power_w: f64 = response.trim().parse().map_err(|e| {
            InstrumentError::ResponseParseError(format!(
                "Failed to parse min user power from response '{}': {}",
                response, e
            ))
        })?;
        Ok(Power::from_watts(min_power_w))
    }

    /// Set the user-set minimum power of the CryoTel GT.
    ///
    /// If this is below the minimum power allowed for the current temperature, the CryoTel GT will
    /// limit its output power automatically to the minimum safe power.
    ///
    /// # Arguments
    /// * `min_power` - The minimum power to set.
    pub fn set_power_min(&mut self, min_power: Power) -> Result<(), InstrumentError> {
        let min_power_w = min_power.as_watts();
        let cmd = format!("SET MIN={:.2}", min_power_w);
        let _ = self.query(&cmd)?;
        Ok(())
    }

    /// Get the setpoint power of the CryoTel GT.
    ///
    /// This value is only relevant when the control mode is set to `ControlMode::Power`.
    pub fn get_power_setpoint(&mut self) -> Result<Power, InstrumentError> {
        let response = self.query("SET PWOUT")?;
        let setpoint_power_w: f64 = response.trim().parse().map_err(|e| {
            InstrumentError::ResponseParseError(format!(
                "Failed to parse power setpoint from response '{}': {}",
                response, e
            ))
        })?;
        Ok(Power::from_watts(setpoint_power_w))
    }

    /// Set the setpoint power of the CryoTel GT.
    ///
    /// This value is only relevant when the control mode is set to `ControlMode::Power`. The
    /// CryoTel GT will only command a power that will not damage the cryocooler.
    ///
    /// # Arguments
    /// * `setpoint_power` - The power setpoint to set.
    pub fn set_power_setpoint(&mut self, setpoint_power: Power) -> Result<(), InstrumentError> {
        let setpoint_power_w = setpoint_power.as_watts();
        let cmd = format!("SET PWOUT={:.2}", setpoint_power_w);
        let _ = self.query(&cmd)?;
        Ok(())
    }

    /// Reset the CryoTel GT to factory settings.
    pub fn reset_to_factory_settings(&mut self) -> Result<(), InstrumentError> {
        let _ = self.query_multiline("RESET=F", 2)?;
        Ok(())
    }

    /// Save the current control mode to non-volatile memory.
    pub fn save_control_mode(&mut self) -> Result<(), InstrumentError> {
        let _ = self.query("SAVE PID")?;
        Ok(())
    }

    /// Query the serial number of the instrument as a vector of two strings.
    pub fn get_serial_number(&mut self) -> Result<Vec<String>, InstrumentError> {
        self.query_multiline("SERIAL", 2)
    }

    /// Get the cooler state of the CryoTel GT.
    ///
    /// This returns if the cooler is enabled or disabled based on the soft stop command.
    pub fn get_state(&mut self) -> Result<CoolerState, InstrumentError> {
        let response = self.query("SET SSTOP")?;
        let state_num: f64 = response.trim().parse().map_err(|e| {
            InstrumentError::ResponseParseError(format!(
                "Failed to parse cooler state from response '{}': {}",
                response, e
            ))
        })?;
        match state_num {
            0.0 => Ok(CoolerState::Enabled),
            1.0 => Ok(CoolerState::Disabled),
            _ => Err(InstrumentError::ResponseParseError(format!(
                "Unknown cooler state number: {}",
                state_num
            ))),
        }
    }

    /// Set the cooler state of the CryoTel GT.
    ///
    /// This function enables or diables the cooler based on the soft stop command.
    ///
    /// # Arguments
    /// * `state` - The cooler state to set.
    pub fn set_state(&mut self, state: CoolerState) -> Result<(), InstrumentError> {
        let state_num = match state {
            CoolerState::Enabled => 0.0,
            CoolerState::Disabled => 1.0,
        };
        let cmd = format!("SET SSTOP={:.2}", state_num);
        let _ = self.query(&cmd)?;
        Ok(())
    }

    /// Get the full state of the CryoTel GT as a vector of strings.
    pub fn get_full_state(&mut self) -> Result<Vec<String>, InstrumentError> {
        self.query_multiline("STATE", 14)
    }

    /// Get the stop mode of the CryoTel GT.
    ///
    /// Returns the current stop mode of the cooler.
    pub fn get_stop_mode(&mut self) -> Result<StopMode, InstrumentError> {
        let response = self.query("SET SSTOPM")?;
        let mode_num: f64 = response.trim().parse().map_err(|e| {
            InstrumentError::ResponseParseError(format!(
                "Failed to parse stop mode from response '{}': {}",
                response, e
            ))
        })?;
        match mode_num {
            0.0 => Ok(StopMode::Remote),
            1.0 => Ok(StopMode::DigitalInput),
            _ => Err(InstrumentError::ResponseParseError(format!(
                "Unknown stop mode number: {}",
                mode_num
            ))),
        }
    }

    /// Get the current temperature of the CryoTel GT.
    pub fn get_temperature(&mut self) -> Result<Temperature, InstrumentError> {
        let response = self.query("TC")?;
        let temp_k: f64 = response.trim().parse().map_err(|e| {
            InstrumentError::ResponseParseError(format!(
                "Failed to parse temperature from response '{}': {}",
                response, e
            ))
        })?;
        Ok(Temperature::from_kelvin(temp_k))
    }

    /// Get the temperature setpoint of the CryoTel GT.
    ///
    /// This value is only relevant when the control mode is set to `ControlMode::Temperature`.
    pub fn get_temperature_setpoint(&mut self) -> Result<Temperature, InstrumentError> {
        let response = self.query("SET TTARGET")?;
        let temp_k: f64 = response.trim().parse().map_err(|e| {
            InstrumentError::ResponseParseError(format!(
                "Failed to parse temperature setpoint from response '{}': {}",
                response, e
            ))
        })?;
        Ok(Temperature::from_kelvin(temp_k))
    }

    /// Set the temperature setpoint of the CryoTel GT.
    ///
    /// This value is only relevant when the control mode is set to `ControlMode::Temperature`.
    ///
    /// # Arguments
    /// * `setpoint_temp` - The temperature setpoint to set.
    pub fn set_temperature_setpoint(
        &mut self,
        setpoint_temp: Temperature,
    ) -> Result<(), InstrumentError> {
        let setpoint_temp_k = setpoint_temp.as_kelvin();
        let cmd = format!("SET TTARGET={:.2}", setpoint_temp_k);
        let _ = self.query(&cmd)?;
        Ok(())
    }

    /// Get the status of the CryoTel GT cooler when in thermostat mode.
    ///
    /// Returns whether the cooler is enabled or disabled based on the external thermostat.
    /// This functionality is only relevant when the thermostat mode is set to `ThermostatMode::Enabled`.
    pub fn get_thermostat_status(&mut self) -> Result<CoolerState, InstrumentError> {
        let response = self.query("TSTAT")?;
        let status_num: u8 = response.trim().parse().map_err(|e| {
            InstrumentError::ResponseParseError(format!(
                "Failed to parse thermostat status from response '{}': {}",
                response, e
            ))
        })?;
        match status_num {
            0 => Ok(CoolerState::Disabled),
            1 => Ok(CoolerState::Enabled),
            _ => Err(InstrumentError::ResponseParseError(format!(
                "Unknown thermostat status number: {}",
                status_num
            ))),
        }
    }

    /// Get the thermostat mode of the CryoTel GT.
    ///
    /// Returns whether the thermostat functionality is enabled or disabled.
    pub fn get_thermostat_mode(&mut self) -> Result<ThermostatMode, InstrumentError> {
        let response = self.query("SET TSTATM")?;
        let mode_num: f64 = response.trim().parse().map_err(|e| {
            InstrumentError::ResponseParseError(format!(
                "Failed to parse thermostat mode from response '{}': {}",
                response, e
            ))
        })?;
        match mode_num {
            0.0 => Ok(ThermostatMode::Disabled),
            1.0 => Ok(ThermostatMode::Enabled),
            _ => Err(InstrumentError::ResponseParseError(format!(
                "Unknown thermostat mode number: {}",
                mode_num
            ))),
        }
    }

    /// Set the thermostat mode of the CryoTel GT.
    ///
    /// The thermostat functionality allows the user to add a thermostat to the system which can be
    /// used to shut down the cryocooler. See the manual for more information.
    ///
    /// # Arguments
    /// * `mode` - The thermostat mode to set.
    pub fn set_thermostat_mode(&mut self, mode: ThermostatMode) -> Result<(), InstrumentError> {
        let cmd = format!("SET TSTATM={:.2}", mode as u8 as f64);
        println!("cmd: {}", cmd);
        let _ = self.query(&cmd)?;
        Ok(())
    }

    /// Set the stop mode of the CryoTel GT.
    ///
    /// # Arguments
    /// * `mode` - The stop mode to set.
    pub fn set_stop_mode(&mut self, mode: StopMode) -> Result<(), InstrumentError> {
        let cmd = format!("SET SSTOPM={:.2}", mode as u8 as f64);
        let _ = self.query(&cmd)?;
        Ok(())
    }

    /// Send a command to the instrument.
    fn sendcmd(&mut self, cmd: &str) -> Result<(), InstrumentError> {
        let mut intf = self.interface.lock().expect("Mutex should not be poisoned");
        intf.sendcmd(cmd)?;
        intf.check_acknowledgment(cmd)
    }

    fn query(&mut self, cmd: &str) -> Result<String, InstrumentError> {
        self.sendcmd(cmd)?;
        let mut intf = self.interface.lock().expect("Mutex should not be poisoned");
        intf.read_until_terminator()
    }

    fn query_multiline(
        &mut self,
        cmd: &str,
        nlines: usize,
    ) -> Result<Vec<String>, InstrumentError> {
        self.sendcmd(cmd)?;
        let mut intf = self.interface.lock().expect("Mutex should not be poisoned");
        let mut responses = Vec::with_capacity(nlines);
        for _ in 0..nlines {
            let line = intf.read_until_terminator()?;
            responses.push(line);
        }
        Ok(responses)
    }
}

impl<T: InstrumentInterface> Clone for CryoTelGt<T> {
    fn clone(&self) -> Self {
        Self {
            interface: self.interface.clone(),
        }
    }
}
