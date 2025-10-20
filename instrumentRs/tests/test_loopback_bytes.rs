//! Test cases for the LoopbackInterfaceBytes.

use rstest::*;

use instrumentrs::{InstrumentInterface, LoopbackInterfaceBytes};

/// A function that creates a new `LoopbackInterfaceBytes` with the given input and output vectors.
fn crt_lbk(input: Vec<Vec<u8>>, output: Vec<Vec<u8>>) -> LoopbackInterfaceBytes {
    LoopbackInterfaceBytes::new(input, output)
}

/// Create a loopback interface that contains no commands.
#[fixture]
fn emp_lbk() -> LoopbackInterfaceBytes {
    crt_lbk(vec![], vec![])
}

// TODO: Acknowledgement handling for bytes
// /// Check acknowledgment of a command sent to the loopback interface.
// #[rstest]
// fn check_acknowledgment() {
//     let mut lbk = crt_lbk(vec![vec![0x01]], vec![vec![0x06]]);
//     lbk.write_raw(&[0x01]).unwrap();
//     lbk.check_acknowledgment("ACK").unwrap();
// }
//
// /// Ensure that acknowledgment fails if command is not acknowledged.
// #[rstest]
// fn check_acknowledgment_fail() {
//     let mut lbk = crt_lbk(vec![], vec!["NACK"]);
//     assert!(lbk.check_acknowledgment("ACK").is_err());
// }

/// Ensure `finalize` method passes if an empty loopback interface is used.
///
/// This routine calls the finalize method manually, however, it is not necessary to do so as it is
/// implemented in the `Drop` trait for `LoopbackInterfaceBytes`.
#[rstest]
fn finalize_test(mut emp_lbk: LoopbackInterfaceBytes) {
    emp_lbk.finalize();
}

/// Ensure `finalize` method panics if comma's are left in the loopback interface.
///
/// Note that the finalize method is called in the `Drop` trait, so it is not necessary to call it
/// directly.
#[rstest]
#[case(vec![vec![0x01]], vec![])]
#[case(vec![], vec![vec![0x02]])]
#[case(vec![vec![0x01]], vec![vec![0x02]])]
#[should_panic]
fn finalize_test_panic(#[case] from_host: Vec<Vec<u8>>, #[case] from_inst: Vec<Vec<u8>>) {
    let _ = crt_lbk(from_host, from_inst);
}

#[rstest]
fn write_raw() {
    let mut lbk = crt_lbk(vec![vec![0x01], vec![0x02]], vec![]);
    lbk.write_raw(&[0x01]).unwrap();
    lbk.write_raw(&[0x02]).unwrap();
}

#[rstest]
#[should_panic]
fn sendcmd_mismatch() {
    let mut lbk = crt_lbk(vec![vec![0x01]], vec![]);
    assert!(lbk.write_raw(&[0x03]).is_err());
}

#[rstest]
fn query() {
    let mut lbk = crt_lbk(vec![vec![0x01], vec![0x02]], vec![vec![0x11], vec![0x22]]);
    lbk.write_raw(&[0x01]).unwrap();
    let mut resp1 = [0u8; 1];
    lbk.read_exact(&mut resp1).unwrap();
    assert_eq!(resp1, [0x11]);

    lbk.write_raw(&[0x02]).unwrap();
    let mut resp2 = [0u8; 1];
    lbk.read_exact(&mut resp2).unwrap();
    assert_eq!(resp2, [0x22]);
}
