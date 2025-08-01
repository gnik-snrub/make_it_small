#[cfg(test)]
use crate::headers::Headers;
use crate::{flags::is_stored_raw, huffman::{decoder::decode, encoder::encode, tree::serialize_tree}};


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


    let mut tree_serial = Vec::new();
    serialize_tree(&hdr.tree, &mut tree_serial);

    assert_eq!(hdr.original_file_name, "empty");
    assert_eq!(hdr.compressed_size, 0);
    assert_eq!(hdr.padding_bits, 0);
    assert!(payload.is_empty());
    assert_eq!(cursor, hdr.original_size as usize + 43 + hdr.original_file_name.len() + tree_serial.len());
}

#[test]
fn single_byte() {
    let src = b"a".to_vec();
    let file = encode(&src, "one");
    let (hdr, cursor, payload) = parse(&file);

    assert_eq!(hdr.original_file_name, "one");
    assert_eq!(hdr.compressed_size, 1);
    assert_eq!(hdr.padding_bits, 0);
    assert_eq!(payload.len(), 1);
    assert!(payload[0] == b'a');
    assert_eq!(file.len(), cursor + 1);
}

#[test]
fn alternating_ab() {
    let src = b"abababab".to_vec();
    let file = encode(&src, "alt");
    let (hdr, cursor, payload) = parse(&file);

    assert_eq!(hdr.compressed_size, 1);
    assert_eq!(hdr.padding_bits, 0);
    assert!(payload[0] == 0x55 || payload[0] == 0xAA);
    assert_eq!(file.len(), cursor + 1);
}

#[test]
fn compressible_text() {
    let src = b"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_vec();
    let file = encode(&src, "small.txt");
    let (hdr, _, payload) = parse(&file);

    assert!(!is_stored_raw(hdr.flags));
    assert!(hdr.padding_bits > 0);
    assert_ne!(payload, src);
}

#[test]
fn borderline_raw_vs_compressed() {
    let src = b"ABCABCABCABCABCABCABCABCABCABCABCABCABCABCABCABC".to_vec();
    let file = encode(&src, "edge.txt");
    let (hdr, _, payload) = parse(&file);

    assert!(hdr.padding_bits <= 7);
    assert_eq!(payload.len() as u64, hdr.compressed_size);
}

#[test]
fn stored_raw_large_file() {
    use rand::{Rng, SeedableRng};
    use rand::rngs::StdRng;

    let mut rng = StdRng::seed_from_u64(1337);
    let src: Vec<u8> = (0..100_000).map(|_| rng.random()).collect();

    let file = encode(&src, "dense.bin");
    let (hdr, _, payload) = parse(&file);

    assert!(is_stored_raw(hdr.flags));
    assert_eq!(hdr.padding_bits, 0);
    assert_eq!(hdr.compressed_size, hdr.original_size);
    assert_eq!(payload.len(), hdr.original_size as usize);
    assert_eq!(payload, src);
}
