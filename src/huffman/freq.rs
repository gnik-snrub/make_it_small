use crate::checksum::{adler32_stream_finalize, adler32_stream_update};
use std::io::Read;

pub fn compute_frequencies<R: Read>(reader: &mut R) -> std::io::Result<([u64; 256], u32, u64)> {
    // Return type change
    let mut freq = [0u64; 256];
    let mut buffer = [0; 4096]; // Read in 4KB chunks

    let mut s1: u64 = 1; // Initialize Adler-32
    let mut s2: u64 = 0; // Initialize Adler-32
    let mut original_size: u64 = 0;

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break; // EOF
        }

        for i in 0..bytes_read {
            freq[buffer[i] as usize] += 1;
        }

        adler32_stream_update(&mut s1, &mut s2, &buffer[0..bytes_read]); // Update checksum
        original_size += bytes_read as u64; // Update original size
    }

    let final_checksum = adler32_stream_finalize(s1, s2); // Finalize checksum

    Ok((freq, final_checksum, original_size)) // Return all three values
}
