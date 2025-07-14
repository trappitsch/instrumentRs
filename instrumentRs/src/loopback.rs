//! The loopback module provides an instrument simulator for testing purposes.

use std::fmt;

use crate::{InstrumentError, InstrumentInterface};

/// A self-incrementing index structure that by default starts at 0 and increments whenever `next`
/// is called.
#[derive(Debug, Default)]
struct IncrIndex {
    index: usize,
}

impl IncrIndex {
    fn next(&mut self) -> usize {
        let current = self.index;
        self.index += 1;
        current
    }
}

/// An interface that allows you to simply write tests for your instrument driver.
///
/// TODO: Write docs for how to use the loopback interface!
pub struct LoopbackInterface<T>
where
    T: AsRef<[u8]> + fmt::Display + PartialEq,
{
    from_host: Vec<T>,
    from_inst: Vec<T>,
    from_host_index: IncrIndex,
    from_inst_index: IncrIndex,
    terminator: String,
}

impl<T> LoopbackInterface<T>
where
    T: AsRef<[u8]> + fmt::Display + PartialEq,
{
    /// Create a new loopback instrument with given commands to and from instrument.
    ///
    /// The commands are read in order. Call the [`LoopbackInstrument::finalize`] command in order to ensure that no
    /// commands are left in either vector.
    ///
    /// # Arguments:
    /// * `from_host` - Commands from host to instrument.
    /// * `from_inst` - Commands from instrument to host.
    pub fn new(from_host: Vec<T>, from_inst: Vec<T>) -> Self {
        LoopbackInterface {
            from_host,
            from_inst,
            from_host_index: IncrIndex::default(),
            from_inst_index: IncrIndex::default(),
            terminator: "\n".to_string(), // Default terminator.
        }
    }

    /// This command panics if not all commands in the LoopbackInstrument have been used.
    ///
    /// You should use this command at the end of your test in order to make sure that all the
    /// input and output you provided to the `LoopbackCommunicator` have been consumed.
    pub fn finalize(&mut self) {
        let from_host_leftover = self.from_host.get(self.from_host_index.next());
        let from_inst_leftover = self.from_inst.get(self.from_inst_index.next());
        if let Some(fil) = from_host_leftover {
            panic!("Leftover expected commands found from host to instrument: {fil}");
        }
        if let Some(fil) = from_inst_leftover {
            panic!("Leftover expected commands found from instrument to host: {fil}");
        }
    }

    /// Test the interfaces terminator and ensure the right one is set.
    ///
    /// The correct terminator can either be the default one or the one that is set when the
    /// interface was initialized via the `set_terminator` function.
    pub fn test_terminator(&self, expected_terminator: &str) {
        assert_eq!(
            self.terminator, expected_terminator,
            "Expected terminator '{expected_terminator}', got '{}'",
            self.terminator
        );
    }
}

impl<T> InstrumentInterface for LoopbackInterface<T>
where
    T: AsRef<[u8]> + fmt::Display + PartialEq,
{
    fn sendcmd(&mut self, cmd: &str) -> Result<(), InstrumentError> {
        let exp = self
            .from_host
            .get(self.from_host_index.next())
            .expect("No more commands were expected from host to instrument.")
            .to_string();
        assert_eq!(
            exp, cmd,
            "Sendcommand mismatch: expected '{exp}', got '{cmd}'"
        );
        Ok(())
    }

    fn query(&mut self, cmd: &str) -> Result<String, InstrumentError> {
        self.sendcmd(cmd)
            .expect("Infallible in Loopback communicator");
        Ok(self
            .from_inst
            .get(self.from_inst_index.next())
            .expect("No more responses were expected from instrument to host.")
            .to_string())
    }

    fn set_terminator(&mut self, terminator: &str) {
        self.terminator = terminator.to_string();
    }
}

// do some tests for this
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loopback_instrument() {
        let mut instrument =
            LoopbackInterface::new(vec!["CMD1", "CMD2"], vec![]);

        instrument
            .sendcmd("CMD1")
            .expect("Infallible in Loopback communicator");
        instrument
            .sendcmd("CMD2")
            .expect("Infallible in Loopback communicator");
        instrument.finalize();
    }

    #[should_panic(expected = "Sendcommand mismatch: expected 'CMD1', got 'CMD3'")]
    #[test]
    fn test_loopback_instrument_mismatch() {
        let mut instrument =
            LoopbackInterface::new(vec!["CMD1"], vec!["RESP1"]);

        instrument
            .sendcmd("CMD3")
            .expect("Infallible in Loopback communicator");
    }

    #[should_panic(expected = "Leftover expected commands found from host to instrument: CMD1")]
    #[test]
    fn test_loopback_leftover_commands_host() {
        let mut instrument = LoopbackInterface::new(vec!["CMD1"], vec![]);
        instrument.finalize();
    }

    #[should_panic(expected = "Leftover expected commands found from instrument to host: RESP")]
    #[test]
    fn test_loopback_leftover_commands_inst() {
        let empty_vec: Vec<&str> = vec![];
        let mut instrument = LoopbackInterface::new(empty_vec, vec!["RESP"]);
        instrument.finalize();
    }

    #[should_panic(expected = "Leftover expected commands found from host to instrument: CMD")]
    #[test]
    fn test_loopback_leftover_commands_both() {
        let mut instrument =
            LoopbackInterface::new(vec!["CMD"], vec!["RESP"]);
        instrument.finalize();
    }
}
