use crate::huffman::freq::compute_frequencies;

#[cfg(test)]

#[test]
fn freq_empty_slice() {
    let buf: &[u8] = b"";
    let table = compute_frequencies(buf);

    assert!(table.iter().all(|&c| c == 0));
}

#[test]
fn freq_single_byte() {
    let buf: &[u8] = &[42];
    let table = compute_frequencies(buf);

    assert_eq!(table[42], 1);
    assert_eq!(table.iter().sum::<u64>(), 1);
}

#[test]
fn freq_known_mix() {
    // 0x3, 1x2, 2x1
    let buf: &[u8] = &[0, 0, 0, 1, 1, 2];
    let table = compute_frequencies(buf);

    assert_eq!(table[0], 3);
    assert_eq!(table[1], 2);
    assert_eq!(table[2], 1);
    assert_eq!(table.iter().sum::<u64>(), buf.len() as u64);
}

#[test]
fn freq_hello_world() {
    let buf: &[u8] = b"hello world";
    let table = compute_frequencies(buf);

    assert_eq!(table[b'h' as usize], 1);
    assert_eq!(table[b'e' as usize], 1);
    assert_eq!(table[b'l' as usize], 3);
    assert_eq!(table[b'o' as usize], 2);
    assert_eq!(table[b' ' as usize], 1);
    assert_eq!(table[b'w' as usize], 1);
    assert_eq!(table[b'r' as usize], 1);
    assert_eq!(table[b'd' as usize], 1);
    assert_eq!(table.iter().sum::<u64>(), buf.len() as u64);
}
