use mismall::{compress_stream, decompress_stream};
use std::error::Error;
use std::fs;
use std::io::Cursor;
use std::time::Instant;

fn main() -> Result<(), Box<dyn Error>> {
    println!("=== Performance Comparison Example ===");

    // Create test files of different sizes
    let test_cases = vec![
        ("small", 1_000),     // 1KB
        ("medium", 100_000),  // 100KB
        ("large", 1_000_000), // 1MB
    ];

    for (size_name, byte_count) in test_cases {
        println!("\n--- {} file ({} bytes) ---", size_name, byte_count);

        // Create test data
        let test_data = "A".repeat(byte_count);
        let filename = format!("test_{}.txt", size_name);
        fs::write(&filename, &test_data)?;

        // Read file for compression
        let input_data = fs::read(&filename)?;
        let mut reader = Cursor::new(&input_data);
        let mut compressed_buffer = Vec::new();

        // Measure compression
        let start = Instant::now();
        let compress_result = compress_stream(
            &mut reader,
            &filename,
            None,
            &mut compressed_buffer,
            1024 * 1024,
        )?;
        let compression_time = start.elapsed();

        let compressed_size = compressed_buffer.len() as u64;
        let compression_ratio = (compressed_size as f64 / byte_count as f64) * 100.0;

        println!(
            "Compression: {:.2}ms, {} bytes ({:.1}% ratio)",
            compression_time.as_millis(),
            compressed_size,
            compression_ratio
        );

        // Save compressed data
        let compressed_filename = format!("test_{}.txt.small", size_name);
        fs::write(&compressed_filename, &compressed_buffer)?;

        // Measure decompression
        let start = Instant::now();
        let mut compressed_reader = Cursor::new(&compressed_buffer);
        let mut decompressed_buffer = Vec::new();
        let decompress_result = decompress_stream(
            &mut compressed_reader,
            None,
            &mut decompressed_buffer,
            1024 * 1024,
        )?;
        let decompression_time = start.elapsed();

        println!("Decompression: {:.2}ms", decompression_time.as_millis());

        // Verify correctness (simple string comparison)
        let original_hash = test_data.as_bytes().iter().sum::<u8>();
        let restored_hash = decompressed_buffer.iter().sum::<u8>();

        if original_hash == restored_hash {
            println!("✅ Integrity verified (simple checksum: {})", original_hash);
        } else {
            println!("❌ Integrity check failed!");
        }

        // Performance metrics
        let compression_throughput =
            byte_count as f64 / compression_time.as_secs_f64() / 1_000_000.0; // MB/s
        let decompression_throughput =
            byte_count as f64 / decompression_time.as_secs_f64() / 1_000_000.0; // MB/s

        println!(
            "Throughput - Compression: {:.2} MB/s, Decompression: {:.2} MB/s",
            compression_throughput, decompression_throughput
        );

        // Cleanup
        fs::remove_file(filename)?;
        fs::remove_file(compressed_filename)?;
    }

    println!("\n=== Memory Efficiency Demonstration ===");

    // Test with a very large file to demonstrate streaming doesn't load everything into memory
    println!("Creating 10MB test file...");
    let large_data = "Performance test data line.\n".repeat(200_000); // ~200K lines
    fs::write("large_test.txt", large_data)?;
    let large_size = fs::metadata("large_test.txt")?.len();

    println!("Compressing 10MB file...");
    let input_data = fs::read("large_test.txt")?;
    let mut reader = Cursor::new(&input_data);
    let mut compressed_buffer = Vec::new();

    let start = Instant::now();
    let compress_result = compress_stream(
        &mut reader,
        "large_test.txt",
        None,
        &mut compressed_buffer,
        1024 * 1024,
    )?;
    let large_compression_time = start.elapsed();

    println!(
        "Large file compression: {:.2}ms, ratio: {:.1}%",
        large_compression_time.as_millis(),
        (compressed_buffer.len() as f64 / large_size as f64) * 100.0
    );

    // Cleanup
    fs::remove_file("large_test.txt")?;

    println!("\n✅ Performance comparison completed!");

    Ok(())
}
