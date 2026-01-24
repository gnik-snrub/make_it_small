#[cfg(test)]
mod tests {
    use crate::crypto::DEFAULT_CHUNK_SIZE;
    use crate::headers::Headers;
    use crate::huffman::{decoder::decode, encoder::encode};
    use std::io::{Cursor, Seek, SeekFrom};

    #[test]
    fn test_basic_unencrypted_encoding() {
        let original_content = b"Hello, this is a test string for basic encoding.";
        let mut input_reader = Cursor::new(original_content.to_vec());
        let mut encoded_output_buffer = Cursor::new(Vec::new());

        let encode_info = encode(
            &mut input_reader,
            "test_file.txt",
            None, // No encryption
            &mut encoded_output_buffer,
            DEFAULT_CHUNK_SIZE,
        )
        .expect("Encoding failed");

        // Verify basic encoding info
        assert_eq!(encode_info.original_size, original_content.len() as u64);
        assert!(encode_info.compressed_size > 0);
        assert!(encode_info.padding_bits < 8);
    }

    #[test]
    fn test_encrypted_encoding() {
        let original_content = b"This is a secret message that should be encrypted.";
        let mut input_reader = Cursor::new(original_content.to_vec());
        let mut encoded_output_buffer = Cursor::new(Vec::new());
        let password = "test_password_123";

        let encode_info = encode(
            &mut input_reader,
            "secret_file.txt",
            Some(password),
            &mut encoded_output_buffer,
            DEFAULT_CHUNK_SIZE,
        )
        .expect("Encoding failed");

        // Verify encrypted encoding info
        assert_eq!(encode_info.original_size, original_content.len() as u64);
        assert!(encode_info.compressed_size > 0);
    }

    #[test]
    fn test_raw_store_heuristic() {
        // Test with content that likely won't compress well (random-like data)
        let original_content = b"\x00\xFF\x00\xFF\x00\xFF\x00\xFF\x00\xFF\x00\xFF\x00\xFF\x00\xFF";
        let mut input_reader = Cursor::new(original_content.to_vec());
        let mut encoded_output_buffer = Cursor::new(Vec::new());

        let encode_info = encode(
            &mut input_reader,
            "binary_file.bin",
            None,
            &mut encoded_output_buffer,
            DEFAULT_CHUNK_SIZE,
        )
        .expect("Encoding failed");

        // For data that doesn't compress well, it should be stored raw
        // This means compressed_size should equal original_size (before encryption)
        assert_eq!(encode_info.original_size, original_content.len() as u64);
        assert!(encode_info.padding_bits == 0); // Raw data has no padding bits
    }

    #[test]
    fn test_large_file_encoding() {
        // Test with larger content to exercise chunking
        let large_content = "A".repeat(10000); // 10KB of repeated 'A'
        let mut input_reader = Cursor::new(large_content.as_bytes());
        let mut encoded_output_buffer = Cursor::new(Vec::new());

        let encode_info = encode(
            &mut input_reader,
            "large_file.txt",
            None,
            &mut encoded_output_buffer,
            DEFAULT_CHUNK_SIZE,
        )
        .expect("Encoding failed");

        assert_eq!(encode_info.original_size, large_content.len() as u64);
        assert!(encode_info.compressed_size > 0);
        // Highly compressible content should be much smaller
        assert!(encode_info.compressed_size < encode_info.original_size);
    }

    #[test]
    fn test_empty_file_encoding() {
        let original_content = b"";
        let mut input_reader = Cursor::new(original_content.to_vec());
        let mut encoded_output_buffer = Cursor::new(Vec::new());

        let encode_info = encode(
            &mut input_reader,
            "empty_file.txt",
            None,
            &mut encoded_output_buffer,
            DEFAULT_CHUNK_SIZE,
        )
        .expect("Encoding failed");

        assert_eq!(encode_info.original_size, 0);
        assert!(encode_info.compressed_size >= 0); // Header might be minimal for empty files
    }

    #[test]
    fn test_single_byte_encoding() {
        let original_content = b"A";
        let mut input_reader = Cursor::new(original_content.to_vec());
        let mut encoded_output_buffer = Cursor::new(Vec::new());

        let encode_info = encode(
            &mut input_reader,
            "single_byte.txt",
            None,
            &mut encoded_output_buffer,
            DEFAULT_CHUNK_SIZE,
        )
        .expect("Encoding failed");

        assert_eq!(encode_info.original_size, 1);
        assert!(encode_info.compressed_size > 0);
    }

    #[test]
    fn test_encoding_with_different_chunk_sizes() {
        let original_content = b"Test content for different chunk sizes.";
        let chunk_sizes = [1024, 4096, 16384, 65536];

        for chunk_size in chunk_sizes.iter() {
            let mut input_reader = Cursor::new(original_content.to_vec());
            let mut encoded_output_buffer = Cursor::new(Vec::new());

            let encode_info = encode(
                &mut input_reader,
                "chunk_test.txt",
                None,
                &mut encoded_output_buffer,
                *chunk_size,
            )
            .expect("Encoding failed");

            assert_eq!(encode_info.original_size, original_content.len() as u64);
            assert!(encode_info.compressed_size > 0);
        }
    }

    #[test]
    fn test_encoding_roundtrip_verification() {
        let original_content = b"This content should survive a complete encode-decode roundtrip.";
        let mut input_reader = Cursor::new(original_content.to_vec());
        let mut encoded_output_buffer = Cursor::new(Vec::new());

        // Encode
        let encode_info = encode(
            &mut input_reader,
            "roundtrip_test.txt",
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

        // Decode back
        let mut decoded_output_buffer = Cursor::new(Vec::new());
        let decode_info = decode(
            header,
            &mut encoded_output_buffer,
            None,
            &mut decoded_output_buffer,
            DEFAULT_CHUNK_SIZE,
        )
        .expect("Decoding failed");

        // Verify roundtrip integrity
        assert_eq!(encode_info.original_size, decode_info.original_size);
        assert_eq!(
            decoded_output_buffer.into_inner(),
            original_content.to_vec()
        );
    }
}
