//! The loopback module provides an instrument simulator for testing purposes.
//!
//! Check out the [`LoopbackInterfaceStr`] for more details and examples on how to use it. You can
//! also find simple and more advanced test examples that use the loopback interface in the
//! instrument drivers that are available in the GitHub repository of this project.

use std::collections::VecDeque;

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
/// # Example
///
/// Let us build a simple instrument that would send a `"*IDN?"` command to an instrument and get
/// back a string and then write a test for it using the [`LoopbackInterfaceStr`]. The instrument itself
/// would take any interface that implements the [`InstrumentInterface`] trait.
///
/// ```
/// use std::sync::{Arc, Mutex};
/// use instrumentrs::{InstrumentInterface, InstrumentError, LoopbackInterfaceStr};
///
/// struct MyInstrument<T: InstrumentInterface> {
///    interface: Arc<Mutex<T>>,
/// }
///
/// impl<T: InstrumentInterface> MyInstrument<T> {
///    fn new(interface: T) -> Self {
///        let interface = Arc::new(Mutex::new(interface));
///        MyInstrument { interface }
///    }
///
///    fn get_name(&mut self) -> Result<String, InstrumentError> {
///        self.interface.lock().unwrap().query("*IDN?")
///    }
/// }
///
/// #[cfg(test)]
/// mod tests {
///    use super::*;
///
///    /// Simple check to ensure the instrument returns its name as expected.
///    #[test]
///    fn test_get_name() {
///        let host2inst = vec!["*IDN?"];
///        let inst2host = vec!["MyInstrument,1.0,1234"];
///        let terminator = "\n";  // the default terminator
///        
///        // Create the loopback interface with the expected commands.
///        let loopback = LoopbackInterfaceStr::new(host2inst, inst2host, terminator);
///
///        // Create the instrument
///        let mut inst= MyInstrument::new(loopback);
///        assert_eq!("MyInstrument,1.0,1234", inst.get_name().unwrap());
///    }
///
///    /// This test will panic as it expects as it expects a command from host to instrument that
///    /// is never provided.
///    #[test]
///    #[should_panic]
///    fn test_leftover_commands() {
///        let host2inst = vec!["*IDN?"];
///        let inst2host = vec!["MyInstrument,1.0,1234"];
///
///        // Create the loopback interface with the expected commands.
///        let loopback = LoopbackInterfaceStr::new(host2inst, inst2host, "\n");
///
///        // Create the instrument
///        let mut inst = MyInstrument::new(loopback);
///
///        // Instrument dropped here -> panics as host2inst and inst2host have unused commands.
///    }
///
///    /// This test will panic as an unexpected command is sent to the instrument.
///    #[test]
///    #[should_panic]
///    fn test_unexpected_command() {
///        let host2inst = vec!["*IDX?"];
///        let inst2host = vec!["MyInstrument,1.0,1234"];
///
///        // Create the loopback interface with the expected commands.
///        let loopback = LoopbackInterfaceStr::new(host2inst, inst2host, "\n");
///
///        // Create the instrument
///        let mut inst = MyInstrument::new(loopback);
///
///        // This will panic as the command is not expected.
///        let _ = inst.get_name().unwrap();
///     }
/// }
/// ```
pub struct LoopbackInterfaceStr {
    from_host: Vec<String>,
    from_inst: Vec<String>,
    terminator_exp: String,
    from_host_index: IncrIndex,
    from_inst_index: IncrIndex,
    curr_bytes: VecDeque<u8>,
    terminator: String,
}

impl LoopbackInterfaceStr {
    /// Create a new loopback instrument with given commands to and from instrument.
    ///
    /// The main purpose of this interface is to provide a simple loopback interface for testing of
    /// instrument drivers. To do so, you can provide a list of commands that are expected to go from
    /// the host to the instrument, and a list of commands that are expected to go from the
    /// instrument to the host. The commands are read in order. At the end, when the
    /// [`LoopbackInterfaceStr`] is dropped, a `finalize` function is called that checks if all
    /// commands that you have provided have been used. If not, a the program panics. During
    /// instrument calls, whenever something is sent to the instrument that is not expected, the
    /// [`LoopbackInterfaceStr`] will panic as well. This way, your tests can ensure easily that all
    /// commands that you have provided are used in the correct order.
    ///
    /// # Arguments:
    /// * `from_host` - Commands from host to instrument.
    /// * `from_inst` - Commands from instrument to host.
    /// * `terminator_exp` - The expected terminator. This is required for every instantiation of
    ///   the loopback interface.
    pub fn new(from_host: Vec<String>, from_inst: Vec<String>, terminator_exp: &str) -> Self {
        LoopbackInterfaceStr {
            from_host,
            from_inst,
            terminator_exp: terminator_exp.to_string(), // the expected terminator
            from_host_index: IncrIndex::default(),
            from_inst_index: IncrIndex::default(),
            curr_bytes: VecDeque::new(),
            terminator: "\n".to_string(), // default terminator, as interfaces
        }
    }

    /// This command panics if not all commands in the [`LoopbackInterfaceStr`] have been used.
    ///
    /// It is automatically called when the [`LoopbackInterfaceStr`] is dropped, but you can also call
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
    fn get_next_from_host(&mut self) -> &str {
        self.from_host
            .get(self.from_host_index.next())
            .expect("No more commands were expected from host to instrument.")
    }

    /// Get the next command from instrument to host, or panic.
    fn get_next_from_inst(&mut self) -> &str {
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

impl InstrumentInterface for LoopbackInterfaceStr {
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

impl Drop for LoopbackInterfaceStr {
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
