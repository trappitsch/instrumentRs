//! The loopback module provides an instrument simulator for testing purposes.
//!
//! The [`LoopbackInterfaceString`] allows to test instruments drivers that communicate using
//! strings (which are then encoded as bytes of course) and have a fixed terminator to declare the
//! end of a line.
//!
//! Check out the [`LoopbackInterfaceString`] for more details and examples on how to use it. You can
//! also find simple and more advanced test examples that use the loopback interface in the
//! instrument drivers that are available in the GitHub repository of this project.

mod loopback_interface_bytes;
mod loopback_interface_string;

pub use loopback_interface_bytes::*;
pub use loopback_interface_string::*;

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
