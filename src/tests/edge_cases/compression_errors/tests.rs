//! Compression edge case tests
//!
//! Tests edge cases and error conditions in compression operations.
//! Ensures robustness when handling invalid inputs and error scenarios.

use crate::compress::CompressionBuilder;
use crate::error::{CompressionError, MismallError};

#[cfg(test)]
mod tests {
    use crate::compress::CompressionBuilder;
    use crate::error::{CompressionError, MismallError};

    #[test]
    fn test_compress_nonexistent_file() {
        let builder = CompressionBuilder::new("nonexistent_file.txt");
        let result = builder.compress();

        assert!(result.is_err());

        match result.unwrap_err() {
            MismallError::Compression {
                error: CompressionError::InputRead(_),
                ..
            } => {
                // Expected: Should fail with input read error
            }
            other => panic!("Expected InputRead error, got {:?}", other),
        }
    }

    #[test]
    fn test_compress_empty_file() {
        let builder = CompressionBuilder::new("");
        let result = builder.compress();

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
    fn test_compress_invalid_chunk_size_too_small() {
        let builder = CompressionBuilder::new("test.txt").with_chunk_size(1023); // Below minimum (64KB)
        let result = builder.compress();

        assert!(result.is_err());

        match result.unwrap_err() {
            MismallError::Compression {
                error: CompressionError::InvalidChunkSize(_),
                ..
            } => {
                // Expected: Should fail with invalid chunk size error
            }
            other => panic!("Expected InvalidChunkSize error, got {:?}", other),
        }
    }

    #[test]
    fn test_compress_invalid_chunk_size_too_large() {
        let builder = CompressionBuilder::new("test.txt").with_chunk_size(1024 * 1024 * 1024 + 1); // Above maximum (1GB)
        let result = builder.compress();

        assert!(result.is_err());

        match result.unwrap_err() {
            MismallError::Compression {
                error: CompressionError::InvalidChunkSize(_),
                ..
            } => {
                // Expected: Should fail with invalid chunk size error
            }
            other => panic!("Expected InvalidChunkSize error, got {:?}", other),
        }
    }

    #[test]
    fn test_compress_with_invalid_password() {
        let builder = CompressionBuilder::new("test.txt").with_password("\x00\x01\x02"); // Invalid password format
        let result = builder.compress();

        // Should handle invalid password gracefully
        assert!(result.is_err());
    }

    #[test]
    fn test_compress_file_permission_denied() {
        // Try to compress a directory (should fail)
        let builder = CompressionBuilder::new(".");
        let result = builder.compress();

        assert!(result.is_err());
    }

    #[test]
    fn test_error_context_preservation() {
        let builder = CompressionBuilder::new("test.txt");
        let result = builder
            .with_context(crate::error::ErrorContext::new("test_operation"))
            .compress();

        assert!(result.is_err());
    }
}
