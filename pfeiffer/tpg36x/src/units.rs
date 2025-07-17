//! Module to handle instrument specific units and conversions.

use measurements::{Pressure, Voltage};

/// Since the TPG36x can return either a pressure or a voltage measurement, we return an enum for
/// the measurements with unitful values that can contain either pressure or voltage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tpg36xMeasurement {
    /// Measurement in pressure units.
    Pressure(Pressure),
    /// Measurement in voltage units.
    Voltage(Voltage),
}

/// All the units the TPG36x can be configured to use.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum PressureUnit {
    /// Millibar
    #[allow(non_camel_case_types)] // could stand for Mega otherwise
    mBar,
    /// Torr
    Torr,
    /// Pascal
    Pa,
    /// Millitorr (or micron)
    #[allow(non_camel_case_types)] // could stand for Mega otherwise
    mTorr,
    /// Hectopascal
    #[allow(non_camel_case_types)] // to be consistent with the others
    #[default]
    hPa,
    /// Volt
    V,
}

impl PressureUnit {
    /// Convert pressure units of instrument to a string that can be used in commands.
    pub(crate) fn as_str(&self) -> &str {
        match self {
            PressureUnit::mBar => "0",
            PressureUnit::Torr => "1",
            PressureUnit::Pa => "2",
            PressureUnit::mTorr => "3",
            PressureUnit::hPa => "4",
            PressureUnit::V => "5",
        }
    }

    /// Convert pressure unit string from instrument to a `PressureUnit`.
    pub(crate) fn from_cmd_str(value: &str) -> Result<Self, instrumentrs::InstrumentError> {
        match value.trim() {
            "0" => Ok(PressureUnit::mBar),
            "1" => Ok(PressureUnit::Torr),
            "2" => Ok(PressureUnit::Pa),
            "3" => Ok(PressureUnit::mTorr),
            "4" => Ok(PressureUnit::hPa),
            "5" => Ok(PressureUnit::V),
            _ => Err(instrumentrs::InstrumentError::ResponseParseError(
                value.to_string(),
            )),
        }
    }
}

/// Convert a value and instrument unit into a `Tpg36xMeasurement`.
pub(crate) fn from_value_unit(value: f64, unit: &PressureUnit) -> Tpg36xMeasurement {
    match unit {
        PressureUnit::mBar => Tpg36xMeasurement::Pressure(Pressure::from_millibars(value)),
        PressureUnit::Torr => {
            Tpg36xMeasurement::Pressure(Pressure::from_pascals(value * 133.32236842))
        } // HACK
        PressureUnit::Pa => Tpg36xMeasurement::Pressure(Pressure::from_pascals(value)),
        PressureUnit::mTorr => {
            Tpg36xMeasurement::Pressure(Pressure::from_pascals(value * 0.13332236842))
        } // HACK
        PressureUnit::hPa => Tpg36xMeasurement::Pressure(Pressure::from_pascals(value * 100.0)),
        PressureUnit::V => Tpg36xMeasurement::Voltage(Voltage::from_volts(value)),
    }
}
