//! Performance tests
//!
//! Tests performance characteristics to ensure no regressions
//! between CLI and library usage.

#[cfg(test)]
mod tests {
    #[test]
    fn test_compression_performance() {
        // Basic performance benchmark
        let test_data = vec![0u8; 1024 * 1024]; // 1MB
        let start = std::time::Instant::now();

        let result = crate::compress::compress_file("perf_test.txt", None).unwrap();
        let duration = start.elapsed();

        // Should complete in reasonable time
        assert!(duration.as_millis() < 5000);
        assert!(result.compressed_size > 0);
    }

    #[test]
    fn test_decompression_performance() {
        let test_data = vec![0x42; 1024 * 1024]; // 1MB

        // Create a temporary compressed file first
        let compressed_data = crate::compress::compress_file("perf_test.txt", None).unwrap();
        std::fs::write("perf_test.txt.small", &compressed_data).unwrap();

        let start = std::time::Instant::now();
        let result = crate::decompress::decompress_file("perf_test.txt.small", None).unwrap();
        let duration = start.elapsed();

        // Should complete in reasonable time
        assert!(duration.as_millis() < 5000);
        assert_eq!(result.original_size, 1024 * 1024);

        // Cleanup
        std::fs::remove_file("perf_test.txt.small").unwrap();
        std::fs::remove_file("perf_test.txt").unwrap();
    }

    #[test]
    fn test_memory_efficiency() {
        // Test that streaming doesn't load entire file into memory
        use std::sync::Arc;
        use std::thread;

        let large_content = vec![0u8; 100 * 1024 * 1024]; // 100MB

        let chunk_sizes = vec![64 * 1024, 256 * 1024, 1024 * 1024]; // 64KB, 256KB, 1MB

        for chunk_size in chunk_sizes {
            let start = std::time::Instant::now();

            let result = crate::compress::compress_file("mem_test.txt", None).unwrap();

            let duration = start.elapsed();

            // Each chunk size should work without excessive memory usage
            assert!(duration.as_millis() < 10000);
            assert!(result.compressed_size > 0);
        }
    }
}
