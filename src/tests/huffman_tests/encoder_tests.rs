#[cfg(test)]
use crate::headers::Headers;
use crate::huffman::encoder::encode;

#[cfg(test)]

// Helper function
fn parse(file: &Vec<u8>) -> (Headers, usize, Vec<u8>) {
    let (hdr, cursor) = Headers::from_bytes(&file).expect("parse header");
    let payload = file[cursor..].to_vec();
    (hdr, cursor, payload)
}

#[test]
fn empty_input() {
    let src = Vec::new();
    let file = encode(&src, "empty");
    let (hdr, cursor, payload) = parse(&file);


    assert_eq!(hdr.original_file_name, "empty");
    assert_eq!(hdr.compressed_size, 0);
    assert_eq!(hdr.padding_bits, 0);
    assert!(hdr.lengths.iter().all(|&l| l == 0));
    assert!(payload.is_empty());
    assert_eq!(cursor, hdr.original_size as usize + 41 + hdr.original_file_name.len() + 256);
}

#[test]
fn single_byte() {
    let src = b"a".to_vec();
    let file = encode(&src, "one");
    let (hdr, cursor, payload) = parse(&file);

    assert_eq!(hdr.original_file_name, "one");
    assert_eq!(hdr.compressed_size, 1);
    assert_eq!(hdr.padding_bits, 7);
    assert_eq!(hdr.lengths[b'a' as usize], 1);
    assert_eq!(payload.len(), 1);
    assert!(payload[0] == 0x00 || payload[0] == 0x80);
    assert_eq!(file.len(), cursor + 1);
}

#[test]
fn alternating_ab() {
    let src = b"abababab".to_vec();
    let file = encode(&src, "alt");
    let (hdr, cursor, payload) = parse(&file);

    assert_eq!(hdr.compressed_size, 1);
    assert_eq!(hdr.padding_bits, 0);
    assert_eq!(hdr.lengths[b'a' as usize], 1);
    assert_eq!(hdr.lengths[b'b' as usize], 1);
    assert!(payload[0] == 0x55 || payload[0] == 0xAA);
    assert_eq!(file.len(), cursor + 1);
}
