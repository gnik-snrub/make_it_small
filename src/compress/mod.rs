//! High-level compression API
//!
//! This module provides a simple, high-level interface for compressing files
//! and data streams using Huffman coding with optional AES-256-GCM encryption.

pub mod builder;
mod simple;

pub use builder::CompressionBuilder;
pub use simple::{compress_file, compress_stream};

use crate::error::{CompressionError, Result};
use crate::huffman::encoder::EncodeInfo;

/// Result of a compression operation
#[derive(Debug, Clone)]
pub struct CompressionResult {
    /// Original uncompressed size in bytes
    pub original_size: u64,
    /// Final compressed size in bytes (including encryption overhead if applicable)
    pub compressed_size: u64,
    /// Number of padding bits added to complete final byte (0-7)
    pub padding_bits: u8,
    /// Compression ratio as percentage (0-100)
    pub compression_ratio: f32,
    /// Whether encryption was applied
    pub encrypted: bool,
    /// Filename used for compression
    pub filename: String,
}

impl CompressionResult {
    /// Create a new compression result
    pub fn new(encode_info: EncodeInfo, filename: String, encrypted: bool) -> Self {
        let compression_ratio = if encode_info.original_size > 0 {
            ((encode_info.compressed_size as f32 / encode_info.original_size as f32) * 100.0)
                .round()
                .clamp(0.0, 100.0)
        } else {
            100.0
        };

        Self {
            original_size: encode_info.original_size,
            compressed_size: encode_info.compressed_size,
            padding_bits: encode_info.padding_bits,
            compression_ratio,
            encrypted,
            filename,
        }
    }

    /// Get size savings in bytes
    pub fn bytes_saved(&self) -> i64 {
        self.compressed_size as i64 - self.original_size as i64
    }

    /// Get size savings as percentage
    pub fn savings_percentage(&self) -> f32 {
        100.0 - self.compression_ratio
    }

    /// Check if compression was beneficial
    pub fn is_beneficial(&self) -> bool {
        self.compressed_size < self.original_size
    }
}

impl From<EncodeInfo> for CompressionResult {
    fn from(encode_info: EncodeInfo) -> Self {
        Self::new(encode_info, "unknown".to_string(), false)
    }
}

/// Validate chunk size parameter
pub fn validate_chunk_size(chunk_size: usize) -> Result<()> {
    const MIN_CHUNK_SIZE: usize = 64 * 1024; // 64KB
    const MAX_CHUNK_SIZE: usize = 1024 * 1024 * 1024; // 1GB

    if chunk_size < MIN_CHUNK_SIZE || chunk_size > MAX_CHUNK_SIZE {
        Err(crate::error::MismallError::Compression(
            CompressionError::InvalidChunkSize(chunk_size),
        ))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_result_creation() {
        let encode_info = EncodeInfo {
            original_size: 1000,
            compressed_size: 750,
            padding_bits: 4,
        };

        let result = CompressionResult::new(encode_info, "test.txt".to_string(), false);

        assert_eq!(result.original_size, 1000);
        assert_eq!(result.compressed_size, 750);
        assert_eq!(result.padding_bits, 4);
        assert_eq!(result.compression_ratio, 75.0);
        assert!(!result.encrypted);
        assert_eq!(result.filename, "test.txt");
        assert_eq!(result.bytes_saved(), -250);
        assert_eq!(result.savings_percentage(), 25.0);
        assert!(result.is_beneficial());
    }

    #[test]
    fn test_validate_chunk_size() {
        assert!(validate_chunk_size(1024 * 1024).is_ok()); // 1MB
        assert!(validate_chunk_size(64 * 1024).is_ok()); // 64KB min
        assert!(validate_chunk_size(1024 * 1024 * 1024).is_ok()); // 1GB max

        assert!(validate_chunk_size(32 * 1024).is_err()); // Too small
        assert!(validate_chunk_size(2 * 1024 * 1024 * 1024).is_err()); // Too large
    }

    #[test]
    fn test_compression_ratio_zero_size() {
        let encode_info = EncodeInfo {
            original_size: 0,
            compressed_size: 0,
            padding_bits: 0,
        };

        let result = CompressionResult::new(encode_info, "empty.txt".to_string(), false);
        assert_eq!(result.compression_ratio, 100.0);
    }
}
