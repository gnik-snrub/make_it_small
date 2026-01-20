use crate::io::read_file;
#[cfg(test)]
#[test]
fn roundtrip_file_compression() {
    use crate::huffman::{encoder::encode, decoder::decode};
    use std::io::Cursor;

    let test_files = vec![
        "src/tests/testdata/null_bytes.bin",
        "src/tests/testdata/weird_bytes.bin",
        "src/tests/testdata/test.txt",
    ];

    for path in test_files {
        let original_data = match read_file(path) {
            Ok(bytes) => bytes,
            Err(e) => panic!("Failed to read test file '{}': {}", path, e),
        };
        let file_name = path.split('/').next_back().unwrap();

        // 1. Encode the file
        let mut input_reader_encode = Cursor::new(&original_data);
        let mut compressed_output_buffer = Vec::new();
        let encode_info = encode(&mut input_reader_encode, file_name, None, &mut compressed_output_buffer)
            .expect("Encoding failed");

        // 2. Decode the file
        let mut input_reader_decode = Cursor::new(compressed_output_buffer);
        let mut decompressed_output_buffer = Vec::new();
        let _decode_info = decode(&mut input_reader_decode, None, &mut decompressed_output_buffer)
            .expect("Decoding failed");

        assert_eq!(original_data, decompressed_output_buffer);
        assert_eq!(original_data.len() as u64, encode_info.original_size);
        assert_eq!(decompressed_output_buffer.len() as u64, encode_info.original_size);
    }
}

#[test]
fn test_encrypted_file_roundtrip() {
    
    use crate::huffman::{encoder::encode, decoder::decode};
    use std::io::Cursor;

    let original_data = b"This is a secret message that needs to be encrypted.";
    let file_name = "secret.txt";
    let password = "supersecretpassword";
    let wrong_password = "wrongpassword";

    // 1. Encode with encryption
    let mut input_reader_encode = Cursor::new(original_data);
    let mut encrypted_output_buffer = Vec::new();
    let _encode_info = encode(&mut input_reader_encode, file_name, Some(password), &mut encrypted_output_buffer)
        .expect("Encoding failed");

    // 2. Decode with correct password
    let mut input_reader_decode_correct = Cursor::new(encrypted_output_buffer.clone());
    let mut decrypted_output_buffer_correct = Vec::new();
    let _decode_info_correct = decode(&mut input_reader_decode_correct, Some(password), &mut decrypted_output_buffer_correct)
        .expect("Decryption with correct password failed");
    assert_eq!(original_data, decrypted_output_buffer_correct.as_slice());

    // 3. Decode with wrong password (should fail)
    let mut input_reader_decode_wrong = Cursor::new(encrypted_output_buffer.clone());
    let mut decrypted_output_buffer_wrong = Vec::new();
    let result_wrong_password = decode(&mut input_reader_decode_wrong, Some(wrong_password), &mut decrypted_output_buffer_wrong);
    assert!(result_wrong_password.is_err());

    // 4. Decode without password (should fail if encrypted)
    let mut input_reader_decode_no_pass = Cursor::new(encrypted_output_buffer);
    let mut decrypted_output_buffer_no_pass = Vec::new();
    let result_no_password = decode(&mut input_reader_decode_no_pass, None, &mut decrypted_output_buffer_no_pass);
    assert!(result_no_password.is_err());
}
