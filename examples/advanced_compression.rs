use mismall::progress::ProgressCallback;
use mismall::{compress_stream, decompress_stream};
use std::error::Error;
use std::fs;
use std::io::{Cursor, Read, Write};

fn main() -> Result<(), Box<dyn Error>> {
    println!("=== Advanced Compression with Custom Settings ===");

    // Create a larger test file
    let mut large_content = String::new();
    for i in 0..1000 {
        large_content.push_str(&format!("This is line {} of our test file.\n", i));
    }
    fs::write("large_test.txt", large_content)?;
    let original_size = fs::metadata("large_test.txt")?.len();
    println!("Created large_test.txt with {} bytes", original_size);

    // Read the file for compression
    let input_data = fs::read("large_test.txt")?;
    let mut reader = Cursor::new(&input_data);
    let mut compressed_buffer = Vec::new();

    // Create a progress callback
    let progress_callback: ProgressCallback = Box::new(|progress| {
        println!("Progress: {:.1}%", progress.percentage);
    });

    // Compress with custom settings
    println!("\nCompressing with password and custom chunk size...");
    let compress_result = compress_stream(
        &mut reader,
        "large_test.txt",
        Some("my_secret_password"),
        &mut compressed_buffer,
        64 * 1024, // 64KB chunks
    )?;

    println!(
        "Compressed: {} -> {} bytes ({:.1}% ratio)",
        compress_result.original_size,
        compress_result.compressed_size,
        compress_result.compression_ratio
    );

    // Save compressed data to file
    let compressed_filename = "large_test.txt.small";
    fs::write(compressed_filename, &compressed_buffer)?;

    // Try to decompress with wrong password (should fail)
    println!("\nAttempting decompression with wrong password...");
    let mut wrong_compressed_reader = Cursor::new(&compressed_buffer);
    let mut wrong_decompressed_buffer = Vec::new();

    match decompress_stream(
        &mut wrong_compressed_reader,
        Some("wrong_password"),
        &mut wrong_decompressed_buffer,
        64 * 1024,
    ) {
        Ok(_) => println!("❌ Unexpected success with wrong password!"),
        Err(e) => println!("✅ Expected error: {}", e),
    }

    // Decompress with correct password
    println!("\nDecompressing with correct password...");
    let mut compressed_reader = Cursor::new(&compressed_buffer);
    let mut decompressed_buffer = Vec::new();

    let decompress_result = decompress_stream(
        &mut compressed_reader,
        Some("my_secret_password"),
        &mut decompressed_buffer,
        64 * 1024,
    )?;

    println!("Decompressed: {} bytes", decompress_result.original_size);

    // Verify content
    let original = fs::read_to_string("large_test.txt")?;
    let restored = String::from_utf8(decompressed_buffer)?;

    if original == restored {
        println!(
            "\n✅ Success! Password-protected compression and decompression worked correctly."
        );
    } else {
        println!("\n❌ Error! Content mismatch after password-protected operations.");
    }

    // Cleanup
    fs::remove_file("large_test.txt")?;
    fs::remove_file(compressed_filename)?;

    Ok(())
}
