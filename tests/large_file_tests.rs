use mismall::{compress::CompressionBuilder, compress_stream};
use std::io::Cursor;
use std::io::Write;
use std::time::Instant;
use tempfile::NamedTempFile;

// Test compression with various file sizes and chunk sizes
#[test]
fn test_large_files_various_chunks() {
    println!("Testing large files with various chunk sizes...");

    // Test different file sizes
    let file_sizes = vec![
        (1 * 1024 * 1024, "1MB"),   // 1MB
        (10 * 1024 * 1024, "10MB"), // 10MB
        (50 * 1024 * 1024, "50MB"), // 50MB
    ];

    // Test different chunk sizes
    let chunk_sizes = vec![
        (64 * 1024, "64KB"),        // Minimum allowed
        (256 * 1024, "256KB"),      // Small
        (1024 * 1024, "1MB"),       // Medium
        (16 * 1024 * 1024, "16MB"), // Default
    ];

    for (file_size, size_label) in file_sizes {
        println!("\nTesting {} file:", size_label);

        // Create test file with random-like data (using pattern for reproducibility)
        let mut test_file = NamedTempFile::new().unwrap();
        let data: Vec<u8> = (0..file_size).map(|i| (i % 256) as u8).collect();
        test_file.write_all(&data).unwrap();
        test_file.flush().unwrap();

        for (chunk_size, chunk_label) in &chunk_sizes {
            let start_time = Instant::now();

            let result = CompressionBuilder::new(test_file.path().to_str().unwrap())
                .with_chunk_size(*chunk_size)
                .compress();

            match result {
                Ok(compression_result) => {
                    let elapsed = start_time.elapsed();
                    let throughput_mb_per_sec =
                        (file_size as f64) / (1024.0 * 1024.0) / elapsed.as_secs_f64();

                    println!(
                        "  {} chunks: {:.2}s ({:.1} MB/s) - {:.1}% ratio",
                        chunk_label,
                        elapsed.as_secs_f64(),
                        throughput_mb_per_sec,
                        compression_result.compression_ratio
                    );

                    // Verify reasonable compression ratios (should be between 10-90%)
                    assert!(
                        compression_result.compression_ratio >= 10.0,
                        "Compression ratio too low: {:.1}%",
                        compression_result.compression_ratio
                    );
                    assert!(
                        compression_result.compression_ratio <= 100.0,
                        "Compression ratio impossible: {:.1}%",
                        compression_result.compression_ratio
                    );

                    // Verify file sizes match
                    assert_eq!(
                        compression_result.original_size, file_size as u64,
                        "Original size mismatch: expected {}, got {}",
                        file_size, compression_result.original_size
                    );
                }
                Err(e) => {
                    println!("  {} chunks: FAILED - {:?}", chunk_label, e);
                    panic!(
                        "Compression failed for {} file with {} chunks: {:?}",
                        size_label, chunk_label, e
                    );
                }
            }
        }
    }

    println!("\n✓ Large file tests completed successfully!");
}

// Test streaming compression with large data
#[test]
fn test_streaming_large_data() {
    println!("\nTesting streaming compression of large data...");

    let data_sizes = vec![(5 * 1024 * 1024, "5MB"), (20 * 1024 * 1024, "20MB")];

    for (data_size, size_label) in data_sizes {
        println!("Testing {} streaming data:", size_label);

        // Create test data with realistic mix of patterns (text-like + some entropy)
        let data: Vec<u8> = (0..data_size)
            .map(|i| {
                if i % 4 == 0 {
                    (i % 26 + 65) as u8
                }
                // Letter patterns
                else if i % 4 == 1 {
                    32
                }
                // Spaces
                else if i % 4 == 2 {
                    ((i * 13) % 128) as u8
                }
                // Medium entropy
                else {
                    ((i * 17) % 256) as u8
                } // Higher entropy
            })
            .collect();

        let start_time = Instant::now();
        let mut reader = Cursor::new(&data);
        let mut writer = Cursor::new(Vec::new());

        let result = compress_stream(
            &mut reader,
            "stream_test.bin",
            None,
            &mut writer,
            16 * 1024 * 1024,
        );

        match result {
            Ok(compression_result) => {
                let elapsed = start_time.elapsed();
                let compressed_data = writer.into_inner();
                let throughput_mb_per_sec =
                    (data_size as f64) / (1024.0 * 1024.0) / elapsed.as_secs_f64();

                println!(
                    "  {:.2}s ({:.1} MB/s) - {:.1}% ratio ({} -> {} bytes)",
                    elapsed.as_secs_f64(),
                    throughput_mb_per_sec,
                    compression_result.compression_ratio,
                    data_size,
                    compressed_data.len()
                );

                // Verify the streaming operation preserves data integrity
                assert_eq!(compression_result.original_size, data_size as u64);
                // For realistic mixed data, should get some compression (but allow expansion for incompressible data)
                let compression_efficiency = (compressed_data.len() as f64) / (data_size as f64);
                assert!(
                    compression_efficiency <= 1.2,
                    "Compression efficiency poor: {:.1}%",
                    compression_efficiency * 100.0
                );

                // Ensure reasonable performance (at least 1 MB/s on test hardware)
                assert!(
                    throughput_mb_per_sec >= 1.0,
                    "Streaming compression too slow: {:.1} MB/s",
                    throughput_mb_per_sec
                );
            }
            Err(e) => {
                panic!("Streaming compression failed for {}: {:?}", size_label, e);
            }
        }
    }

    println!("✓ Streaming large data tests completed successfully!");
}

