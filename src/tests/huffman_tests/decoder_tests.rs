#[cfg(test)]
mod tests {
    use crate::constants::{MAGIC_BYTES, VERSION};
    use crate::crypto::DEFAULT_CHUNK_SIZE;
    use crate::headers::Headers;
    use crate::huffman::{decoder::decode, encoder::encode};
    use std::io::{Cursor, Seek, SeekFrom};

    #[test]
    fn test_basic_unencrypted_decoding() {
        let original_content = b"Hello, this is a test string for basic decoding.";
        let mut input_reader = Cursor::new(original_content.to_vec());
        let mut encoded_output_buffer = Cursor::new(Vec::new());

        // First encode the content
        let encode_info = encode(
            &mut input_reader,
            "test_file.txt",
            None,
            &mut encoded_output_buffer,
            DEFAULT_CHUNK_SIZE,
        )
        .expect("Encoding failed");

        // Rewind and read header
        encoded_output_buffer
            .seek(SeekFrom::Start(0))
            .expect("Failed to rewind");
        let header =
            Headers::from_reader(&mut encoded_output_buffer).expect("Failed to read header");

        // Now decode it
        let mut decoded_output_buffer = Cursor::new(Vec::new());
        let decode_info = decode(
            header,
            &mut encoded_output_buffer,
            None,
            &mut decoded_output_buffer,
            DEFAULT_CHUNK_SIZE,
        )
        .expect("Decoding failed");

        // Verify decoding info
        assert_eq!(decode_info.original_size, original_content.len() as u64);
        assert_eq!(decode_info.original_file_name, "test_file.txt");
        assert_eq!(
            decoded_output_buffer.into_inner(),
            original_content.to_vec()
        );
    }

    #[test]
    fn test_encrypted_decoding() {
        let original_content =
            b"This is a secret message that should be encrypted and then decrypted.";
        let mut input_reader = Cursor::new(original_content.to_vec());
        let mut encoded_output_buffer = Cursor::new(Vec::new());
        let password = "test_password_123";

        // Encode with encryption
        let encode_info = encode(
            &mut input_reader,
            "secret_file.txt",
            Some(password),
            &mut encoded_output_buffer,
            DEFAULT_CHUNK_SIZE,
        )
        .expect("Encoding failed");

        // Rewind and read header
        encoded_output_buffer
            .seek(SeekFrom::Start(0))
            .expect("Failed to rewind");
        let header =
            Headers::from_reader(&mut encoded_output_buffer).expect("Failed to read header");

        // Decode with correct password
        let mut decoded_output_buffer = Cursor::new(Vec::new());
        let decode_info = decode(
            header,
            &mut encoded_output_buffer,
            Some(password),
            &mut decoded_output_buffer,
            DEFAULT_CHUNK_SIZE,
        )
        .expect("Decoding failed");

        // Verify decrypted content
        assert_eq!(decode_info.original_size, original_content.len() as u64);
        assert_eq!(decode_info.original_file_name, "secret_file.txt");
        assert_eq!(
            decoded_output_buffer.into_inner(),
            original_content.to_vec()
        );
    }

    #[test]
    fn test_encrypted_decoding_wrong_password() {
        let original_content = b"This is a secret message.";
        let mut input_reader = Cursor::new(original_content.to_vec());
        let mut encoded_output_buffer = Cursor::new(Vec::new());
        let correct_password = "correct_password";
        let wrong_password = "wrong_password";

        // Encode with correct password
        encode(
            &mut input_reader,
            "secret_file.txt",
            Some(correct_password),
            &mut encoded_output_buffer,
            DEFAULT_CHUNK_SIZE,
        )
        .expect("Encoding failed");

        // Rewind and read header
        encoded_output_buffer
            .seek(SeekFrom::Start(0))
            .expect("Failed to rewind");
        let header =
            Headers::from_reader(&mut encoded_output_buffer).expect("Failed to read header");

        // Try to decode with wrong password
        let mut decoded_output_buffer = Cursor::new(Vec::new());
        let result = decode(
            header,
            &mut encoded_output_buffer,
            Some(wrong_password),
            &mut decoded_output_buffer,
            DEFAULT_CHUNK_SIZE,
        );

        // Should fail with decryption error
        assert!(result.is_err());
    }

    #[test]
    fn test_raw_store_decoding() {
        // Test with content that will be stored raw (not compressed)
        let original_content = b"\x00\xFF\x00\xFF\x00\xFF\x00\xFF\x00\xFF\x00\xFF\x00\xFF\x00\xFF";
        let mut input_reader = Cursor::new(original_content.to_vec());
        let mut encoded_output_buffer = Cursor::new(Vec::new());

        // Encode (should store raw)
        let encode_info = encode(
            &mut input_reader,
            "binary_file.bin",
            None,
            &mut encoded_output_buffer,
            DEFAULT_CHUNK_SIZE,
        )
        .expect("Encoding failed");

        // Rewind and read header
        encoded_output_buffer
            .seek(SeekFrom::Start(0))
            .expect("Failed to rewind");
        let header =
            Headers::from_reader(&mut encoded_output_buffer).expect("Failed to read header");

        // Decode
        let mut decoded_output_buffer = Cursor::new(Vec::new());
        let decode_info = decode(
            header,
            &mut encoded_output_buffer,
            None,
            &mut decoded_output_buffer,
            DEFAULT_CHUNK_SIZE,
        )
        .expect("Decoding failed");

        // Verify raw data integrity
        assert_eq!(decode_info.original_size, original_content.len() as u64);
        assert_eq!(
            decoded_output_buffer.into_inner(),
            original_content.to_vec()
        );
    }

