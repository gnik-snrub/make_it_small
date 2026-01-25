//! Archive edge case tests
//!
//! Tests edge cases and error conditions in archive operations.
//! Ensures robustness when handling invalid archives and extraction errors.

use crate::archive::ArchiveBuilder;
use crate::archive::ArchiveExtractor;
use crate::error::{ArchiveError, MismallError};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_archive_builder_empty_filename() {
        let result = ArchiveBuilder::new().add_file("", b"content");

        assert!(result.is_err());
    }

    #[test]
    fn test_archive_builder_duplicate_filename() {
        let result = ArchiveBuilder::new()
            .add_file("test.txt", b"content1")
            .and_then(|builder| builder.add_file("test.txt", b"content2"));

        assert!(result.is_err());
    }

    #[test]
    fn test_archive_builder_too_many_files() {
        let builder = ArchiveBuilder::new();

        // Try to add way too many files (should fail gracefully)
        for i in 0..10000 {
            builder = builder
                .add_file(&format!("file{}.txt", i), b"content")
                .unwrap();
        }

        let result = builder.build("test.small");
        // Should handle gracefully or indicate resource limits
    }

    #[test]
    fn test_archive_extractor_nonexistent_file() {
        let extractor = ArchiveExtractor::new("nonexistent.small");
        let result = extractor.extract_all();

        assert!(result.is_err());
    }

    #[test]
    fn test_archive_extractor_corrupted_file() {
        // Create an invalid file that looks like it has a .small header
        let invalid_data = b"fake header content";
        std::fs::write("corrupted.small", invalid_data).unwrap();

        let extractor = ArchiveExtractor::new("corrupted.small");
        let result = extractor.extract_all();

        assert!(result.is_err());

        // Clean up
        std::fs::remove_file("corrupted.small").unwrap();
    }

    #[test]
    fn test_archive_extractor_missing_file() {
        let result =
            ArchiveExtractor::new("test.small").extract_file("nonexistent.txt", "output.txt");

        assert!(result.is_err());
    }

    #[test]
    fn test_archive_builder_large_file() {
        // Test with very large file content
        let large_content = vec![0u8; 100 * 1024 * 1024]; // 100MB
        let result = ArchiveBuilder::new().add_file("large.txt", large_content);

        // Should handle large files without issues
        assert!(result.is_ok());
    }
}
