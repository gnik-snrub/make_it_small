#[cfg(test)]
use crate::headers::Headers;
use crate::{flags::is_stored_raw, huffman::encoder::encode};
use std::io::Cursor;


#[cfg(test)]
// Helper function
fn parse(file_data: &[u8]) -> (Headers, Vec<u8>) {
    let mut reader = Cursor::new(file_data);
    let hdr = Headers::from_reader(&mut reader).expect("parse header");
    let payload_start = reader.position() as usize;
    let payload = file_data[payload_start..].to_vec();
    (hdr, payload)
}

#[test]
fn empty_input() {
    let src = Vec::new();
    let mut input_reader = Cursor::new(&src);
    let mut encoded_output_buffer = Vec::new();
    let _encode_info = encode(&mut input_reader, "empty", None, &mut encoded_output_buffer)
        .expect("Encoding failed");

    let (hdr, payload) = parse(&encoded_output_buffer);

    // Calculate the expected total size for an empty file (header only)
    let mut dummy_header_for_size_calc = Headers::new();
    dummy_header_for_size_calc.original_file_name = "empty".to_string(); // Use the same name as in the test
    let expected_total_output_size = dummy_header_for_size_calc.to_bytes().len() as u64;

    assert_eq!(hdr.original_file_name, "empty");
    assert_eq!(hdr.compressed_size, 0); // For empty input, there's no compressed payload
    assert_eq!(hdr.padding_bits, 0);
    assert!(payload.is_empty());
    assert_eq!(encoded_output_buffer.len() as u64, expected_total_output_size);
}

#[test]
fn single_byte() {
    let src = b"a".to_vec();
    let mut input_reader = Cursor::new(&src);
    let mut encoded_output_buffer = Vec::new();
    let _encode_info = encode(&mut input_reader, "one", None, &mut encoded_output_buffer)
        .expect("Encoding failed");

    let (hdr, payload) = parse(&encoded_output_buffer);

    assert_eq!(hdr.original_file_name, "one");
    assert_eq!(hdr.compressed_size, 1);
    assert_eq!(hdr.padding_bits, 0);
    assert_eq!(payload.len(), 1);
    assert!(payload[0] == b'a');
    // The original assert_eq!(file.len(), cursor + 1); is no longer valid due to streaming
    // and parse returning different values.
    // The relevant assertion is payload.len() and compressed_size
}

#[test]
fn alternating_ab() {
    let src = b"abababab".to_vec();
    let mut input_reader = Cursor::new(&src);
    let mut encoded_output_buffer = Vec::new();
    let _encode_info = encode(&mut input_reader, "alt", None, &mut encoded_output_buffer)
        .expect("Encoding failed");

    let (hdr, payload) = parse(&encoded_output_buffer);

    assert_eq!(hdr.compressed_size, 1);
    assert_eq!(hdr.padding_bits, 0);
    assert!(payload[0] == 0x55 || payload[0] == 0xAA);
    // Original assertion assert_eq!(file.len(), cursor + 1); is no longer relevant.
}

#[test]
fn compressible_text() {
    let src = b"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_vec();
    let mut input_reader = Cursor::new(&src);
    let mut encoded_output_buffer = Vec::new();
    let _encode_info = encode(&mut input_reader, "small.txt", None, &mut encoded_output_buffer)
        .expect("Encoding failed");

    let (hdr, payload) = parse(&encoded_output_buffer);

    assert!(!is_stored_raw(hdr.flags));
    assert!(hdr.padding_bits > 0);
    assert_ne!(payload, src);
}

#[test]
fn borderline_raw_vs_compressed() {
    let src = b"ABCABCABCABCABCABCABCABCABCABCABCABCABCABCABCABC".to_vec();
    let mut input_reader = Cursor::new(&src);
    let mut encoded_output_buffer = Vec::new();
    let _encode_info = encode(&mut input_reader, "edge.txt", None, &mut encoded_output_buffer)
        .expect("Encoding failed");

    let (hdr, payload) = parse(&encoded_output_buffer);

    assert!(hdr.padding_bits <= 7);
    assert_eq!(payload.len() as u64, hdr.compressed_size);
}

#[test]
fn stored_raw_large_file() {
    use rand::{Rng, SeedableRng};
    use rand::rngs::StdRng;
    use std::io::Cursor;

    let mut rng = StdRng::seed_from_u64(1337);
    let src: Vec<u8> = (0..100_000).map(|_| rng.random::<u8>()).collect();

    let mut input_reader = Cursor::new(&src);
    let mut encoded_output_buffer = Vec::new();
    let _encode_info = encode(&mut input_reader, "dense.bin", None, &mut encoded_output_buffer)
        .expect("Encoding failed");

    let (hdr, payload) = parse(&encoded_output_buffer);

    assert!(is_stored_raw(hdr.flags));
    assert_eq!(hdr.padding_bits, 0);
    assert_eq!(hdr.compressed_size, hdr.original_size);
    assert_eq!(payload.len() as u64, hdr.original_size);
    assert_eq!(payload, src);
}
