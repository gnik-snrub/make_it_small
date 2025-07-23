use crate::huffman::encoder::encode;

#[cfg(test)]

// Helper function
fn active_codes(table: &[(u32, u8); 256]) -> usize {
    table.iter().filter(|(_, len)| *len > 0).count()
}

#[test]
fn empty_input() {
    let encoded = encode(&[].to_vec());

    assert!(encoded.output.is_empty());
    assert_eq!(encoded.padding_bits, 0);
    assert_eq!(active_codes(&encoded.code_table), 0);
}

#[test]
fn single_byte() {
    let src = b"a";
    let encoded = encode(&src.to_vec());

    assert_eq!(active_codes(&encoded.code_table), 1);
    assert_eq!(encoded.code_table[b'a' as usize].1, 1);

    assert_eq!(encoded.output.len(), 1);
    assert_eq!(encoded.padding_bits, 7);

    assert!(encoded.output[0] == 0x00 || encoded.output[0] == 0x80);
}

#[test]
fn eight_identical_bytes() {
    let src = vec![b'a'; 8];
    let encoded = encode(&src.to_vec());

    assert_eq!(active_codes(&encoded.code_table), 1);
    assert_eq!(encoded.output.len(), 1);
    assert_eq!(encoded.padding_bits, 0);

    assert!(encoded.output[0] == 0x00 || encoded.output[0] == 0xFF);
}

#[test]
fn alternating_pattern() {
    let src = b"abababab";
    let encoded = encode(&src.to_vec());

    let active: Vec<_> = [b'a', b'b']
        .iter()
        .map(|&c| encoded.code_table[c as usize].1)
        .collect();

    assert_eq!(active, vec![1, 1]);
    assert_eq!(encoded.output.len(), 1);
    assert_eq!(encoded.padding_bits, 0);

    assert!(encoded.output[0] == 0x55 || encoded.output[0] == 0xAA);
}
