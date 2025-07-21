//! Module to handle instrument specific units and conversions.

use std::fmt::Display;

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

impl Display for Tpg36xMeasurement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tpg36xMeasurement::Pressure(p) => write!(f, "{p}"),
            Tpg36xMeasurement::Voltage(v) => write!(f, "{v}"),
        }
    }
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

impl Display for PressureUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PressureUnit::mBar => write!(f, "mBar"),
            PressureUnit::Torr => write!(f, "Torr"),
            PressureUnit::Pa => write!(f, "Pa"),
            PressureUnit::mTorr => write!(f, "mTorr"),
            PressureUnit::hPa => write!(f, "hPa"),
            PressureUnit::V => write!(f, "V"),
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

#[cfg(test)]
mod test {
    use super::*;
    use measurements::{Measurement, test_utils::almost_eq};
    use rstest::*;

    #[rstest]
    #[case(1000.0, PressureUnit::mBar, Pressure::from_millibars(1000.0))]
    #[case(1000.0, PressureUnit::Torr, Pressure::from_pascals(1000.0 * 133.3224))]
    #[case(1000.0, PressureUnit::Pa, Pressure::from_pascals(1000.0))]
    #[case(1000.0, PressureUnit::mTorr, Pressure::from_pascals(1000.0 * 133322.4))]
    #[case(1000.0, PressureUnit::hPa, Pressure::from_pascals(1000.0 * 100.0))]
    fn test_from_value_unit_pressure(
        #[case] value: f64,
        #[case] unit: PressureUnit,
        #[case] expected: Pressure,
    ) {
        let measurement = from_value_unit(value, &unit);
        if let Tpg36xMeasurement::Pressure(pressure) = measurement {
            almost_eq(expected.as_base_units(), pressure.as_base_units());
        } else {
            panic!("Expected a pressure measurement.");
        }
    }

    #[rstest]
    fn test_from_value_unit_voltage() {
        let value = 5.0;
        let unit = PressureUnit::V;
        let expected = Voltage::from_volts(value);
        let measurement = from_value_unit(value, &unit);

        if let Tpg36xMeasurement::Voltage(voltage) = measurement {
            almost_eq(expected.as_base_units(), voltage.as_base_units());
        } else {
            panic!("Expected a voltage measurement.");
        }
    }
}
