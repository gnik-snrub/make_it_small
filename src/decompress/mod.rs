//! High-level decompression API
//!
//! This module provides a simple, high-level interface for decompressing files
//! and data streams that were compressed with mismall.

pub mod builder;
mod simple;

pub use builder::DecompressionBuilder;
pub use simple::{decompress_file, decompress_stream};

/// Result of a decompression operation
#[derive(Debug, Clone)]
pub struct DecompressionResult {
    /// Original filename extracted from header
    pub original_filename: String,
    /// Size of decompressed data in bytes
    pub original_size: u64,
    /// Checksum of original data for integrity verification
    pub checksum: u32,
    /// Whether the file was encrypted
    pub encrypted: bool,
}

impl DecompressionResult {
    /// Create a new decompression result
    pub fn new(
        original_filename: String,
        original_size: u64,
        checksum: u32,
        encrypted: bool,
    ) -> Self {
        Self {
            original_filename,
            original_size,
            checksum,
            encrypted,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decompression_result_creation() {
        let result = DecompressionResult::new("document.txt".to_string(), 1000, 0x12345678, false);

        assert_eq!(result.original_filename, "document.txt");
        assert_eq!(result.original_size, 1000);
        assert_eq!(result.checksum, 0x12345678);
        assert!(!result.encrypted);
    }
}
