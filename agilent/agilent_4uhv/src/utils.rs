//! Utilities for the driver that are used in multiple places.

/// Calculate the checksum (CRC) for a given byte slice.
///
/// Take the XOR of all bytes in the slice, represent it as 1 byte HEX, then turn that into a
/// 2-byte ASCII.
///
/// # Arguments
/// * `data`: The byte slice to calculate the checksum for.
///
/// # Returns
/// A 2-element array containing the ASCII representation of the checksum.
pub fn calculate_crc(data: &[u8]) -> [u8; 2] {
    let crc = data.iter().fold(0u8, |acc, b| acc ^ b);
    let crc_hex = format!("{:02X}", crc);
    let crc_bytes = crc_hex.as_bytes();
    [crc_bytes[0], crc_bytes[1]]
}