    #[test]
    fn test_empty_file_decoding() {
        let original_content = b"";
        let mut input_reader = Cursor::new(original_content.to_vec());
        let mut encoded_output_buffer = Cursor::new(Vec::new());

        // Encode empty file
        let encode_info = encode(
            &mut input_reader,
            "empty_file.txt",
            None,
            &mut encoded_output_buffer,
            DEFAULT_CHUNK_SIZE,
        )
        .expect("Encoding failed");

        // Rewind and read header
        encoded_output_buffer
            .seek(SeekFrom::Start(0))
            .expect("Failed to rewind");
        let header =
            Headers::from_reader(&mut encoded_output_buffer).expect("Failed to read header");

        // Decode empty file
        let mut decoded_output_buffer = Cursor::new(Vec::new());
        let decode_info = decode(
            header,
            &mut encoded_output_buffer,
            None,
            &mut decoded_output_buffer,
            DEFAULT_CHUNK_SIZE,
        )
        .expect("Decoding failed");

        // Verify empty file handling
        assert_eq!(decode_info.original_size, 0);
        assert_eq!(decode_info.original_file_name, "empty_file.txt");
        assert_eq!(decoded_output_buffer.into_inner(), Vec::<u8>::new());
    }

    #[test]
    fn test_invalid_magic_bytes() {
        // Create a header with invalid magic bytes
        let mut invalid_header = Headers::default();
        invalid_header.magic_bytes = [0x00, 0x00, 0x00, 0x00]; // Invalid magic
        invalid_header.version = VERSION;

        let mut fake_payload = Cursor::new(b"fake payload");
        let mut output_buffer = Cursor::new(Vec::new());

        let result = decode(
            invalid_header,
            &mut fake_payload,
            None,
            &mut output_buffer,
            DEFAULT_CHUNK_SIZE,
        );

        // Should fail with invalid magic bytes error
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Not a valid .small file"));
    }

    #[test]
    fn test_invalid_version() {
        // Create a header with invalid version
        let mut invalid_header = Headers::default();
        invalid_header.magic_bytes = MAGIC_BYTES;
        invalid_header.version = 0xFF; // Invalid version

        let mut fake_payload = Cursor::new(b"fake payload");
        let mut output_buffer = Cursor::new(Vec::new());

        let result = decode(
            invalid_header,
            &mut fake_payload,
            None,
            &mut output_buffer,
            DEFAULT_CHUNK_SIZE,
        );

        // Should fail with incorrect version error
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Incorrect version"));
    }

    #[test]
    fn test_missing_password_for_encrypted_file() {
        let original_content = b"Encrypted content";
        let mut input_reader = Cursor::new(original_content.to_vec());
        let mut encoded_output_buffer = Cursor::new(Vec::new());

        // Encode with encryption
        encode(
            &mut input_reader,
            "encrypted_file.txt",
            Some("password"),
            &mut encoded_output_buffer,
            DEFAULT_CHUNK_SIZE,
        )
        .expect("Encoding failed");

        // Rewind and read header
        encoded_output_buffer
            .seek(SeekFrom::Start(0))
            .expect("Failed to rewind");
        let header =
            Headers::from_reader(&mut encoded_output_buffer).expect("Failed to read header");

        // Try to decode without password
        let mut decoded_output_buffer = Cursor::new(Vec::new());
        let result = decode(
            header,
            &mut encoded_output_buffer,
            None, // No password provided
            &mut decoded_output_buffer,
            DEFAULT_CHUNK_SIZE,
        );

        // Should fail with password required error
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("password required"));
    }

    #[test]
    fn test_decoding_with_different_chunk_sizes() {
        let original_content = b"Test content for different chunk sizes during decoding.";
        let chunk_sizes = [1024, 4096, 16384, 65536];

        for chunk_size in chunk_sizes.iter() {
            let mut input_reader = Cursor::new(original_content.to_vec());
            let mut encoded_output_buffer = Cursor::new(Vec::new());

            // Encode
            let encode_info = encode(
                &mut input_reader,
                "chunk_test.txt",
                None,
                &mut encoded_output_buffer,
                *chunk_size,
            )
            .expect("Encoding failed");

            // Rewind and read header
            encoded_output_buffer
                .seek(SeekFrom::Start(0))
                .expect("Failed to rewind");
            let header =
                Headers::from_reader(&mut encoded_output_buffer).expect("Failed to read header");

            // Decode with same chunk size
            let mut decoded_output_buffer = Cursor::new(Vec::new());
            let decode_info = decode(
                header,
                &mut encoded_output_buffer,
                None,
                &mut decoded_output_buffer,
                *chunk_size,
            )
            .expect("Decoding failed");

            // Verify integrity
            assert_eq!(encode_info.original_size, decode_info.original_size);
            assert_eq!(
                decoded_output_buffer.into_inner(),
                original_content.to_vec()
            );
        }
    }
}
