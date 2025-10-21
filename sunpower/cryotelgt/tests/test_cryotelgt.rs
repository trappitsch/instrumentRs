//! Tests for the Sunpower CryoTelGt driver.

use measurements::{Power, Temperature};
use rstest::*;

use instrumentrs::LoopbackInterfaceString;

use sunpower_cryotelgt::*;

// Type alias for the loopback interface with the CryoTelGt driver.
type CryoTelGtLbk = CryoTelGt<LoopbackInterfaceString>;

/// Function that creates a new CryoTelGt instance with the given input
/// and output commands.
fn crt_inst(host2inst: Vec<&str>, inst2host: Vec<&str>) -> CryoTelGtLbk {
    let term = "\r";
    let h2i: Vec<String> = host2inst.iter().map(|s| s.to_string()).collect();
    let i2h: Vec<String> = inst2host.iter().map(|s| s.to_string()).collect();
    let interface = LoopbackInterfaceString::new(h2i, i2h, term);
    CryoTelGt::try_new(interface).unwrap()
}

#[fixture]
fn emp_inst() -> CryoTelGtLbk {
    crt_inst(vec![], vec![])
}

/// This test initializes the instruments with empty vectors, which should always pass.
#[rstest]
fn test_initialization(_emp_inst: CryoTelGtLbk) {}

/// Temperature band set/get.
#[rstest]
fn test_at_temperature_band() {
    let mut inst = crt_inst(
        vec!["SET TBAND", "SET TBAND=0.07", "SET TBAND"],
        vec![
            "SET TBAND",
            "0.500",
            "SET TBAND=0.07",
            "0.07",
            "SET TBAND",
            "0.07",
        ],
    );

    let tband_exp = Temperature::from_kelvin(0.5);
    let tband = inst.get_at_temperature_band().unwrap();
    assert_eq!(tband, tband_exp);

    let tband_exp = Temperature::from_kelvin(0.07);
    inst.set_at_temperature_band(tband_exp).unwrap();
    let tband = inst.get_at_temperature_band().unwrap();
    assert_eq!(tband, tband_exp);
}

/// Control mode set/get.
#[rstest]
fn test_control_mode() {
    let mut inst = crt_inst(
        vec!["SET PID=0", "SET PID=2", "SET PID"],
        vec!["SET PID=0", "0", "SET PID=2", "2", "SET PID", "2"],
    );

    inst.set_control_mode(ControlMode::Power).unwrap();
    inst.set_control_mode(ControlMode::Temperature).unwrap();
    let mode = inst.get_control_mode().unwrap();
    assert_eq!(mode, ControlMode::Temperature);
}

/// Get error codes as human readable strings.
#[rstest]
fn test_get_error_codes() {
    let mut inst = crt_inst(
        vec!["ERROR", "ERROR", "ERROR"],
        vec!["ERROR", "100000", "ERROR", "000000", "ERROR", "011001"],
    );

    let errors_exp = Some(vec!["Temperature Sensor Error".to_string()]);
    let errors = inst.get_errors().unwrap();
    assert_eq!(errors, errors_exp);

    let errors_exp = None;
    let errors = inst.get_errors().unwrap();
    assert_eq!(errors, errors_exp);

    let errors_exp = Some(vec![
        "Over Current".to_string(),
        "Non-volatile Memory Error".to_string(),
        "Watchdog Error".to_string(),
    ]);
    let errors = inst.get_errors().unwrap();
    assert_eq!(errors, errors_exp);
}

/// KI property, get/set.
#[rstest]
fn test_ki_property() {
    let mut inst = crt_inst(
        vec!["SET KI=0.10000", "SET KI", "SET KI=1.00000"],
        vec![
            "SET KI=0.10000",
            "0.10",
            "SET KI",
            "0.10",
            "SET KI=1.00000",
            "1.0",
        ],
    );

    let ki_exp = 0.10;
    inst.set_ki(ki_exp).unwrap();
    let ki = inst.get_ki().unwrap();
    assert_eq!(ki, ki_exp);

    inst.reset_ki().unwrap();
}

