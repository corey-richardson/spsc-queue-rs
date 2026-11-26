use std::fmt;

/// 5-byte serial frame with checksum byte
///
/// ## Example
///
/// ```
/// use spsc_queue::message::Message;
///
/// let message = Message {
///     bytes: [0xFF, 0x10, 0x20, 0x30, 0x40],
///     checksum: 0x60,
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
    ///     bytes: [0xFF, 0x10, 0x20, 0x30, 0x40],
    ///     checksum: 0x60,
    /// };
    /// assert!(valid.verify());
    ///
    /// let invalid = Message {
    ///     bytes: [0xFF, 0x10, 0x20, 0x30, 0x40],
    ///     checksum: 0x61,
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
    /// let bytes = [0xFF, 0x00, 0x0A, 0xFF, 0xFE];
    /// let sum = bytes.iter().fold(0u8, |acc, &b| acc.wrapping_add(b));
    /// let msg = Message {
    ///     bytes,
    ///     checksum: !sum,
    /// };
    ///
    /// assert_eq!(msg.to_string(), "255,0,10,10,255,254,-2,249\n");
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
mod tests {
    use super::*;
    use crate::config;

    fn create_valid_message(value1: i16, value2: i16) -> Message {
        let [msb1, lsb1] = value1.to_be_bytes();
        let [msb2, lsb2] = value2.to_be_bytes();
        let bytes = [config::SYNC_BYTE, msb1, lsb1, msb2, lsb2];

        let sum = bytes.iter().fold(0u8, |acc, &b| acc.wrapping_add(b));

        Message {
            bytes,
            checksum: !sum,
        }
    }

    #[test]
    fn combine_msb_lsb_handles_positive_and_negative_values() {
        assert_eq!(Message::combine_msb_lsb(0x01, 0x00), 256);
        assert_eq!(Message::combine_msb_lsb(0xFF, 0xFF), -1);
        assert_eq!(Message::combine_msb_lsb(0x80, 0x00), i16::MIN);
        assert_eq!(Message::combine_msb_lsb(0x7F, 0xFF), i16::MAX);
    }

    #[test]
    fn verifies_valid_checksum() {
        let valid = create_valid_message(256, -1);

        assert!(valid.verify());
    }

    #[test]
    fn rejects_invalid_checksum() {
        let valid = create_valid_message(256, -1);

        let mut bad_checksum = valid;
        bad_checksum.checksum = bad_checksum.checksum.wrapping_add(1);

        assert!(!bad_checksum.verify());
    }

    #[test]
    fn rejects_corrupted_payload_byte() {
        let mut mutated_message = create_valid_message(256, -1);

        // Flip a bit in a bytes[2]
        mutated_message.bytes[2] ^= 0x01;

        assert!(!mutated_message.verify());
    }

    #[test]
    fn verifies_checksum_when_byte_sum_exceeds_255() {
        // Sum of bytes is 0x0451 (1105 > 255).
        // Wrapped to u8 -> 0x51 (81); bitwise NOT !0x51 = 0xAE (174).
        let message = create_valid_message(-1, -1);

        assert_eq!(message.bytes, [config::SYNC_BYTE, 0xFF, 0xFF, 0xFF, 0xFF]);
        assert_eq!(message.checksum, 174);
        assert!(message.verify());
    }

    #[test]
    fn displays_message_in_expected_format() {
        let message = create_valid_message(256, -1);

        assert_eq!(message.to_string(), "85,1,0,256,255,255,-1,171\n")
    }

    #[test]
    fn handles_zeroed_data() {
        let valid = create_valid_message(0, 0);

        assert!(valid.verify());
        assert_eq!(valid.to_string(), "85,0,0,0,0,0,0,170\n");
    }
}
