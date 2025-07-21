//! The loopback module provides an instrument simulator for testing purposes.

use std::{collections::VecDeque, fmt};

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
    terminator_exp: String,
    from_host_index: IncrIndex,
    from_inst_index: IncrIndex,
    curr_bytes: VecDeque<u8>,
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
    /// * `terminator_exp` - The expected terminator.
    pub fn new(from_host: Vec<T>, from_inst: Vec<T>, terminator_exp: &str) -> Self {
        LoopbackInterface {
            from_host,
            from_inst,
            terminator_exp: terminator_exp.to_string(), // the expected terminator
            from_host_index: IncrIndex::default(),
            from_inst_index: IncrIndex::default(),
            curr_bytes: VecDeque::new(),
            terminator: "\n".to_string(), // default terminator, as interfaces
        }
    }

    /// This command panics if not all commands in the `LoopbackInstrument` have been used.
    ///
    /// It is automatically called when the `LoopbackInstrument` is dropped, but you can also call
    /// it manually to ensure that all commands have been used.
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

    /// Get the next command from host to instrument, or panic.
    fn get_next_from_host(&mut self) -> &T {
        self.from_host
            .get(self.from_host_index.next())
            .expect("No more commands were expected from host to instrument.")
    }

    /// Get the next command from instrument to host, or panic.
    fn get_next_from_inst(&mut self) -> &T {
        self.from_inst
            .get(self.from_inst_index.next())
            .expect("No more commands were expected from instrument to host.")
    }

    /// Get the next command from host to instrument as a string including the terminator.
    fn get_next_from_host_with_terminator(&mut self) -> String {
        let cmd = self.get_next_from_host().to_string();
        format!("{cmd}{}", self.terminator_exp)
    }

    /// Get the next command from instrument to host as a string including the terminator.
    fn get_next_from_inst_with_terminator(&mut self) -> String {
        let cmd = self.get_next_from_inst().to_string();
        format!("{cmd}{}", self.terminator_exp)
    }

    /// Function to read exactly one byte from the next command from the instrument.
    ///
    /// This just panics if there are no more commands. If there are no more commands but one is
    /// required, the panic is justified as this is a test interface.
    fn read_one_byte(&mut self) -> u8 {
        match self.curr_bytes.pop_front() {
            Some(byte) => byte,
            None => {
                let next_cmd = self.get_next_from_inst_with_terminator();
                self.curr_bytes = next_cmd.as_bytes().iter().copied().collect();
                self.read_one_byte()
            }
        }
    }
}

impl<T> InstrumentInterface for LoopbackInterface<T>
where
    T: AsRef<[u8]> + fmt::Display + PartialEq,
{
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), InstrumentError> {
        for byte in buf.iter_mut() {
            *byte = self.read_one_byte();
        }
        Ok(())
    }

    fn get_terminator(&self) -> &str {
        self.terminator.as_str()
    }

    fn set_terminator(&mut self, terminator: &str) {
        self.terminator = terminator.to_string();
    }

    fn write_raw(&mut self, cmd: &[u8]) -> Result<(), InstrumentError> {
        let exp = self.get_next_from_host_with_terminator();
        assert_eq!(
            exp.as_bytes(),
            cmd,
            "Expected sendcmd '{0}', got '{1:?}'",
            exp,
            str::from_utf8(cmd)
        );
        Ok(())
    }
}

impl<T: AsRef<[u8]> + fmt::Display + PartialEq> Drop for LoopbackInterface<T> {
    fn drop(&mut self) {
        self.finalize();
    }
}

// Tests of internal functionality
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_incrementing_index() {
        let mut idx = IncrIndex::default();
        assert_eq!(0, idx.next());
        assert_eq!(1, idx.next());
        assert_eq!(2, idx.next());
    }
}
