pub const MAGIC_BYTES: [u8; 4] = *b"SMAL";
pub const VERSION: u8 = 1;

// Flags
pub const FLAG_ENCRYPTED: u8 = 0b0000_0001;
pub const FLAG_HAS_CHECKSUM: u8 = 0b0000_0010;

// Sizes
pub const SALT_LENGTH: usize = 16;
pub const IV_LENGTH: usize = 16;
