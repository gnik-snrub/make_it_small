pub fn compute_frequencies(buf: &[u8]) -> [u64; 256] {
    let mut freq = [0u64; 256];

    for byte in buf {
        freq[*byte as usize] += 1;
    }

    freq
}

