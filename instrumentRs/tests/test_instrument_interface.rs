//! Tests for the default implementation of the [`InstrumentInterface`] trait.

use std::{collections::VecDeque, io::Read, io::Write, time::Duration};

use rstest::*;

use instrumentrs::{InstrumentError, InstrumentInterface};

struct TestInstrument<P: Read + Write> {
    port: P,
    _terminator: String,
    _timeout: Duration,
}

impl<P: Read + Write> InstrumentInterface for TestInstrument<P> {
    fn read_exact(&mut self, _buf: &mut [u8]) -> Result<(), InstrumentError> {
        Ok(())
    }

    fn write_raw(&mut self, _data: &[u8]) -> Result<(), InstrumentError> {
        Ok(())
    }
}

#[fixture]
fn inst() -> TestInstrument<VecDeque<u8>> {
    TestInstrument {
        port: VecDeque::new(),
        _terminator: "\r\n".to_string(),
        _timeout: Duration::from_secs(0),
    }
}

#[rstest]
fn test_default_get_terminator(inst: TestInstrument<VecDeque<u8>>) {
    assert_eq!(inst.get_terminator(), "\n");
}

#[rstest]
fn test_default_get_timeout(inst: TestInstrument<VecDeque<u8>>) {
    assert_eq!(inst.get_timeout(), Duration::from_secs(3));
}
