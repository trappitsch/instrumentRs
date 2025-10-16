//! Tests for the Pfeiffer Omnicontrol driver.

use measurements::{test_utils::assert_almost_eq, Pressure};
use rstest::*;

use instrumentrs::LoopbackInterfaceString;

use pfeiffer_omnicontrol::*;

// Type alias for the loopback interface with the Omnicontrol driver.
type OmnicontrolLbk = Omnicontrol<LoopbackInterfaceString>;

/// Function that creates a new Omnicontrol instance with the given input
/// and output commands.
///
/// Check sum is calculated in here as well, so does not have to be added to the input.
fn crt_inst(host2inst: Vec<&str>, inst2host: Vec<&str>) -> OmnicontrolLbk {
    let term = "\r";
    let h2i: Vec<String> = host2inst.iter().map(|s| add_checksum(s)).collect();
    let i2h: Vec<String> = inst2host.iter().map(|s| add_checksum(s)).collect();
    let interface = LoopbackInterfaceString::new(h2i, i2h, term);
    Omnicontrol::new(interface, BaseAddress::OneHundred)
}

/// Take a command, add the checksum, and return the full command string.
///
/// Checksum is calculated as the sum of ASCII values from start (address) to end of data field,
/// modulo 256.
fn add_checksum(cmd: &str) -> String {
    let checksum = cmd.bytes().fold(0u8, |acc, b| acc.wrapping_add(b));
    format!("{}{:03}", cmd, checksum)
}

#[fixture]
fn emp_inst() -> OmnicontrolLbk {
    crt_inst(vec![], vec![])
}

/// This test initializes the instruments with empty vectors, which should always pass.
#[rstest]
fn test_initialization(_emp_inst: OmnicontrolLbk) {}

/// Test to get the name of the Omnicontrol.
#[rstest]
fn test_get_name() {
    let mut inst = crt_inst(vec!["1010034902=?"], vec!["1011034904Omni"]);
    let name = inst.get_name().unwrap();
    assert_eq!(name, "Omni");
}

/// Test getting the pressure from various channels.
#[rstest]
#[case(0)]
#[case(1)]
#[case(2)]
#[case(3)]
fn test_channel_get_pressure(#[case] idx: usize) {
    let mut inst = crt_inst(
        vec![&format!("1{}20074002=?", idx+1)],
        vec![&format!("1{}20074006100023", idx+1)], // pressure: 1.0e3 hPa
    );
    let pressure_exp = Pressure::from_hectopascals(1.0e3);
    let mut ch = inst.get_channel(idx).unwrap();
    let pressure = ch.get_pressure().unwrap();

    assert_almost_eq(pressure.as_hectopascals(), pressure_exp.as_hectopascals());
}
