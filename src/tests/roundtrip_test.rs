use crate::io::read_file;
#[cfg(test)]
#[test]
fn roundtrip_file_compression() {
    use crate::huffman::{encoder::encode, decoder::decode};
    use std::io::Cursor;
    use crate::headers::Headers; // Added import

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
        let mut encoded_cursor = Cursor::new(compressed_output_buffer); // Corrected variable name
        let header = Headers::from_reader(&mut encoded_cursor).expect("Failed to read header for decode"); // Read header
        let mut decoded_output_buffer = Vec::new(); // Corrected variable name

        let _decode_info = decode(header, &mut encoded_cursor, None, &mut decoded_output_buffer) // Updated call
            .expect("Decoding failed");

        assert_eq!(original_data, decoded_output_buffer); // Corrected variable name
        assert_eq!(original_data.len() as u64, encode_info.original_size);
        assert_eq!(decoded_output_buffer.len() as u64, encode_info.original_size); // Corrected variable name
    }
}

#[test]
fn test_encrypted_file_roundtrip() {
    
    use crate::huffman::{encoder::encode, decoder::decode};
    use std::io::Cursor;
    use crate::headers::Headers; // Added import

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
    let mut encoded_cursor = Cursor::new(encrypted_output_buffer.clone());
    let header = Headers::from_reader(&mut encoded_cursor).expect("Failed to read header for decode");
    let mut decoded_output_buffer = Vec::new();

    let _decode_info = decode(header.clone(), &mut encoded_cursor, Some(password), &mut decoded_output_buffer)
        .expect("Decryption with correct password failed");
    assert_eq!(original_data.as_ref(), decoded_output_buffer.as_slice()); // Corrected variable name

    // 3. Decode with wrong password (should fail)
    let mut input_reader_decode_wrong = Cursor::new(encrypted_output_buffer.clone());
    let header_wrong = Headers::from_reader(&mut input_reader_decode_wrong).expect("Failed to read header for decode (wrong pass)"); // Read header
    let mut decrypted_output_buffer_wrong = Vec::new();
    let result_wrong_password = decode(header_wrong, &mut input_reader_decode_wrong, Some(wrong_password), &mut decrypted_output_buffer_wrong); // Updated call
    assert!(result_wrong_password.is_err());
    assert_eq!(result_wrong_password.unwrap_err().to_string(), "aead::Error");

    // 4. Decode without password (should fail if encrypted)
    let mut input_reader_decode_no_pass = Cursor::new(encrypted_output_buffer);
    let header_no_pass = Headers::from_reader(&mut input_reader_decode_no_pass).expect("Failed to read header for decode (no pass)"); // Read header
    let mut decrypted_output_buffer_no_pass = Vec::new();
    let result_no_password = decode(header_no_pass, &mut input_reader_decode_no_pass, None, &mut decrypted_output_buffer_no_pass); // Updated call
    assert!(result_no_password.is_err());
    assert_eq!(result_no_password.unwrap_err().to_string(), "Error: File is encrypted, password required.");
}
