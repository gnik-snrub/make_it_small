#[cfg(test)]
mod tests {
    use crate::crypto::DEFAULT_CHUNK_SIZE;
    use crate::headers::Headers;
    use crate::huffman::{decoder::decode, encoder::encode};
    use std::io::{Cursor, Seek, SeekFrom};

    #[test]
    fn test_basic_unencrypted_roundtrip() {
        let original_content =
            b"Hello, this is a test string for roundtrip compression and decompression.";
        let mut input_reader = Cursor::new(original_content.to_vec());
        let mut encoded_output_buffer = Cursor::new(Vec::new());

        // 1. Encode
        let encode_info = encode(
            &mut input_reader,
            "test_file.txt",
            None, // No encryption
            &mut encoded_output_buffer,
            DEFAULT_CHUNK_SIZE,
        )
        .expect("Encoding failed");

        // Rewind encoded buffer to read from the beginning
        encoded_output_buffer
            .seek(SeekFrom::Start(0))
            .expect("Failed to rewind encoded buffer");

        // 2. Read header first
        let header =
            Headers::from_reader(&mut encoded_output_buffer).expect("Failed to read header");

        let mut decoded_output_buffer = Cursor::new(Vec::new());

        // 3. Decode
        let decode_info = decode(
            header,
            &mut encoded_output_buffer,
            None, // No decryption
            &mut decoded_output_buffer,
            DEFAULT_CHUNK_SIZE,
        )
        .expect("Decoding failed");

        // Verify that original_size matches
        assert_eq!(encode_info.original_size, original_content.len() as u64);
        assert_eq!(decode_info.original_size, original_content.len() as u64);

        // Verify content
        assert_eq!(
            decoded_output_buffer.into_inner(),
            original_content.to_vec()
        );
    }

    #[test]
    fn test_basic_encrypted_roundtrip() {
        let original_content =
            b"This is some secret data that should be encrypted and then decrypted successfully.";
        let password = "supersecretpassword";
        let mut input_reader = Cursor::new(original_content.to_vec());
        let mut encoded_output_buffer = Cursor::new(Vec::new());

        // 1. Encode with encryption
        let encode_info = encode(
            &mut input_reader,
            "secret_file.txt",
            Some(password), // With encryption
            &mut encoded_output_buffer,
            DEFAULT_CHUNK_SIZE,
        )
        .expect("Encoding failed");

        // Rewind encoded buffer to read from the beginning
        encoded_output_buffer
            .seek(SeekFrom::Start(0))
            .expect("Failed to rewind encoded buffer");

        // 2. Read header first
        let header =
            Headers::from_reader(&mut encoded_output_buffer).expect("Failed to read header");

        let mut decoded_output_buffer = Cursor::new(Vec::new());

        // 3. Decode with decryption
        let decode_info = decode(
            header,
            &mut encoded_output_buffer,
            Some(password), // With decryption
            &mut decoded_output_buffer,
            DEFAULT_CHUNK_SIZE,
        )
        .expect("Decoding failed");

        // Verify that original_size matches
        assert_eq!(encode_info.original_size, original_content.len() as u64);
        assert_eq!(decode_info.original_size, original_content.len() as u64);

        // Verify content
        assert_eq!(
            decoded_output_buffer.into_inner(),
            original_content.to_vec()
        );
    }

    #[test]
    fn test_large_data_roundtrip() {
        let original_content: Vec<u8> = (0..100000).map(|i| (i % 256) as u8).collect(); // 100KB of repeating data
        let password = "anothersecretpassword";
        let mut input_reader = Cursor::new(original_content.to_vec());
        let mut encoded_output_buffer = Cursor::new(Vec::new());

        // 1. Encode with encryption
        let encode_info = encode(
            &mut input_reader,
            "large_file.bin",
            Some(password), // With encryption
            &mut encoded_output_buffer,
            DEFAULT_CHUNK_SIZE,
        )
        .expect("Encoding failed for large data");

        // Rewind encoded buffer to read from the beginning
        encoded_output_buffer
            .seek(SeekFrom::Start(0))
            .expect("Failed to rewind encoded buffer");

        // 2. Read header first
        let header = Headers::from_reader(&mut encoded_output_buffer)
            .expect("Failed to read header for large data");

        let mut decoded_output_buffer = Cursor::new(Vec::new());

        // 3. Decode with decryption
        let decode_info = decode(
            header,
            &mut encoded_output_buffer,
            Some(password), // With decryption
            &mut decoded_output_buffer,
            DEFAULT_CHUNK_SIZE,
        )
        .expect("Decoding failed for large data");

        // Verify that original_size matches
        assert_eq!(encode_info.original_size, original_content.len() as u64);
        assert_eq!(decode_info.original_size, original_content.len() as u64);

        // Verify content
        assert_eq!(decoded_output_buffer.into_inner(), original_content);
    }

    #[test]
    fn test_empty_file_roundtrip() {
        let original_content = b""; // Empty content
        let password = "emptyfilepassword";
        let mut input_reader = Cursor::new(original_content.to_vec());
        let mut encoded_output_buffer = Cursor::new(Vec::new());

        // 1. Encode with encryption
        let encode_info = encode(
            &mut input_reader,
            "empty_file.txt",
            Some(password), // With encryption
            &mut encoded_output_buffer,
            DEFAULT_CHUNK_SIZE,
        )
        .expect("Encoding failed for empty file");

        // Rewind encoded buffer to read from the beginning
        encoded_output_buffer
            .seek(SeekFrom::Start(0))
            .expect("Failed to rewind encoded buffer");

        // 2. Read header first
        let header = Headers::from_reader(&mut encoded_output_buffer)
            .expect("Failed to read header for empty file");

        let mut decoded_output_buffer = Cursor::new(Vec::new());

        // 3. Decode with decryption
        let decode_info = decode(
            header,
            &mut encoded_output_buffer,
            Some(password), // With decryption
            &mut decoded_output_buffer,
            DEFAULT_CHUNK_SIZE,
        )
        .expect("Decoding failed for empty file");

        // Verify that original_size matches
        assert_eq!(encode_info.original_size, original_content.len() as u64);
        assert_eq!(decode_info.original_size, original_content.len() as u64);

        // Verify content
        assert_eq!(decoded_output_buffer.into_inner(), original_content);
    }
}
