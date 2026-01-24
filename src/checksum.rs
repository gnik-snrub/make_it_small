/// Calculates the Adler-32 checksum of a byte slice.
///
/// The Adler-32 checksum is a checksum algorithm that was invented by Mark Adler.
/// It is more reliable than the Fletcher-16 checksum, and nearly as reliable as the CRC-32 checksum.
///
/// # Arguments
///
/// * `data` - A byte slice (`&[u8]`) for which to calculate the checksum.
///
/// # Returns
///
/// A `u32` representing the Adler-32 checksum.
pub fn adler32(data: &[u8]) -> u32 {
    const MOD_ADLER: u64 = 65521;

    let mut s1: u64 = 1;
    let mut s2: u64 = 0;

    for &byte in data {
        s1 += byte as u64;
        s2 += s1;
    }
    s1 %= MOD_ADLER;
    s2 %= MOD_ADLER;

    ((s2 << 16) | s1) as u32
}

pub const MOD_ADLER: u64 = 65521;

/// Updates Adler-32 checksum components incrementally.
pub fn adler32_stream_update(s1: &mut u64, s2: &mut u64, data: &[u8]) {
    for &byte in data {
        *s1 = (*s1 + byte as u64) % MOD_ADLER;
        *s2 = (*s2 + *s1) % MOD_ADLER;
    }
}

/// Finalizes Adler-32 checksum from components.
pub fn adler32_stream_finalize(s1: u64, s2: u64) -> u32 {
    ((s2 << 16) | s1) as u32
}

use std::io::{self, Read};

/// Calculates the Adler-32 checksum and original size of a stream.
///
/// Reads from the provided `reader` in chunks, updating the Adler-32 checksum incrementally.
///
/// # Arguments
///
/// * `reader` - A mutable reference to an object implementing the `Read` trait.
///
/// # Returns
///
/// A `Result` containing a tuple `(u32, u64)` on success, where the first element is the
/// Adler-32 checksum and the second is the original size of the stream in bytes.
/// Returns an `io::Error` if an error occurs during reading.
pub fn adler32_stream<R: Read>(reader: &mut R) -> io::Result<(u32, u64)> {
    let mut s1: u64 = 1;
    let mut s2: u64 = 0;
    let mut original_size: u64 = 0;
    let mut buffer = [0; 4096]; // Read in 4KB chunks

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break; // End of stream
        }
        adler32_stream_update(&mut s1, &mut s2, &buffer[..bytes_read]);
        original_size += bytes_read as u64;
    }

    Ok((adler32_stream_finalize(s1, s2), original_size))
}