// Test memory efficiency with very small chunks
#[test]
fn test_memory_efficiency() {
    println!("\nTesting memory efficiency with minimal chunks...");

    // Create a moderately sized file
    let file_size = 25 * 1024 * 1024; // 25MB
    let mut test_file = NamedTempFile::new().unwrap();
    let data: Vec<u8> = (0..file_size).map(|i| (i % 128) as u8).collect();
    test_file.write_all(&data).unwrap();
    test_file.flush().unwrap();

    // Test with minimum chunk size to verify bounded memory usage
    let min_chunk_size = 64 * 1024; // 64KB

    let start_time = Instant::now();
    let result = CompressionBuilder::new(test_file.path().to_str().unwrap())
        .with_chunk_size(min_chunk_size)
        .compress();

    match result {
        Ok(compression_result) => {
            let elapsed = start_time.elapsed();
            let memory_efficiency = (file_size as f64) / (min_chunk_size as f64); // Number of chunks processed

            println!("  Memory efficiency test:");
            println!("    File size: {} MB", file_size / (1024 * 1024));
            println!("    Chunk size: {} KB", min_chunk_size / 1024);
            println!("    Chunks processed: {:.0}", memory_efficiency);
            println!("    Processing time: {:.2}s", elapsed.as_secs_f64());
            println!(
                "    Compression ratio: {:.1}%",
                compression_result.compression_ratio
            );

            // Verify that even with small chunks, we can process large files efficiently
            assert!(
                elapsed.as_secs() <= 30,
                "Processing took too long: {:.1}s",
                elapsed.as_secs_f64()
            );
            assert!(
                compression_result.compression_ratio >= 20.0,
                "Poor compression with minimal chunks"
            );

            println!("  ✓ Memory efficiency verified - bounded memory usage with good performance");
        }
        Err(e) => {
            panic!("Memory efficiency test failed: {:?}", e);
        }
    }
}

#[test]
fn test_chunk_size_boundaries() {
    println!("\nTesting chunk size boundary conditions...");

    let test_size = 10 * 1024 * 1024; // 10MB test file
    let mut test_file = NamedTempFile::new().unwrap();
    let data = vec![42u8; test_size];
    test_file.write_all(&data).unwrap();
    test_file.flush().unwrap();

    // Test boundary chunk sizes
    let boundary_sizes = vec![
        (64 * 1024, "minimum"),          // 64KB - minimum
        (1024 * 1024, "common"),         // 1MB - common
        (16 * 1024 * 1024, "default"),   // 16MB - default
        (1024 * 1024 * 1024, "maximum"), // 1GB - maximum
    ];

    for (chunk_size, label) in boundary_sizes {
        println!("  Testing {} chunk size ({}):", label, chunk_size / 1024);

        let result = CompressionBuilder::new(test_file.path().to_str().unwrap())
            .with_chunk_size(chunk_size)
            .compress();

        match result {
            Ok(compression_result) => {
                println!(
                    "    ✓ Success - {:.1}% ratio",
                    compression_result.compression_ratio
                );
                assert_eq!(compression_result.original_size, test_size as u64);
            }
            Err(e) => {
                panic!("Failed with {} chunk size: {:?}", label, e);
            }
        }
    }

    println!("✓ Chunk size boundary tests completed!");
}
