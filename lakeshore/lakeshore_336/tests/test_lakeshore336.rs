//! Tests for the Lakeshore Lakeshore336 driver.

use rstest::*;

use instrumentrs::LoopbackInterface;

use lakeshore_336::*;

// Type alias for the loopback interface with the Lakeshore336 driver.
type Lakeshore336Lbk = Lakeshore336<LoopbackInterface<String>>;

/// Function that creates a new Lakeshore336 instance with the given input
/// and output commands.
fn crt_inst(host2inst: Vec<&str>, inst2host: Vec<&str>) -> Lakeshore336Lbk {
    let term = "\n";
    let h2i: Vec<String> = host2inst.iter().map(|s| s.to_string()).collect();
    let i2h: Vec<String> = inst2host.iter().map(|s| s.to_string()).collect();
    let interface = LoopbackInterface::new(h2i, i2h, term);
    Lakeshore336::try_new(interface).unwrap()
}

#[fixture]
fn emp_inst() -> Lakeshore336Lbk {
    crt_inst(vec![], vec![])
}

/// This test initializes the instruments with empty vectors, which should always pass.
#[rstest]
fn test_initialization(_emp_inst: Lakeshore336Lbk) {}
