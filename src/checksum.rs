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