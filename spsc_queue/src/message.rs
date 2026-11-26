use std::fmt;

/// 5-byte serial frame with checksum byte
///
/// ## Example
///
/// ```
/// use spsc_queue::message::Message;
///
/// let message = Message {
///     bytes: [0x55, 0x10, 0x20, 0x30, 0x40],
///     checksum: 0x0A,
/// };
/// ```
pub struct Message {
    pub bytes: [u8; 5],
    pub checksum: u8,
}

impl Message {
    /// Combines the Most Significant Byte and Least Significant Byte into `i16` using Big-Endian byte order.
    ///
    /// ## Example
    ///
    /// ```
    /// use spsc_queue::message::Message;
    ///
    /// assert_eq!(Message::combine_msb_lsb(0x01, 0x00), 256);
    /// assert_eq!(Message::combine_msb_lsb(0xFF, 0xFF), -1);
    /// ```
    pub const fn combine_msb_lsb(msb: u8, lsb: u8) -> i16 {
        i16::from_be_bytes([msb, lsb]) // from big endian bytes
    }

    /// Returns true if `self.checksum` is equal to the ones-complement (Bitwise NOT) of the sum of `self.bytes`
    ///
    /// ## Example
    ///
    /// ```
    /// use spsc_queue::message::Message;
    ///
    /// // Sum of bytes is 0x019F -> 0x9F (u8); Bitwise NOT is 0x60.
    /// let valid = Message {
    ///     bytes: [0x55, 0x10, 0x20, 0x30, 0x40],
    ///     checksum: 0x0A,
    /// };
    /// assert!(valid.verify());
    ///
    /// let invalid = Message {
    ///     bytes: [0x55, 0x10, 0x20, 0x30, 0x40],
    ///     checksum: 0x0B,
    /// };
    /// assert!(!invalid.verify());
    /// ```
    pub fn verify(&self) -> bool {
        // let sum = 0;
        // for byte in self.bytes {
        //     sum += byte;
        // }

        // `wrapping_add` so overflow wraps modulo 256 instead of panicking.
        let mut sum = self
            .bytes
            .iter()
            .fold(0u8, |accumulator, &b| accumulator.wrapping_add(b));

        sum = !sum;
        self.checksum == sum
    }
}

impl fmt::Display for Message {
    /// Formats the message as a CSV row containing the raw bytes, combined 16-bit sensor values, and checksum.
    ///
    /// ## Example
    ///
    /// ```
    /// use spsc_queue::message::Message;
    ///
    /// let bytes = [0x55, 0x00, 0x0A, 0xFF, 0xFE];
    /// let sum = bytes.iter().fold(0u8, |acc, &b| acc.wrapping_add(b));
    /// let msg = Message {
    ///     bytes,
    ///     checksum: !sum,
    /// };
    ///
    /// assert_eq!(msg.to_string(), "85,0,10,10,255,254,-2,163\n");
    /// ```
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            formatter,
            "{},{},{},{},{},{},{},{}",
            self.bytes[0],
            self.bytes[1],
            self.bytes[2],
            Self::combine_msb_lsb(self.bytes[1], self.bytes[2]),
            self.bytes[3],
            self.bytes[4],
            Self::combine_msb_lsb(self.bytes[3], self.bytes[4]),
            self.checksum,
        )
    }
}

#[cfg(test)]
#[path = "message.unit.rs"]
mod tests;
