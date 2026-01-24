use mismall::{compress_stream, decompress_stream};
use std::error::Error;
use std::fs;
use std::io::{Cursor, Read, Write};

fn main() -> Result<(), Box<dyn Error>> {
    println!("=== Simple Compression/Decompression Example ===");

    // Create a test file
    let test_content = "This is a test file for the mismall library.\nIt contains multiple lines.\nLet's see how well it compresses!";
    fs::write("test_input.txt", test_content)?;
    println!("Created test_input.txt with {} bytes", test_content.len());

    // Read the file for compression
    let mut input_data = fs::read("test_input.txt")?;
    let mut reader = Cursor::new(&input_data);
    let mut compressed_buffer = Vec::new();

    // Compress using stream API
    println!("\nCompressing file...");
    let compress_result = compress_stream(
        &mut reader,
        "test_input.txt",
        None,
        &mut compressed_buffer,
        1024 * 1024,
    )?;
    println!(
        "Compressed: {} -> {} bytes ({:.1}% ratio)",
        compress_result.original_size,
        compress_result.compressed_size,
        compress_result.compression_ratio
    );

    // Save compressed data to file
    let compressed_filename = "test_input.txt.small";
    fs::write(compressed_filename, &compressed_buffer)?;

    // Decompress using stream API
    println!("\nDecompressing file...");
    let mut compressed_reader = Cursor::new(&compressed_buffer);
    let mut decompressed_buffer = Vec::new();

    let decompress_result = decompress_stream(
        &mut compressed_reader,
        None,
        &mut decompressed_buffer,
        1024 * 1024,
    )?;
    println!("Decompressed: {} bytes", decompress_result.original_size);

    // Verify the content
    let original_content = fs::read_to_string("test_input.txt")?;
    let restored_content = String::from_utf8(decompressed_buffer)?;

    if original_content == restored_content {
        println!("\n✅ Success! Original and decompressed content match.");
    } else {
        println!("\n❌ Error! Content mismatch.");
    }

    // Cleanup
    fs::remove_file("test_input.txt")?;
    fs::remove_file(compressed_filename)?;

    // Cleanup
    fs::remove_file("test_input.txt")?;
    fs::remove_file(compressed_filename)?;

    Ok(())
}
