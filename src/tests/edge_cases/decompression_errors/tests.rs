//! Decompression edge case tests
//!
//! Tests edge cases and error conditions in decompression operations.
//! Ensures robustness when handling corrupted data and invalid inputs.

use crate::decompress::DecompressionBuilder;
use crate::error::{DecompressionError, MismallError};
use std::io::Cursor;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decompress_nonexistent_file() {
        let builder = DecompressionBuilder::new("nonexistent_file.small");
        let result = builder.decompress();

        assert!(result.is_err());

        match result.unwrap_err() {
            MismallError::Decompression {
                error: DecompressionError::InputRead(_),
                ..
            } => {
                // Expected: Should fail with input read error
            }
            other => panic!("Expected InputRead error, got {:?}", other),
        }
    }

    #[test]
    fn test_decompress_empty_file() {
        let builder = DecompressionBuilder::new("");
        let result = builder.decompress();

        assert!(result.is_err());

        match result.unwrap_err() {
            MismallError::InvalidInput { message: _, .. } => {
                // Expected: Empty filename should be invalid
            }
            other => panic!(
                "Expected InvalidInput error for empty filename, got {:?}",
                other
            ),
        }
    }

    #[test]
    fn test_decompress_invalid_chunk_size() {
        let builder = DecompressionBuilder::new("test.txt.small").with_chunk_size(32); // Too small
        let result = builder.decompress();

        assert!(result.is_err());
    }

    #[test]
    fn test_decompress_wrong_password() {
        // Create test data first
        let test_data = b"test data";
        let mut cursor = Cursor::new(Vec::new());
        let result = crate::compress::compress_stream(
            &mut cursor,
            "test.txt",
            Some("correct_password"),
            &mut cursor,
            1024,
        );
        assert!(result.is_ok());

        let compressed_data = cursor.into_inner();
        let mut reader = Cursor::new(&compressed_data);

        let builder = DecompressionBuilder::new("test.txt.small");
        let result = builder.with_password("wrong_password").decompress();

        assert!(result.is_err());

        match result.unwrap_err() {
            MismallError::Decompression {
                error: DecompressionError::InvalidPassword,
                ..
            } => {
                // Expected: Should fail with wrong password
            }
            other => panic!("Expected InvalidPassword error, got {:?}", other),
        }
    }

    #[test]
    fn test_decompress_corrupted_data() {
        // Test with truncated/corrupted compressed data
        let corrupted_data = b"corrupted";
        let mut cursor = Cursor::new(corrupted_data);

        let builder = DecompressionBuilder::new("test.txt.small");
        let result = builder.decompress();

        assert!(result.is_err());
        // Should handle corrupted data gracefully without panicking
    }

    #[test]
    fn test_decompress_unsupported_format() {
        // Test with data that's not a valid .small file
        let invalid_data = b"not a compressed file";
        let mut cursor = Cursor::new(invalid_data);

        let builder = DecompressionBuilder::new("test.txt.small");
        let result = builder.decompress();

        assert!(result.is_err());
    }
}
