//! Tests for the Agilent Agilent4Uhv driver.

use measurements::test_utils::assert_almost_eq;
use rstest::*;

use instrumentrs::LoopbackInterfaceBytes;

use agilent_4uhv::*;

const STX: u8 = 0x02;
const ADDR: u8 = 0x80; // Default address for most tests
const READ: u8 = 0x30;
const WRT: u8 = 0x31;
const ETX: u8 = 0x03;
const ACK: u8 = 0x06;

// Type alias for the loopback interface with the Agilent4Uhv driver.
type Agilent4UhvLbk = Agilent4Uhv<LoopbackInterfaceBytes>;

/// Function that creates a new Agilent4Uhv instance with the given input
/// and output commands.
fn crt_inst(host2inst: Vec<Vec<u8>>, inst2host: Vec<Vec<u8>>) -> Agilent4UhvLbk {
    let interface = LoopbackInterfaceBytes::new(host2inst, inst2host);
    Agilent4Uhv::try_new(interface).unwrap()
}

/// Get `host2inst` and `inst2host` `Vec<Vec<u8>>` structures for initialization (units!).
fn init_unit_cmd_bytes() -> (Vec<Vec<u8>>, Vec<Vec<u8>>) {
    let mut host2inst: Vec<u8> = Vec::from([STX, ADDR, b'6', b'0', b'0', READ, ETX]);
    let mut inst2host: Vec<u8> = Vec::from([
        STX, ADDR, b'6', b'0', b'0', WRT, b'0', b'0', b'0', b'0', b'0', b'1', ETX,
    ]);
    add_crc(&mut host2inst);
    add_crc(&mut inst2host);
    (vec![host2inst], vec![inst2host])
}

/// An empty instrument instance with no commands, but queries update when initialized.
///
/// Used for basic testing and the standard initialization test.
#[fixture]
fn emp_inst() -> Agilent4UhvLbk {
    let (h2i, i2h) = init_unit_cmd_bytes();
    crt_inst(h2i, i2h)
}

/// This test initializes the instruments and ensures that unit query is performed correctly.
#[rstest]
fn test_initialization(mut _emp_inst: Agilent4UhvLbk) {}

/// This test checks initialization with a non-default address.
#[rstest]
fn test_initialization_non_default_address() {
    let addr = 0x0D; // Non-default address

    let mut host2inst: Vec<u8> = Vec::from([STX, 0x80 + addr, b'6', b'0', b'0', READ, ETX]);
    let mut inst2host: Vec<u8> = Vec::from([
        STX,
        0x80 + addr,
        b'6',
        b'0',
        b'0',
        WRT,
        b'0',
        b'0',
        b'0',
        b'0',
        b'0',
        b'1',
        ETX,
    ]);
    add_crc(&mut host2inst);
    add_crc(&mut inst2host);

    let interface = LoopbackInterfaceBytes::new(vec![host2inst], vec![inst2host]);
    let _ = Agilent4Uhv::try_new_with_address(interface, addr).unwrap();
}

/// Get the name of the instrument.
///
/// Ensure that the string is properly trimmed of whitespace from left and right.
#[rstest]
fn test_get_name() {
    let (mut host2inst, mut inst2host) = init_unit_cmd_bytes();

    let mut cmd = vec![STX, ADDR, b'3', b'1', b'9', READ, ETX];
    add_crc(&mut cmd);
    host2inst.push(cmd);

    let mut resp = vec![
        STX, ADDR, b'3', b'1', b'9', WRT, b' ', b'A', b'g', b'i', b'l', b'e', b'n', b't', b' ',
        b'4', b'U', b'H', b'V', b' ', ETX,
    ];
    add_crc(&mut resp);
    inst2host.push(resp);

    let mut inst = crt_inst(host2inst, inst2host);
    let name = inst.get_name().unwrap();
    assert_eq!(name, "Agilent 4UHV");
}

