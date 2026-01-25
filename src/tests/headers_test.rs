#[cfg(test)]
mod tests {
    use crate::constants::{MAGIC_BYTES, VERSION};
    use crate::flags::{flip_encrypted, flip_stored_raw, is_encrypted, is_stored_raw};
    use crate::headers::{write_header, Headers};
    use std::io::Cursor;

    #[test]
    fn headers_round_trip() {
        let mut original = write_header(
            b"test".len() as u64,
            crate::checksum::adler32(b"test".as_ref()),
            "testfile.txt",
        );
        original.tree = crate::huffman::tree::Node {
            weight: 0,
            symbol: Some(b'a'),
            left: None,
            right: None,
        };

        let bytes = original.clone().to_bytes();
        let reconstructed = match Headers::from_reader(&mut std::io::Cursor::new(&bytes)) {
            Ok(header) => header,
            Err(e) => {
                panic!("Error deserializing headers: {e}");
            }
        };

        assert_eq!(original.magic_bytes, reconstructed.magic_bytes);
        assert_eq!(original.version, reconstructed.version);
        assert_eq!(original.flags, reconstructed.flags);
        assert_eq!(original.original_size, reconstructed.original_size);
        assert_eq!(original.compressed_size, reconstructed.compressed_size);
        assert_eq!(
            original.original_file_name,
            reconstructed.original_file_name
        );
        assert_eq!(original.salt, reconstructed.salt);
        assert_eq!(original.iv, reconstructed.iv);
        assert_eq!(original.tag, reconstructed.tag);
        assert_eq!(original.padding_bits, reconstructed.padding_bits);
        assert_eq!(original.checksum, reconstructed.checksum);
    }

    #[test]
    fn test_header_default_values() {
        let header = Headers::default();

        assert_eq!(header.magic_bytes, MAGIC_BYTES);
        assert_eq!(header.version, VERSION);
        assert_eq!(header.original_size, 0);
        assert_eq!(header.compressed_size, 0);
        assert_eq!(header.payload_actual_size, 0);
        assert_eq!(header.flags, 0);
        assert_eq!(header.padding_bits, 0);
        assert_eq!(header.checksum, 0);
        assert_eq!(header.original_file_name, "");
        assert_eq!(header.salt, [0u8; 16]);
        assert_eq!(header.iv, [0u8; 12]);
        assert_eq!(header.tag, [0u8; 16]);
    }

    #[test]
    fn test_header_with_long_filename() {
        let long_filename = "this_is_a_very_long_filename_that_tests_the_header_serialization_with_extended_length.txt";
        let header = write_header(1024, 12345678, long_filename);

        let bytes = header.clone().to_bytes();
        let reconstructed = Headers::from_reader(&mut Cursor::new(&bytes))
            .expect("Failed to deserialize header with long filename");

        assert_eq!(header.original_file_name, reconstructed.original_file_name);
        assert_eq!(reconstructed.original_file_name, long_filename);
    }

    #[test]
    fn test_header_with_special_characters() {
        let special_filename = "test-file_123.特殊字符!@#$%^&()";
        let header = write_header(512, 87654321, special_filename);

        let bytes = header.clone().to_bytes();
        let reconstructed = Headers::from_reader(&mut Cursor::new(&bytes))
            .expect("Failed to deserialize header with special characters");

        assert_eq!(header.original_file_name, reconstructed.original_file_name);
        assert_eq!(reconstructed.original_file_name, special_filename);
    }

    #[test]
    fn test_header_flags_operations() {
        let mut header = Headers::default();

        // Initially no flags should be set
        assert!(!is_stored_raw(header.flags));
        assert!(!is_encrypted(header.flags));

        // Set stored_raw flag
        flip_stored_raw(&mut header.flags);
        assert!(is_stored_raw(header.flags));
        assert!(!is_encrypted(header.flags));

        // Set encrypted flag
        flip_encrypted(&mut header.flags);
        assert!(is_stored_raw(header.flags));
        assert!(is_encrypted(header.flags));

        // Verify both flags are set (bitwise OR)
        assert!(header.flags != 0);
    }

    #[test]
    fn test_header_size_consistency() {
        let header = write_header(1024, 12345, "size_test.txt");
        let original_size = header.original_size;
        let compressed_size = header.compressed_size;
        let bytes = header.to_bytes();

        // Header should have consistent serialized size
        assert!(!bytes.is_empty());

        // Try to read it back
        let reconstructed = Headers::from_reader(&mut Cursor::new(&bytes))
            .expect("Failed to read back header for size consistency test");

        assert_eq!(original_size, reconstructed.original_size);
        assert_eq!(compressed_size, reconstructed.compressed_size);
    }

    #[test]
    fn test_header_with_zero_sizes() {
        let header = write_header(0, 0, "empty.txt");

        let bytes = header.clone().to_bytes();
        let reconstructed = Headers::from_reader(&mut Cursor::new(&bytes))
            .expect("Failed to deserialize header with zero sizes");

        assert_eq!(header.original_size, 0);
        assert_eq!(header.compressed_size, 0);
        assert_eq!(reconstructed.original_size, 0);
        assert_eq!(reconstructed.compressed_size, 0);
    }

    #[test]
    fn test_header_checksum_preservation() {
        let test_checksum = 0xDEADBEEF;
        let header = write_header(256, test_checksum, "checksum_test.txt");

        let bytes = header.clone().to_bytes();
        let reconstructed = Headers::from_reader(&mut Cursor::new(&bytes))
            .expect("Failed to deserialize header with checksum");

        assert_eq!(header.checksum, test_checksum);
        assert_eq!(reconstructed.checksum, test_checksum);
    }

    #[test]
    fn test_header_padding_bits_range() {
        let mut header = Headers::default();

        // Test valid padding bits (0-7)
        for padding in 0..8 {
            header.padding_bits = padding;
            let bytes = header.clone().to_bytes();
            let reconstructed = Headers::from_reader(&mut Cursor::new(&bytes))
                .expect("Failed to deserialize header with padding bits");

            assert_eq!(reconstructed.padding_bits, padding);
        }
    }
}
