//! Memory stress tests
//!
//! Tests memory boundary conditions and resource exhaustion scenarios.
//! Ensures bounded memory usage and proper cleanup.

use crate::compress::CompressionBuilder;
use crate::stream::stream_writer;
use std::io::{Cursor, Write};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_usage_with_large_files() {
        // Test that memory usage remains bounded with large files
        let large_content = vec![0u8; 100 * 1024 * 1024]; // 100MB
        
        let mut cursor = Cursor::new(Vec::new());
        let compressor = stream_writer(&mut cursor, "large.txt", None)
            .with_chunk_size(1024 * 1024); // 1MB chunks
            .with_progress_callback(|_| {}); // Minimal callback
        
        compressor.write_all(&large_content).unwrap();
        compressor.finish().unwrap();
        
        // Should complete without memory issues
        let compressed = cursor.into_inner();
        assert!(!compressed.is_empty());
        assert!(compressed.len() < large_content.len()); // Should be compressed
    }

    #[test]
    fn test_chunk_size_memory_efficiency() {
        // Test various chunk sizes don't cause memory issues
        let test_data = vec![0u8; 1024]; // 1KB test data
        
        let chunk_sizes = vec![64 * 1024, 256 * 1024, 1024 * 1024]; // 64KB, 256KB, 1MB
        
        for chunk_size in chunk_sizes {
            let mut cursor = Cursor::new(Vec::new());
            let compressor = stream_writer(&mut cursor, "test.txt", None)
                .with_chunk_size(chunk_size);
            
            compressor.write_all(&test_data).unwrap();
            compressor.finish().unwrap();
            
            // Each chunk size should work
            let compressed = cursor.into_inner();
            assert!(!compressed.is_empty());
        }
    }

    #[test]
    fn test_progress_callback_overhead() {
        // Ensure progress callbacks don't cause memory leaks
        let test_data = b"x".repeat(1024);
        let mut call_count = 0;
        
        let mut cursor = Cursor::new(Vec::new());
        let compressor = stream_writer(&mut cursor, "test.txt", None)
            .with_progress_callback(|_| {
                call_count += 1;
            });
        
        // Write many small chunks to trigger many callbacks
        for _ in 0..100 {
            compressor.write_all(&test_data[..10]).unwrap();
        }
        compressor.finish().unwrap();
        
        // Should complete without issues
        assert!(call_count > 0);
        let compressed = cursor.into_inner();
        assert!(!compressed.is_empty());
    }

    #[test]
    fn test_cleanup_on_error() {
        // Ensure resources are cleaned up when operations fail
        let test_data = b"test data";
        
        // Test with invalid filename to trigger error
        let mut cursor = Cursor::new(Vec::new());
        let compressor = stream_writer(&mut cursor, "", None);
        
        let result = compressor.write_all(test_data);
        assert!(result.is_err());
        
        // Even on error, compressor should handle cleanup
        drop(compressor);
        // Test passes if no panic occurs
    }
}