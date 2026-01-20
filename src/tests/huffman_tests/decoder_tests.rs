use crate::huffman::{decoder::decode, encoder::encode};



#[cfg(test)]
#[test]
fn empty_round_trip() {
    use std::io::Cursor;
    let src = Vec::new();
    
    let mut input_reader_encode = Cursor::new(&src);
    let mut encoded_output_buffer = Vec::new();
    let _encode_info = encode(&mut input_reader_encode, "empty", None, &mut encoded_output_buffer)
        .expect("Encoding failed");

    let mut input_reader_decode = Cursor::new(encoded_output_buffer);
    let mut decompressed_output_buffer = Vec::new();
    let _decode_info = decode(&mut input_reader_decode, None, &mut decompressed_output_buffer)
        .expect("Decoding failed");

    assert!(decompressed_output_buffer.is_empty());
}

#[test]
fn single_byte_round_trip() {
    use std::io::Cursor;
    let src = b"a".to_vec();
    
    let mut input_reader_encode = Cursor::new(&src);
    let mut encoded_output_buffer = Vec::new();
    let _encode_info = encode(&mut input_reader_encode, "one", None, &mut encoded_output_buffer)
        .expect("Encoding failed");

    let mut input_reader_decode = Cursor::new(encoded_output_buffer);
    let mut decompressed_output_buffer = Vec::new();
    let _decode_info = decode(&mut input_reader_decode, None, &mut decompressed_output_buffer)
        .expect("Decoding failed");

    assert_eq!(decompressed_output_buffer, src);
}

#[test]
fn alternating_ab_round_trip() {
    use std::io::Cursor;
    let src = b"abababab".to_vec();
    
    let mut input_reader_encode = Cursor::new(&src);
    let mut encoded_output_buffer = Vec::new();
    let _encode_info = encode(&mut input_reader_encode, "alternating", None, &mut encoded_output_buffer)
        .expect("Encoding failed");

    let mut input_reader_decode = Cursor::new(encoded_output_buffer);
    let mut decompressed_output_buffer = Vec::new();
    let _decode_info = decode(&mut input_reader_decode, None, &mut decompressed_output_buffer)
        .expect("Decoding failed");

    assert_eq!(decompressed_output_buffer, src);
}

#[test]
fn full_ramp_round_trip() {
    use std::io::Cursor;
    let src: Vec<u8> = (0..=255).collect();
    
    let mut input_reader_encode = Cursor::new(&src);
    let mut encoded_output_buffer = Vec::new();
    let _encode_info = encode(&mut input_reader_encode, "ramp", None, &mut encoded_output_buffer)
        .expect("Encoding failed");

    let mut input_reader_decode = Cursor::new(encoded_output_buffer);
    let mut decompressed_output_buffer = Vec::new();
    let _decode_info = decode(&mut input_reader_decode, None, &mut decompressed_output_buffer)
        .expect("Decoding failed");

    assert_eq!(decompressed_output_buffer, src);
}

#[test]
fn random_1k_round_trip() {
    use std::io::Cursor;
    use rand::{RngCore, SeedableRng};
    use rand::rngs::StdRng;

    let mut rng = StdRng::seed_from_u64(42);
    let mut src = vec![0u8; 1024];
    rng.fill_bytes(&mut src);

    let mut input_reader_encode = Cursor::new(&src);
    let mut encoded_output_buffer = Vec::new();
    let _encode_info = encode(&mut input_reader_encode, "ramp", None, &mut encoded_output_buffer)
        .expect("Encoding failed");

    let mut input_reader_decode = Cursor::new(encoded_output_buffer);
    let mut decompressed_output_buffer = Vec::new();
    let _decode_info = decode(&mut input_reader_decode, None, &mut decompressed_output_buffer)
        .expect("Decoding failed");

    assert_eq!(decompressed_output_buffer, src);
}
