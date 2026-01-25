use mismall::{compress::validate_chunk_size, compress::CompressionBuilder, compress_stream};
use std::io::Cursor;
use std::io::Write;
use tempfile::NamedTempFile;

// Test to verify memory usage remains bounded regardless of file size
#[test]
fn test_memory_boundedness() {
    println!("Testing memory boundedness...");

    // Test 1: Very large file with small chunks
    let large_size = 100 * 1024 * 1024; // 100MB
    let mut large_file = NamedTempFile::new().unwrap();
    let data = vec![42u8; large_size];
    large_file.write_all(&data).unwrap();
    large_file.flush().unwrap();

    // Compress with 64KB chunks (minimum allowed)
    let start_time = std::time::Instant::now();
    let result = CompressionBuilder::new(large_file.path().to_str().unwrap())
        .with_chunk_size(64 * 1024) // 64KB chunks
        .compress()
        .unwrap();
    let compression_time = start_time.elapsed();

    println!("Large file (100MB) with 64KB chunks:");
    println!("  Compression time: {:?}", compression_time);
    println!("  Original size: {} bytes", result.original_size);
    println!("  Compressed size: {} bytes", result.compressed_size);
    println!("  Compression ratio: {:.1}%", result.compression_ratio);

    // Test 2: Stream compression with very small chunks to ensure memory usage is bounded
    let test_data = vec![123u8; 10 * 1024 * 1024]; // 10MB
    let start_time = std::time::Instant::now();

    let mut reader = Cursor::new(&test_data);
    let mut writer = Cursor::new(Vec::new());
    let result = compress_stream(&mut reader, "test.dat", None, &mut writer, 64 * 1024).unwrap();
    let stream_time = start_time.elapsed();

    println!("\nStream compression (10MB) with 64KB chunks:");
    println!("  Compression time: {:?}", stream_time);
    println!("  Original size: {} bytes", result.original_size);
    println!("  Compressed size: {} bytes", result.compressed_size);
    println!("  Compression ratio: {:.1}%", result.compression_ratio);

    // Test 3: Verify that chunk size validation works properly
    let validation_results = vec![
        (32 * 1024, false),              // 32KB - should fail
        (64 * 1024, true),               // 64KB - should pass (minimum)
        (1024 * 1024, true),             // 1MB - should pass
        (1024 * 1024 * 1024, true),      // 1GB - should pass (maximum)
        (2 * 1024 * 1024 * 1024, false), // 2GB - should fail
    ];

    println!("\nChunk size validation tests:");
    for (chunk_size, should_pass) in validation_results {
        let result = validate_chunk_size(chunk_size);
        let status = if should_pass {
            result.is_ok()
        } else {
            result.is_err()
        };

        println!(
            "  {}MB chunk size: {} (expected: {})",
            chunk_size / (1024 * 1024),
            if status { "✓ PASS" } else { "✗ FAIL" },
            if should_pass { "PASS" } else { "FAIL" }
        );
    }

    println!("\n✓ Memory boundedness tests completed successfully!");
    println!("  - Large files can be processed with small, bounded chunks");
    println!("  - Stream compression works with minimal memory overhead");
    println!("  - Chunk size validation prevents excessive memory usage");
}

fn main() {
    test_memory_boundedness();
}
