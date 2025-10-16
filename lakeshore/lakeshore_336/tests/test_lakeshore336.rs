//! Tests for the Lakeshore Lakeshore336 driver.

use rstest::*;

use instrumentrs::LoopbackInterfaceString;

use lakeshore_336::*;

// Type alias for the loopback interface with the Lakeshore336 driver.
type Lakeshore336Lbk = Lakeshore336<LoopbackInterfaceString>;

/// Function that creates a new Lakeshore336 instance with the given input
/// and output commands.
fn crt_inst(host2inst: Vec<&str>, inst2host: Vec<&str>) -> Lakeshore336Lbk {
    let term = "\n";
    let h2i: Vec<String> = host2inst.iter().map(|s| s.to_string()).collect();
    let i2h: Vec<String> = inst2host.iter().map(|s| s.to_string()).collect();
    let interface = LoopbackInterfaceString::new(h2i, i2h, term);
    Lakeshore336::try_new(interface).unwrap()
}

#[fixture]
fn emp_inst() -> Lakeshore336Lbk {
    crt_inst(vec![], vec![])
}

/// Empty initialization should always pass.
#[rstest]
fn test_initialization(_emp_inst: Lakeshore336Lbk) {}

/// Get the name from the instrument.
#[rstest]
fn test_get_name() {
    let mut inst = crt_inst(vec!["*IDN?"], vec!["Lakeshore,336,12345678,1.0"]);
    let name = inst.get_name().unwrap();
    assert_eq!(name, "Lakeshore,336,12345678,1.0");
}

/// Get temperature for the four channels.
#[rstest]
#[case(0, "A")]
#[case(1, "B")]
#[case(2, "C")]
#[case(3, "D")]
fn test_channel_get_temperature(#[case] ch_num: usize, #[case] ch_id: &str) {
    let mut inst = crt_inst(vec![&format!("KRDG?{}", ch_id)], vec!["273.15"]);
    let mut ch = inst.get_channel(ch_num).unwrap();
    let temp = ch.get_temperature().unwrap();
    assert_eq!(temp.as_kelvin(), 273.15);
}

/// Return a sensor error if the reading is zero kelvin.
#[rstest]
fn test_channel_get_temperature_sensor_error() {
    let mut inst = crt_inst(vec!["KRDG?A"], vec!["0.0"]);
    let mut ch = inst.get_channel(0).unwrap();
    assert!(ch.get_temperature().is_err());
}

/// Ensure cloning an instrument and a channel works correctly.
#[rstest]
fn test_cloning(mut emp_inst: Lakeshore336Lbk) {
    let _ = emp_inst.clone();
    let ch_c = emp_inst.get_channel(2).unwrap();
    let _ = ch_c.clone();
}
