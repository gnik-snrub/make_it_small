#[cfg(test)]
mod tests {
    use super::*;
    use crate::huffman::{encoder::encode, decoder::decode};
    use crate::headers::Headers;
    use crate::crypto::DEFAULT_CHUNK_SIZE;
    use std::io::{Cursor, Seek, SeekFrom};

    #[test]
    fn test_basic_unencrypted_roundtrip() {
        let original_content = b"Hello, this is a test string for roundtrip compression and decompression.";
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
        encoded_output_buffer.seek(SeekFrom::Start(0)).expect("Failed to rewind encoded buffer");

        // 2. Read header first
        let header = Headers::from_reader(&mut encoded_output_buffer).expect("Failed to read header");
        
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
        assert_eq!(decoded_output_buffer.into_inner(), original_content.to_vec());
    }

    #[test]
    fn test_basic_encrypted_roundtrip() {
        let original_content = b"This is some secret data that should be encrypted and then decrypted successfully.";
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
        encoded_output_buffer.seek(SeekFrom::Start(0)).expect("Failed to rewind encoded buffer");

        // 2. Read header first
        let header = Headers::from_reader(&mut encoded_output_buffer).expect("Failed to read header");
        
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
        assert_eq!(decoded_output_buffer.into_inner(), original_content.to_vec());
    }
}
