//! Tests for the {{ manufacturer }} {{ device }} driver.

use rstest::*; 

use instrumentrs::LoopbackInterface;

use {{ crate_name }}::*;

// Type alias for the loopback interface with the {{ device }} driver.
type {{ device | upper_camel_case }}Lbk = {{ device | upper_camel_case }}<LoopbackInterface<String>>;

/// Function that creates a new {{ device | upper_camel_case }} instance with the given input
/// and output commands.
fn crt_inst(host2inst: Vec<&str>, inst2host: Vec<&str>) -> {{ device | upper_camel_case }}Lbk {
    let term = "{{ terminator }}";
    let h2i: Vec<String> = host2inst.iter().map(|s| s.to_string()).collect();
    let i2h: Vec<String> = inst2host.iter().map(|s| s.to_string()).collect();
    let interface = LoopbackInterface::new(h2i, i2h, term);
    {{ device | upper_camel_case }}::try_new(interface).unwrap()
}

#[fixture] 
fn emp_inst() -> {{ device | upper_camel_case }}Lbk {
    crt_inst(vec![], vec![])
}

/// This test initializes the instruments with empty vectors, which should always pass.
#[rstest]
fn test_initialization(_emp_inst: {{ device | upper_camel_case }}Lbk) {
}
