use digoutbox::*;
use instrumentrs::{InstrumentError, LoopbackInterface};
use rstest::*;

/// Create a new loopback instrument from the given input string slices.
fn crt_inst(host2inst: Vec<&str>, inst2host: Vec<&str>) -> DigOutBox<LoopbackInterface<String>> {
    let term = "\n";
    let h2i: Vec<String> = host2inst.iter().map(|s| s.to_string()).collect();
    let i2h: Vec<String> = inst2host.iter().map(|s| s.to_string()).collect();
    let interface = LoopbackInterface::new(h2i, i2h, term);
    DigOutBox::new(interface)
}

/// Create an empty loopback interface for the DigOutBox instrument.
#[fixture]
fn emp_inst() -> DigOutBox<LoopbackInterface<String>> {
    crt_inst(vec![], vec![])
}

#[rstest]
pub fn test_all_off() {
    let mut inst = crt_inst(vec!["ALLOFF"], vec![]);

    inst.all_off().unwrap();
}
#[rstest]
fn test_get_all_outputs() {
    let mut inst = crt_inst(vec!["ALLDO?"], vec!["1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0"]);

    assert_eq!(
        inst.get_all_outputs().unwrap(),
        vec![
            true, false, true, false, true, false, true, false, true, false, true, false, true,
            false, true, false
        ]
    );
}

#[rstest]
fn test_get_interlock_status() {
    let mut inst = crt_inst(vec!["INTERLOCKS?", "INTERLOCKS?"], vec!["0", "1"]);

    let interlock_status = inst.get_interlock_status().unwrap();
    assert_eq!(interlock_status, InterlockStatus::Ready);
    assert!(format!("{interlock_status}").contains("is ready"));

    let interlock_status = inst.get_interlock_status().unwrap();
    assert_eq!(interlock_status, InterlockStatus::Interlocked);
    assert!(format!("{interlock_status}").contains("is interlocked and not ready"));
}

#[rstest]
fn test_get_name() {
    let mut inst = crt_inst(vec!["*IDN?"], vec!["Inst Name"]);

    assert_eq!(inst.get_name().unwrap(), "Inst Name");
}

#[rstest]
fn test_get_software_control_status() {
    let mut inst = crt_inst(vec!["SWL?", "SWL?"], vec!["0", "1"]);

    let scs = inst.get_software_control_status().unwrap();
    assert_eq!(scs, SoftwareControlStatus::Ready);
    assert!(format!("{scs}").contains("Software control is possible."));

    let scs = inst.get_software_control_status().unwrap();
    assert_eq!(scs, SoftwareControlStatus::LockedOut);
    assert!(format!("{scs}").contains("is locked out"));
}

// Tests for the channels
#[rstest]
fn test_get_channel(mut emp_inst: DigOutBox<LoopbackInterface<String>>) {
    // Get a channel and check if it is created correctly
    let channel = emp_inst.get_channel(0).unwrap();

    // Try to get a channel that is out of range
    match emp_inst.get_channel(17) {
        Err(InstrumentError::ChannelIndexOutOfRange { idx, nof_channels }) => {
            assert_eq!(idx, 17);
            assert_eq!(nof_channels, 16);
        }
        _ => panic!("Expected ChannelIndexOutOfRange error"),
    }

    // Now set the box up so it has only 6 channels
    emp_inst.set_num_channels(6);
    // Try to get a channel that is out of range
    assert!(emp_inst.get_channel(6).is_err());
}

#[rstest]
fn test_channel_output() {
    let mut inst = crt_inst(vec!["DO0 1", "DO0?", "DO1 0", "DO1?"], vec!["1", "0"]);

    let mut ch0 = inst.get_channel(0).unwrap();
    ch0.set_output(true).unwrap();
    assert!(ch0.get_output().unwrap());

    let mut ch1 = inst.get_channel(1).unwrap();
    ch1.set_output(false).unwrap();
    assert!(!ch1.get_output().unwrap());
}
