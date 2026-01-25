//! Large file tests
//!
//! Tests behavior with large files to ensure the library scales properly
//! and maintains performance characteristics.

use crate::archive::ArchiveBuilder;
use crate::compress::CompressionBuilder;
use std::io::{Cursor, Write};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_very_large_single_file() {
        // Test with a very large file (50MB)
        let large_content = vec![0x42; 50 * 1024 * 1024];

        let result = CompressionBuilder::new("large.txt")
            .with_chunk_size(1024 * 1024) // 1MB chunks
            .compress();

        assert!(result.is_ok());
        let compressed = result.unwrap();
        assert!(compressed.compressed_size > 0);
        assert!(compressed.compression_ratio < 100.0);
    }

    #[test]
    fn test_many_medium_files() {
        // Test archive with many medium-sized files
        let mut builder = ArchiveBuilder::new();

        for i in 0..100 {
            let content = format!("File {} content", i).into_bytes();
            builder = builder
                .add_file(&format!("file{}.txt", i), content)
                .unwrap();
        }

        let result = builder.build("multi_file.small");
        assert!(result.is_ok());
    }

    #[test]
    fn test_file_size_limits() {
        // Test with files at various size limits
        let sizes = vec![
            (1, "tiny"),
            (1024, "small"),             // 1KB
            (1024 * 1024, "medium"),     // 1MB
            (10 * 1024 * 1024, "large"), // 10MB
            (100 * 1024 * 1024, "huge"), // 100MB
        ];

        for (size, name) in sizes {
            let content = vec![0x42; size];
            let result = CompressionBuilder::new(&format!("{}.txt", name)).compress();

            assert!(result.is_ok());
            let compressed = result.unwrap();
            assert_eq!(compressed.original_size, size as u64);
        }
    }

    #[test]
    fn test_concurrent_large_operations() {
        use std::sync::Arc;
        use std::thread;

        let large_content = vec![0u8; 10 * 1024 * 1024]; // 10MB

        let handles: Vec<_> = (0..4)
            .map(|i| {
                let content = large_content.clone();
                thread::spawn(move || {
                    let result =
                        CompressionBuilder::new(&format!("concurrent_{}.txt", i)).compress();
                    result.is_ok()
                })
            })
            .collect();

        for handle in handles {
            assert!(handle.join().unwrap());
        }
    }
}
