pub const MAGIC_BYTES: [u8; 4] = *b"SMAL";
pub const VERSION: u8 = 1;

// Flags
pub const ENCRYPTED: u8         = 0b0000_0001; // TODO
pub const HAS_CHECKSUM: u8      = 0b0000_0010; // TODO
pub const STORED_RAW: u8        = 0b0000_0100;
pub const IS_ARCHIVE: u8        = 0b0000_1000; // TODO

// Sizes
pub const SALT_LENGTH: usize = 16;
pub const IV_LENGTH: usize = 16;
