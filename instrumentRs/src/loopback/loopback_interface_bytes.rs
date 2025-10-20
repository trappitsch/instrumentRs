//! Loopback interface for instrument drivers that send data packages in bytes back and forth.
//!
//! Generally, your instrument driver in this case should implement the reading of the bytes and
//! there is no dedicated "end-of-command" terminator.

use std::collections::VecDeque;

use crate::{InstrumentError, InstrumentInterface, loopback::IncrIndex};

pub struct LoopbackInterfaceBytes {
    from_host: Vec<Vec<u8>>,
    from_inst: Vec<Vec<u8>>,
    from_host_index: IncrIndex,
    from_inst_index: IncrIndex,
    curr_bytes: VecDeque<u8>,
}

impl LoopbackInterfaceBytes {
    /// Create a new loopback instrument with given commands to and from instrument.
    ///
    /// The main purpose of this interface is to provide a simple loopback interface for testing of
    /// instrument drivers. To do so, you can provide a list of bytes that are expected to go from
    /// the host to the instrument, and a list of bytes that are expected to go from the
    /// instrument to the host. The bytes are read in order. At the end, when the
    /// [`LoopbackInterfaceBytes`] is dropped, a `finalize` function is called that checks if all
    /// bytes that you have provided have been used. If not, a the program panics. During
    /// instrument calls, whenever something is sent to the instrument that is not expected, the
    /// [`LoopbackInterfaceBytes`] will panic as well. This way, your tests can ensure easily that all
    /// bytes that you have provided are used in the correct order.
    ///
    /// # Arguments:
    /// * `from_host` - Vector of vectors for command bytes from host to instrument.
    /// * `from_inst` - Vector of vectors for command bytes from instrument to host.
    pub fn new(from_host: Vec<Vec<u8>>, from_inst: Vec<Vec<u8>>) -> Self {
        LoopbackInterfaceBytes {
            from_host,
            from_inst,
            from_host_index: IncrIndex::default(),
            from_inst_index: IncrIndex::default(),
            curr_bytes: VecDeque::new(),
        }
    }

    /// This command panics if not all commands in the [`LoopbackInterfaceBytes`] have been used.
    ///
    /// It is automatically called when the [`LoopbackInterfaceBytes`] is dropped, but you can also call
    /// it manually to ensure that all commands have been used.
    pub fn finalize(&mut self) {
        let from_host_leftover = self.from_host.get(self.from_host_index.next());
        let from_inst_leftover = self.from_inst.get(self.from_inst_index.next());
        if let Some(fil) = from_host_leftover {
            panic!("Leftover expected commands found from host to instrument: {fil:?}");
        }
        if let Some(fil) = from_inst_leftover {
            panic!("Leftover expected commands found from instrument to host: {fil:?}");
        }
    }

    /// Get the next command bytes from host to instrument, or panic.
    fn get_next_from_host(&mut self) -> &Vec<u8> {
        self.from_host
            .get(self.from_host_index.next())
            .expect("No more bytes were expected from host to instrument.")
    }

    /// Get the next bytes from instrument to host, or panic.
    fn get_next_from_inst(&mut self) -> &Vec<u8> {
        self.from_inst
            .get(self.from_inst_index.next())
            .expect("No more bytes were expected from instrument to host.")
    }

    /// Function to read exactly one byte from the next command from the instrument.
    ///
    /// This just panics if there are no more commands. If there are no more commands but one is
    /// required, the panic is justified as this is a test interface.
    fn read_one_byte(&mut self) -> u8 {
        match self.curr_bytes.pop_front() {
            Some(byte) => byte,
            None => {
                let next_cmd = self.get_next_from_inst();
                self.curr_bytes = next_cmd.clone().into();
                self.read_one_byte()
            }
        }
    }
}

impl InstrumentInterface for LoopbackInterfaceBytes {
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), InstrumentError> {
        for byte in buf.iter_mut() {
            *byte = self.read_one_byte();
        }
        Ok(())
    }

    fn write_raw(&mut self, cmd: &[u8]) -> Result<(), InstrumentError> {
        let exp = self.get_next_from_host().as_slice();
        assert_eq!(
            exp,
            cmd,
            "Expected sendcmd '{0:?}', got '{1:?}'",
            exp,
            str::from_utf8(cmd)
        );
        Ok(())
    }
}

impl Drop for LoopbackInterfaceBytes {
    fn drop(&mut self) {
        self.finalize();
    }
}
