use std::time::Duration;

// Serial Port Communication Settings
pub const BAUD_RATE: u32 = 115_200;
pub const BYTE_SIZE: u8 = 8;
pub const DATA_BITS: serialport::DataBits = serialport::DataBits::Eight;
pub const PARITY: serialport::Parity = serialport::Parity::None;
pub const STOP_BITS: serialport::StopBits = serialport::StopBits::One;
pub const SERIAL_BUFFER_SIZE: usize = 128;

// Packet Framing Constants
pub const SYNC_BYTE: u8 = 0x55;
pub const PACKET_SIZE: usize = 6; // Sync byte (1) + Data bytes (4) + Checksum (1)

// Subsampling display rate (~3 Hz)
pub const PRINT_SUBSAMPLE_RATE: Duration = Duration::from_millis(1000 / 3);

// Default configuration values
pub const FILENAME_PREFIX: &str = "log";
