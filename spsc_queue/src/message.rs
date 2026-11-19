use std::fmt;

pub struct Message {
    pub bytes: [u8; 5],
    pub checksum: u8,
}

impl Message {
    pub const fn combine_msb_lsb(msb: u8, lsb: u8) -> i16 {
        i16::from_be_bytes([msb, lsb]) // from big endian bytes
    }

    pub fn verify(&self) -> bool {
        // let sum = 0;
        // for byte in self.bytes {
        //     sum += byte;
        // }

        let mut sum = self
            .bytes
            .iter()
            .fold(0u8, |accumalator, &b| accumalator.wrapping_add(b));

        sum = !sum;
        self.checksum == sum
    }
}

impl fmt::Display for Message {
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
