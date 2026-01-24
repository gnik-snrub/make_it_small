//! Archive operations API
//!
//! This module provides functionality for creating and extracting archives
//! that contain multiple compressed files.

pub mod builder;
pub mod extractor;
mod simple;

pub use builder::ArchiveBuilder;
pub use extractor::ArchiveExtractor;
pub use simple::{extract_archive, list_archive_contents};

use crate::error::{ArchiveError, Result};
use std::path::PathBuf;

/// Information about a file within an archive
#[derive(Debug, Clone)]
pub struct FileInfo {
    /// Path of the file within the archive
    pub path: String,
    /// Original uncompressed size in bytes
    pub original_size: u64,
    /// Compressed size in bytes within the archive
    pub compressed_size: u64,
    /// Compression ratio as percentage
    pub compression_ratio: f32,
    /// Whether the file is encrypted
    pub encrypted: bool,
}

impl FileInfo {
    /// Create a new file info
    pub fn new(path: String, original_size: u64, compressed_size: u64, encrypted: bool) -> Self {
        let compression_ratio = if original_size > 0 {
            ((compressed_size as f32 / original_size as f32) * 100.0)
                .round()
                .clamp(0.0, 100.0)
        } else {
            100.0
        };

        Self {
            path,
            original_size,
            compressed_size,
            compression_ratio,
            encrypted,
        }
    }

    /// Check if compression was beneficial
    pub fn is_compressed(&self) -> bool {
        self.compressed_size < self.original_size
    }

    /// Get size savings in bytes
    pub fn bytes_saved(&self) -> i64 {
        self.compressed_size as i64 - self.original_size as i64
    }

    /// Get size savings as percentage
    pub fn savings_percentage(&self) -> f32 {
        100.0 - self.compression_ratio
    }
}

/// Information about an archive
#[derive(Debug, Clone)]
pub struct ArchiveInfo {
    /// Total number of files in the archive
    pub file_count: usize,
    /// Total original size of all files
    pub total_original_size: u64,
    /// Total compressed size of all files
    pub total_compressed_size: u64,
    /// Overall compression ratio as percentage
    pub overall_compression_ratio: f32,
    /// Overall size savings in bytes
    pub overall_bytes_saved: i64,
    /// Overall size savings as percentage
    pub overall_savings_percentage: f32,
    /// Whether any files in the archive are encrypted
    pub has_encrypted_files: bool,
}

impl ArchiveInfo {
    /// Create a new archive info
    pub fn new(
        file_count: usize,
        total_original_size: u64,
        total_compressed_size: u64,
        has_encrypted_files: bool,
    ) -> Self {
        let overall_compression_ratio = if total_original_size > 0 {
            ((total_compressed_size as f32 / total_original_size as f32) * 100.0)
                .round()
                .clamp(0.0, 100.0)
        } else {
            100.0
        };

        let overall_bytes_saved = total_compressed_size as i64 - total_original_size as i64;
        let overall_savings_percentage = 100.0 - overall_compression_ratio;

        Self {
            file_count,
            total_original_size,
            total_compressed_size,
            overall_compression_ratio,
            overall_bytes_saved,
            overall_savings_percentage,
            has_encrypted_files,
        }
    }

    /// Create archive info from a list of file infos
    pub fn from_files(files: &[FileInfo]) -> Self {
        let file_count = files.len();
        let total_original_size = files.iter().map(|f| f.original_size).sum();
        let total_compressed_size = files.iter().map(|f| f.compressed_size).sum();
        let has_encrypted_files = files.iter().any(|f| f.encrypted);

        Self::new(
            file_count,
            total_original_size,
            total_compressed_size,
            has_encrypted_files,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_info_creation() {
        let info = FileInfo::new("test.txt".to_string(), 1000, 750, false);

        assert_eq!(info.path, "test.txt");
        assert_eq!(info.original_size, 1000);
        assert_eq!(info.compressed_size, 750);
        assert_eq!(info.compression_ratio, 75.0);
        assert!(!info.encrypted);
        assert!(info.is_compressed());
        assert_eq!(info.bytes_saved(), -250);
        assert_eq!(info.savings_percentage(), 25.0);
    }

    #[test]
    fn test_archive_info_creation() {
        let info = ArchiveInfo::new(2, 2000, 1500, true);

        assert_eq!(info.file_count, 2);
        assert_eq!(info.total_original_size, 2000);
        assert_eq!(info.total_compressed_size, 1500);
        assert_eq!(info.overall_compression_ratio, 75.0);
        assert_eq!(info.overall_bytes_saved, -500);
        assert_eq!(info.overall_savings_percentage(), 25.0);
        assert!(info.has_encrypted_files);
    }

    #[test]
    fn test_archive_info_from_files() {
        let files = vec![
            FileInfo::new("file1.txt".to_string(), 1000, 750, false),
            FileInfo::new("file2.txt".to_string(), 2000, 1600, true),
        ];

        let info = ArchiveInfo::from_files(&files);

        assert_eq!(info.file_count, 2);
        assert_eq!(info.total_original_size, 3000);
        assert_eq!(info.total_compressed_size, 2350);
        assert!(info.has_encrypted_files);
    }
}