// KP property, get/set.
#[rstest]
fn test_kp_property() {
    let mut inst = crt_inst(
        vec!["SET KP=0.20000", "SET KP", "SET KP=50.00000"],
        vec![
            "SET KP=0.20000",
            "0.20",
            "SET KP",
            "0.20",
            "SET KP=50.00000",
            "2.0",
        ],
    );

    let kp_exp = 0.20;
    inst.set_kp(kp_exp).unwrap();
    let kp = inst.get_kp().unwrap();
    assert_eq!(kp, kp_exp);

    inst.reset_kp().unwrap();
}

/// Get the current power of the instrument.
#[rstest]
fn test_get_power() {
    let mut inst = crt_inst(vec!["P"], vec!["P", "75.0"]);

    let power_exp = Power::from_watts(75.0);
    let power = inst.get_power().unwrap();
    assert_eq!(power, power_exp);
}

/// Get current power and power limits (min, max).
#[rstest]
fn test_get_power_limits() {
    let mut inst = crt_inst(vec!["E"], vec!["E", "230.00", "070.00", "170.00"]);

    let p_max_exp = Power::from_watts(230.0);
    let p_min_exp = Power::from_watts(70.0);
    let p_curr_exp = Power::from_watts(170.0);

    let (p_max, p_min, p_curr) = inst.get_power_limits_current().unwrap();
    assert_eq!(p_max, p_max_exp);
    assert_eq!(p_min, p_min_exp);
    assert_eq!(p_curr, p_curr_exp);
}

/// Max user power, get/set.
#[rstest]
fn test_power_max() {
    let mut inst = crt_inst(
        vec!["SET MAX=100.00", "SET MAX"],
        vec!["SET MAX=100.00", "100.00", "SET MAX", "100.00"],
    );

    let p_max_exp = Power::from_watts(100.0);
    inst.set_power_max(p_max_exp).unwrap();
    let p_max = inst.get_power_max().unwrap();
    assert_eq!(p_max, p_max_exp);
}

/// Min user power, get/set.
#[rstest]
fn test_power_min() {
    let mut inst = crt_inst(
        vec!["SET MIN=20.00", "SET MIN"],
        vec!["SET MIN=20.00", "20.00", "SET MIN", "20.00"],
    );

    let p_min_exp = Power::from_watts(20.0);
    inst.set_power_min(p_min_exp).unwrap();
    let p_min = inst.get_power_min().unwrap();
    assert_eq!(p_min, p_min_exp);
}

/// Power setpoint, get/set.
#[rstest]
fn test_power_setpoint() {
    let mut inst = crt_inst(
        vec!["SET PWOUT=75.00", "SET PWOUT"],
        vec!["SET PWOUT=75.00", "75.00", "SET PWOUT", "75.00"],
    );

    let p_set_exp = Power::from_watts(75.0);
    inst.set_power_setpoint(p_set_exp).unwrap();
    let p_set = inst.get_power_setpoint().unwrap();
    assert_eq!(p_set, p_set_exp);
}

/// Reset the cooler to factory settings.
#[rstest]
fn test_reset_factory() {
    let mut inst = crt_inst(
        vec!["RESET=F"],
        vec![
            "RESET=F",
            "RESETTING TO FACTORY DEFAULT...",
            "FACTORY RESET COMPLETE!",
        ],
    );
    inst.reset_to_factory_settings().unwrap();
}

/// Read back the serial number of the instrument.
#[rstest]
fn test_get_serial_number() {
    let mut inst = crt_inst(vec!["SERIAL"], vec!["SERIAL", "serial", "revision"]);
    let serial = inst.get_serial_number().unwrap();

    let serial_exp = Vec::from(["serial".to_string(), "revision".to_string()]);
    assert_eq!(serial, serial_exp);
}

/// Save the current control mode to non-volatile memory.
#[rstest]
fn test_save_control_mode() {
    let mut inst = crt_inst(vec!["SAVE PID"], vec!["SAVE PID", "2.00"]);
    inst.save_control_mode().unwrap();
}

