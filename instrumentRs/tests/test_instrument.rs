//! Tests for the [`Instrument`] interface itself.
//!
//! Note that many of the functionality of the [`InstrumentInterface`] trait is tested in the
//! [`instrumentrs::LoopbackInterfaceStr`] tests.

use std::{collections::VecDeque, time::Duration};

use rstest::*;

use instrumentrs::{Instrument, InstrumentError, InstrumentInterface};

/// Set up a empty instrument with default 3 second timeout.
#[fixture]
fn empt_inst() -> Instrument<VecDeque<u8>> {
    Instrument::new(VecDeque::new(), std::time::Duration::from_secs(3))
}

/// Set up a instrument with no terminator and no timeout duration.
#[fixture]
fn no_term_inst() -> Instrument<VecDeque<u8>> {
    Instrument::new(
        VecDeque::from(vec![b'r', b'e', b's', b'p']),
        std::time::Duration::from_secs(0),
    )
}

#[rstest]
fn test_instrument_terminator(mut empt_inst: Instrument<VecDeque<u8>>) {
    assert_eq!(empt_inst.get_terminator(), "\n");

    empt_inst.set_terminator("\r\n");
    assert_eq!(empt_inst.get_terminator(), "\r\n");
}

#[rstest]
fn test_instrument_timeout(empt_inst: Instrument<VecDeque<u8>>) {
    assert_eq!(empt_inst.get_timeout(), std::time::Duration::from_secs(3));
}

#[rstest]
fn test_instrument_write_read(mut empt_inst: Instrument<VecDeque<u8>>) {
    let data = b"Hello, Instrument!";
    empt_inst.write_raw(data).unwrap();

    let mut buf = vec![0; data.len()];
    empt_inst.read_exact(&mut buf).unwrap();
    assert_eq!(&buf, data);
}

#[rstest]
fn test_instrument_read_until_terminator_timeout(mut no_term_inst: Instrument<VecDeque<u8>>) {
    let timeout_exp = Duration::from_secs(0);

    match no_term_inst.read_until_terminator() {
        Err(InstrumentError::Timeout(timeout)) => {
            assert_eq!(timeout_exp, timeout);
        }
        _ => panic!("Expected timeout error, but got a different result."),
    }
}

#[rstest]
fn test_instrument_query_timeout(mut no_term_inst: Instrument<VecDeque<u8>>) {
    let timeout_exp = Duration::from_secs(0);
    let query_exp = "QUERY";

    match no_term_inst.query(query_exp) {
        Err(InstrumentError::TimeoutQuery { query, timeout }) => {
            assert_eq!(query_exp, query);
            assert_eq!(timeout_exp, timeout);
        }
        _ => panic!("Expected timeout error, but got a different result."),
    }
}