/// Get and set the unit of the instrument.
#[rstest]
#[case(b'0', Unit::Torr)]
#[case(b'1', Unit::mBar)]
#[case(b'2', Unit::Pa)]
fn test_get_set_unit(#[case] code: u8, #[case] unit_case: Unit) {
    let (mut host2inst, mut inst2host) = init_unit_cmd_bytes();
    // query unit (mbar)
    host2inst.push(host2inst[0].clone());
    inst2host.push(inst2host[0].clone());

    // set unit
    let mut cmd = vec![
        STX, ADDR, b'6', b'0', b'0', WRT, b'0', b'0', b'0', b'0', b'0', code, ETX,
    ];
    add_crc(&mut cmd);
    host2inst.push(cmd);

    let mut resp = vec![STX, ADDR, ACK, ETX];
    add_crc(&mut resp);
    inst2host.push(resp);

    let mut inst = crt_inst(host2inst, inst2host);
    let unit = inst.get_unit().unwrap();
    assert_eq!(unit, Unit::mBar);

    inst.set_unit(unit_case).unwrap();
}

/// Set the high voltage state of a given channel.
#[rstest]
#[case(0)]
#[case(1)]
#[case(2)]
#[case(3)]
fn test_channel_get_set_hv_state(#[case] channel: usize) {
    let (mut host2inst, mut inst2host) = init_unit_cmd_bytes();

    let winbt = format!("{}", channel + 1).as_bytes()[0];

    // command to set HV state of channel to ON
    let mut cmd = vec![
        STX, ADDR, b'0', b'1', winbt, WRT, b'0', b'0', b'0', b'0', b'0', b'1', ETX,
    ];
    add_crc(&mut cmd);
    host2inst.push(cmd);

    let mut resp = vec![STX, ADDR, ACK, ETX];
    add_crc(&mut resp);
    inst2host.push(resp);

    // command to get HV state of channel
    let mut cmd = vec![STX, ADDR, b'0', b'1', winbt, READ, ETX];
    add_crc(&mut cmd);
    host2inst.push(cmd);

    let mut resp = vec![
        STX, ADDR, b'0', b'1', winbt, WRT, b'0', b'0', b'0', b'0', b'0', b'1', ETX,
    ];
    add_crc(&mut resp);
    inst2host.push(resp);

    let mut inst = crt_inst(host2inst, inst2host);
    let mut channel = inst.get_channel(channel).unwrap();
    channel.set_hv_state(true.into()).unwrap();

    assert_eq!(channel.get_hv_state().unwrap(), HvState::On);
}

/// Read the pressure from a given channel.
#[rstest]
#[case(0)]
#[case(1)]
#[case(2)]
#[case(3)]
fn test_channel_read_pressure(#[case] channel: usize) {
    // expected pressure value
    let p_exp = measurements::Pressure::from_millibars(0.0000123);

    let (mut host2inst, mut inst2host) = init_unit_cmd_bytes();
    let winbt = format!("{}", channel + 1).as_bytes()[0];

    // command to read pressure of channel
    let mut cmd = vec![STX, ADDR, b'8', winbt, b'2', READ, ETX];
    add_crc(&mut cmd);
    host2inst.push(cmd);

    let mut resp = vec![
        STX, ADDR, b'8', winbt, b'2', WRT, b'1', b'.', b'2', b'3', b'E', b'-', b'0', b'5', ETX,
    ];
    add_crc(&mut resp);
    inst2host.push(resp);

    let mut inst = crt_inst(host2inst, inst2host);
    let mut channel = inst.get_channel(channel).unwrap();
    let p_meas = channel.get_pressure().unwrap();

    assert_almost_eq(p_meas.as_pascals(), p_exp.as_pascals());
}

/// Getting an invalid channel errors out.
#[rstest]
fn test_channel_get_invalid_channel(mut emp_inst: Agilent4UhvLbk) {
    assert!(emp_inst.get_channel(4).is_err());
}

/// Helper function to add checksum to a given command vector.
///
/// Vector must already go from STX to ETX inclusive. STX is ignored for checksum calculation.
fn add_crc(cmd: &mut Vec<u8>) {
    let crc = cmd.iter().skip(1).fold(0u8, |acc, b| acc ^ b);
    let crc_hex = format!("{:02X}", crc);
    let crc_bytes = crc_hex.as_bytes();
    cmd.push(crc_bytes[0]);
    cmd.push(crc_bytes[1]);
}
