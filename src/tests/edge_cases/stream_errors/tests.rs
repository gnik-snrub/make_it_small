//! Stream edge case tests
//!
//! Tests edge cases and error conditions in streaming operations.
//! Ensures robustness when handling stream lifecycle and data corruption.

use crate::error::MismallError;
use crate::stream::{stream_reader, stream_writer, Compressor, Decompressor};
use std::io::{Cursor, Read, Result, Write};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compressor_write_after_finish() {
        let mut cursor = Cursor::new(Vec::new());
        let mut compressor = stream_writer(&mut cursor, "test.txt", None);

        // Write some data and finish
        compressor.write_all(b"test data").unwrap();
        compressor.finish().unwrap();

        // Should fail on write after finish
        let result = compressor.write_all(b"more data");
        assert!(result.is_err());
    }

    #[test]
    fn test_compressor_double_finish() {
        let mut cursor = Cursor::new(Vec::new());
        let mut compressor = stream_writer(&mut cursor, "test.txt", None);

        compressor.write_all(b"test data").unwrap();
        compressor.finish().unwrap();

        // Second finish should fail
        let result = compressor.finish();
        assert!(result.is_err());
    }

    #[test]
    fn test_decompressor_invalid_data() {
        let invalid_data = b"not compressed data";
        let mut cursor = Cursor::new(invalid_data);
        let mut decompressor = stream_reader(&mut cursor, None);

        let mut buffer = Vec::new();
        let result = decompressor.read_to_end(&mut buffer);

        assert!(result.is_err());
        // Should handle corrupted data gracefully
    }

    #[test]
    fn test_stream_writer_empty_filename() {
        let mut cursor = Cursor::new(Vec::new());
        let result = stream_writer(&mut cursor, "", None);

        assert!(result.is_err());
    }

    #[test]
    fn test_stream_chunk_boundary_conditions() {
        // Test streaming with data that aligns exactly on chunk boundaries
        let chunk_size = 1024; // 1KB
        let test_data = vec![0u8; chunk_size * 2]; // Exactly 2 chunks

        let mut cursor = Cursor::new(test_data);
        let mut compressor =
            stream_writer(&mut cursor, "test.txt", None).with_chunk_size(chunk_size);

        compressor.write_all(&test_data).unwrap();
        compressor.finish().unwrap();

        // Verify output
        let compressed = cursor.into_inner();
        assert!(!compressed.is_empty());
    }

    #[test]
    fn test_concurrent_stream_operations() {
        use std::sync::Arc;
        use std::thread;

        let test_data = b"test data for concurrency";
        let mut cursor = Cursor::new(Vec::new());
        let compressor = Arc::new(stream_writer(&mut cursor, "test.txt", None));

        let handles: Vec<_> = (0..4)
            .map(|_| {
                let compressor = compressor.clone();
                thread::spawn(move || {
                    let mut local_cursor = Cursor::new(test_data);
                    let mut local_compressor = (&*compressor).clone();
                    local_compressor.write_all(test_data).unwrap();
                    local_compressor.finish().unwrap();
                })
            })
            .collect();

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify result is consistent
        let compressed = cursor.into_inner();
        assert!(!compressed.is_empty());
    }
}
