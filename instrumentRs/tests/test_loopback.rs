//! Test cases for the LoopbackInterfaceStr.

use rstest::*;

use instrumentrs::{InstrumentInterface, LoopbackInterfaceStr};

/// A function that creates a new `LoopbackInterfaceStr` with the given input and output vectors.
fn crt_lbk(input: Vec<&str>, output: Vec<&str>) -> LoopbackInterfaceStr {
    let input = input.iter().map(|s| s.to_string()).collect();
    let output = output.iter().map(|s| s.to_string()).collect();
    LoopbackInterfaceStr::new(input, output, "\n")
}

/// Create a loopback interface that contains no commands.
#[fixture]
fn emp_lbk() -> LoopbackInterfaceStr {
    crt_lbk(vec![], vec![])
}

/// Check acknowledgment of a command sent to the loopback interface.
#[rstest]
fn check_acknowledgment() {
    let mut lbk = crt_lbk(vec!["cmd1"], vec!["ACK"]);
    lbk.sendcmd("cmd1").unwrap();
    lbk.check_acknowledgment("ACK").unwrap();
}

/// Ensure that acknowledgment fails if command is not acknowledged.
#[rstest]
fn check_acknowledgment_fail() {
    let mut lbk = crt_lbk(vec![], vec!["NACK"]);
    assert!(lbk.check_acknowledgment("ACK").is_err());
}

/// Ensure `finalize` method passes if an empty loopback interface is used.
///
/// This routine calls the finalize method manually, however, it is not necessary to do so as it is
/// implemented in the `Drop` trait for `LoopbackInterfaceStr`.
#[rstest]
fn finalize_test(mut emp_lbk: LoopbackInterfaceStr) {
    emp_lbk.finalize();
}

/// Ensure `finalize` method panics if comma's are left in the loopback interface.
///
/// Note that the finalize method is called in the `Drop` trait, so it is not necessary to call it
/// directly.
#[rstest]
#[case(vec!["cmd"], vec![])]
#[case(vec![], vec!["resp"])]
#[case(vec!["cmd"], vec!["resp"])]
#[should_panic]
fn finalize_test_panic(#[case] from_host: Vec<&str>, #[case] from_inst: Vec<&str>) {
    let _ = crt_lbk(from_host, from_inst);
}

#[rstest]
fn sendcmd() {
    let mut lbk = crt_lbk(vec!["cmd1", "cmd2"], vec![]);
    lbk.sendcmd("cmd1").unwrap();
    lbk.sendcmd("cmd2").unwrap();
}

#[rstest]
#[should_panic]
fn sendcmd_mismatch() {
    let mut lbk = crt_lbk(vec!["cmd1"], vec![]);
    assert!(lbk.sendcmd("cmd3").is_err());
}

#[rstest]
fn query() {
    let mut lbk = crt_lbk(vec!["cmd1", "cmd2"], vec!["resp1", "resp2"]);
    let resp1 = lbk.query("cmd1").unwrap();
    assert_eq!(resp1, "resp1");
    let resp2 = lbk.query("cmd2").unwrap();
    assert_eq!(resp2, "resp2");
}
