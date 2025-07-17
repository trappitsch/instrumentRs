//! Module that contains various status codes and their meanings.

use std::fmt::Display;

/// Status codes for the pressure measurement data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PressMsrDatStat {
    Ok = 0,
    Underrange = 1,
    Overrange = 2,
    SensorError = 3,
    SensorOff = 4,
    MeasurementError = 5,
    IdentificationError = 6,
    UnknownError = 7, // not an actual status code, but used for parsing
}

impl PressMsrDatStat {
    pub(crate) fn from_cmd_str(value: &str) -> Result<Self, instrumentrs::InstrumentError> {
        match value.trim() {
            "0" => Ok(PressMsrDatStat::Ok),
            "1" => Ok(PressMsrDatStat::Underrange),
            "2" => Ok(PressMsrDatStat::Overrange),
            "3" => Ok(PressMsrDatStat::SensorError),
            "4" => Ok(PressMsrDatStat::SensorOff),
            "5" => Ok(PressMsrDatStat::MeasurementError),
            "6" => Ok(PressMsrDatStat::IdentificationError),
            _ => Ok(PressMsrDatStat::UnknownError),
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
            PressMsrDatStat::MeasurementError => "Measurement Error",
            PressMsrDatStat::IdentificationError => "Identification Error",
            PressMsrDatStat::UnknownError => "Unknown Error",
        };
        write!(f, "{description}")
    }
}