/// Cooler state, get/set.
#[rstest]
#[case("0.00", CoolerState::Enabled)]
#[case("1.00", CoolerState::Disabled)]
fn test_cooler_state(#[case] state_str: &str, #[case] state_enum: CoolerState) {
    let cmd = format!("SET SSTOP={}", state_str);
    let mut inst = crt_inst(
        vec![&cmd, "SET SSTOP"],
        vec![&cmd, state_str, "SET SSTOP", state_str],
    );

    inst.set_state(state_enum.clone()).unwrap();
    let state = inst.get_state().unwrap();
    assert_eq!(state, state_enum);
}

/// Get the full state of the CryoTel GT.
#[rstest]
fn test_full_state() {
    let state_exp: Vec<String> = Vec::from([
        "MODE = 002.00".to_string(),
        "TSTATM = 000.00".to_string(),
        "TSTAT = 000.00".to_string(),
        "SSTOPM = 000.00".to_string(),
        "SSTOP = 000.00".to_string(),
        "PID = 002.00".to_string(),
        "LOCK = 000.00".to_string(),
        "MAX = 300.00".to_string(),
        "MIN = 000.00".to_string(),
        "PWOUT = 000.00".to_string(),
        "TTARGET = 000.00".to_string(),
        "TBAND = 000.50".to_string(),
        "KI = 000.50".to_string(),
        "KP = 050.00000".to_string(),
    ]);

    let mut inst2host = vec!["STATE"];
    inst2host.extend(state_exp.iter().map(String::as_str));
    let mut inst = crt_inst(vec!["STATE"], inst2host);

    let state = inst.get_full_state().unwrap();
    assert_eq!(state, state_exp);
}

/// StopMode get/set.
#[rstest]
#[case("1.00", StopMode::DigitalInput)]
#[case("0.00", StopMode::Remote)]
fn test_stop_mode(#[case] mode_str: &str, #[case] mode_enum: StopMode) {
    let cmd = format!("SET SSTOPM={}", mode_str);
    let mut inst = crt_inst(
        vec![&cmd, "SET SSTOPM"],
        vec![&cmd, mode_str, "SET SSTOPM", mode_str],
    );

    inst.set_stop_mode(mode_enum.clone()).unwrap();
    let mode = inst.get_stop_mode().unwrap();
    assert_eq!(mode, mode_enum);
}

/// Get the current temperature
#[rstest]
fn test_get_temperature() {
    let mut inst = crt_inst(vec!["TC"], vec!["TC", "120.00"]);
    let temp_exp = Temperature::from_kelvin(120.0);
    let temp = inst.get_temperature().unwrap();
    assert_eq!(temp, temp_exp);
}

/// Temperature setpoint, get/set.
#[rstest]
fn test_temperature_setpoint() {
    let mut inst = crt_inst(
        vec!["SET TTARGET=100.00", "SET TTARGET"],
        vec!["SET TTARGET=100.00", "100.00", "SET TTARGET", "100.00"],
    );

    let t_set_exp = Temperature::from_kelvin(100.0);
    inst.set_temperature_setpoint(t_set_exp).unwrap();
    let t_set = inst.get_temperature_setpoint().unwrap();
    assert_eq!(t_set, t_set_exp);
}

/// Get the cryo cooler status when in thermostat mode.
#[rstest]
fn test_thermostat_status() {
    let mut inst = crt_inst(vec!["TSTAT", "TSTAT"], vec!["TSTAT", "1", "TSTAT", "0"]);

    let status = inst.get_thermostat_status().unwrap();
    assert_eq!(status, CoolerState::Enabled);

    let status = inst.get_thermostat_status().unwrap();
    assert_eq!(status, CoolerState::Disabled);
}

/// Thermostat mode, get/set.
#[rstest]
#[case("1.00", ThermostatMode::Enabled)]
#[case("0.00", ThermostatMode::Disabled)]
fn test_thermostat_mode(#[case] mode_str: &str, #[case] mode_enum: ThermostatMode) {
    let cmd = format!("SET TSTATM={}", mode_str);
    let mut inst = crt_inst(
        vec![&cmd, "SET TSTATM"],
        vec![&cmd, mode_str, "SET TSTATM", mode_str],
    );

    inst.set_thermostat_mode(mode_enum.clone()).unwrap();
    let mode = inst.get_thermostat_mode().unwrap();
    assert_eq!(mode, mode_enum);
}
