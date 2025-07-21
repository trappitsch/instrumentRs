//! Module that contains various status codes and their meanings.

use std::fmt::Display;

use instrumentrs::InstrumentError;

/// Status codes for the pressure measurement data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PressMsrDatStat {
    Ok = 0,
    Underrange = 1,
    Overrange = 2,
    SensorError = 3,
    SensorOff = 4,
    NoSensor = 5,
    IdentificationError = 6,
}

impl PressMsrDatStat {
    pub(crate) fn from_cmd_str(value: &str) -> Result<Self, instrumentrs::InstrumentError> {
        match value.trim() {
            "0" => Ok(PressMsrDatStat::Ok),
            "1" => Ok(PressMsrDatStat::Underrange),
            "2" => Ok(PressMsrDatStat::Overrange),
            "3" => Ok(PressMsrDatStat::SensorError),
            "4" => Ok(PressMsrDatStat::SensorOff),
            "5" => Ok(PressMsrDatStat::NoSensor),
            "6" => Ok(PressMsrDatStat::IdentificationError),
            _ => Err(InstrumentError::ResponseParseError(value.to_string())),
        }
    }
}

impl Display for PressMsrDatStat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let description = match self {
            PressMsrDatStat::Ok => "OK",
            PressMsrDatStat::Underrange => "Underrange",
            PressMsrDatStat::Overrange => "Overrange",
            PressMsrDatStat::SensorError => "Sensor Error",
            PressMsrDatStat::SensorOff => "Sensor Off",
            PressMsrDatStat::NoSensor => "No Sensor",
            PressMsrDatStat::IdentificationError => "Identification Error",
        };
        write!(f, "{description}")
    }
}

/// Status that can be sent to the an individual sensor to change its state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SensorStatus {
    /// Set: leave the sensor in its current state / Get: Sensor cannot be changed.
    NoChange,
    /// Set: turn the sensor off / Get: Sensor is off.
    Off,
    /// Set: turn the sensor on / Get: Sensor is on.
    On,
}

impl SensorStatus {
    pub(crate) fn from_cmd_str(value: &str) -> Result<Self, instrumentrs::InstrumentError> {
        match value.trim() {
            "0" => Ok(SensorStatus::NoChange),
            "1" => Ok(SensorStatus::Off),
            "2" => Ok(SensorStatus::On),
            _ => Err(InstrumentError::ResponseParseError(value.to_string())),
        }
    }

    pub(crate) fn to_cmd_str(&self) -> String {
        match self {
            SensorStatus::NoChange => "0".to_string(),
            SensorStatus::Off => "1".to_string(),
            SensorStatus::On => "2".to_string(),
        }
    }
}

impl Display for SensorStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let description = match self {
            SensorStatus::NoChange => "Set: No change / Get: Sensor cannot be changed",
            SensorStatus::Off => "Off",
            SensorStatus::On => "On",
        };
        write!(f, "{description}")
    }
}
