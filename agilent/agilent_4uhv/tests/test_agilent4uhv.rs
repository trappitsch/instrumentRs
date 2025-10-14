//! Tests for the Agilent Agilent4Uhv driver.

use rstest::*;

use instrumentrs::LoopbackInterface;

use agilent_4uhv::*;

// Type alias for the loopback interface with the Agilent4Uhv driver.
type Agilent4UhvLbk = Agilent4Uhv<LoopbackInterface<String>>;

/// Function that creates a new Agilent4Uhv instance with the given input
/// and output commands.
fn crt_inst(host2inst: Vec<Vec<u8>>, inst2host: Vec<Vec<u8>>) -> Agilent4UhvLbk {
    let term = "";
    let interface = LoopbackInterface::new(host2inst, inst2host, term);
    Agilent4Uhv::try_new(interface).unwrap()
}

#[fixture]
fn emp_inst() -> Agilent4UhvLbk {
    crt_inst(vec![], vec![])
}

/// This test initializes the instruments with empty vectors, which should always pass.
#[rstest]
fn test_initialization(_emp_inst: Agilent4UhvLbk) {}
