//! Test cases for the LoopbackInterface.

use std::fmt::Display;

use rstest::*;

use instrumentrs::{InstrumentInterface, LoopbackInterface};

/// A function that creates a new `LoopbackInterface` with the given input and output vectors.
fn crt_lbk<T: AsRef<[u8]> + Display + PartialEq>(
    input: Vec<T>,
    output: Vec<T>,
) -> LoopbackInterface<T> {
    LoopbackInterface::new(input, output)
}

/// Create a loopback interface that contains no commands.
#[fixture]
fn emp_lbk() -> LoopbackInterface<String> {
    crt_lbk(vec![], vec![])
}

/// Check acknowledgment of a command sent to the loopback interface.
#[rstest]
fn check_acknowledgment() {
    let mut lbk = crt_lbk(vec!["cmd1"], vec!["ACK"]);
    lbk.sendcmd("cmd1").unwrap();
    lbk.check_acknowledgment("ACK").unwrap();
    lbk.finalize();
}

/// Ensure that acknowledgment fails if command is not acknowledged.
#[rstest]
fn check_acknowledgment_fail() {
    let mut lbk = crt_lbk(vec![], vec!["NACK"]);
    assert!(lbk.check_acknowledgment("ACK").is_err());
}

/// Ensure `finalize` method passes if an empty loopback interface is used.
#[rstest]
fn finalize_test(mut emp_lbk: LoopbackInterface<String>) {
    emp_lbk.finalize();
}

/// Ensure `finalize` method panics if comma's are left in the loopback interface.
#[rstest]
#[case(vec!["cmd"], vec![])]
#[case(vec![], vec!["resp"])]
#[case(vec!["cmd"], vec!["resp"])]
#[should_panic]
fn finalize_test_panic(#[case] from_host: Vec<&str>, #[case] from_inst: Vec<&str>) {
    let mut lbk = crt_lbk(from_host, from_inst);
    lbk.finalize();
}

#[rstest]
fn sendcmd() {
    let mut lbk = crt_lbk(vec!["cmd1", "cmd2"], vec![]);
    lbk.sendcmd("cmd1").unwrap();
    lbk.sendcmd("cmd2").unwrap();
    lbk.finalize();
}

#[rstest]
#[should_panic]
fn sendcmd_mismatch() {
    let mut lbk = crt_lbk(vec!["cmd1"], vec![]);
    assert!(lbk.sendcmd("cmd3").is_err());
}

#[rstest]
fn terminator(mut emp_lbk: LoopbackInterface<String>) {
    emp_lbk.test_terminator("\n");
    emp_lbk.set_terminator("\r\n");
    emp_lbk.test_terminator("\r\n");
}

#[rstest]
#[should_panic]
fn terminator_wrong(emp_lbk: LoopbackInterface<String>) {
    emp_lbk.test_terminator("\r\n");
}

#[rstest]
fn query() {
    let mut lbk = crt_lbk(vec!["cmd1", "cmd2"], vec!["resp1", "resp2"]);
    let resp1 = lbk.query("cmd1").unwrap();
    assert_eq!(resp1, "resp1");
    let resp2 = lbk.query("cmd2").unwrap();
    assert_eq!(resp2, "resp2");
    lbk.finalize();
}
