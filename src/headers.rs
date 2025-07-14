use serde::{Deserialize, Serialize};

use crate::constants::{MAGIC_BYTES, VERSION};

#[derive(Serialize, Deserialize, Debug)]
pub struct Headers {
    magic_bytes: [u8; 4],
    version: u8,
    flags: u8,
    original_size: u64,
    compressed_size: u64,
    salt_and_iv: [usize; 2],
}

pub fn write_header() -> Headers {
    Headers {
        magic_bytes: MAGIC_BYTES,
        version: VERSION,
        flags: 0b0000_0000, // TODO - Create a bitmask from selected flags using consts
        original_size: 0, // TODO - Check input file sizes, and put here
        compressed_size: 0, // TODO - Check output file sizes, and put here
        // TODO - Randomize both values with a secure RNG
        salt_and_iv: [1234432112344321, 4321123443211234]
    }
}
