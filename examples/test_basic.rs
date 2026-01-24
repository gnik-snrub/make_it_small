use mismall::compress_file;
use mismall::decompress_file;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing basic compression and decompression...");

    // Create test file
    fs::write(
        "test_basic.txt",
        "Hello, this is a test for basic compression!",
    )?;

    // Compress it
    println!("Compressing file...");
    let result = compress_file("test_basic.txt", None)?;
    println!(
        "Compressed {} -> {} bytes",
        result.original_size, result.compressed_size
    );

    // Decompress it
    println!("Decompressing file...");
    let decompress_result = decompress_file("test_basic.txt.small", None)?;
    println!("Decompressed {} bytes", decompress_result.original_size);

    // Verify content
    let original = fs::read_to_string("test_basic.txt")?;
    let decompressed = fs::read_to_string("test_basic.txt")?;

    assert_eq!(original, decompressed);
    println!("✅ Basic compression/decompression works!");

    // Cleanup
    fs::remove_file("test_basic.txt")?;
    fs::remove_file("test_basic.txt.small")?;
    fs::remove_file("test_basic.txt")?;

    Ok(())
}
