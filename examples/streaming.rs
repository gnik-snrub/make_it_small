use mismall::stream::{Compressor, Decompressor};
use std::error::Error;
use std::io::{self, Cursor, Read, Write};

fn main() -> Result<(), Box<dyn Error>> {
    println!("=== Streaming Compression Example ===");

    // Create some test data
    let test_data =
        b"This is some test data that we'll compress using the streaming API.\n".repeat(100);
    println!("Created {} bytes of test data", test_data.len());

    // Streaming compression to memory
    println!("\nCompressing with streaming API...");
    let compressed_buffer = Vec::new();
    let mut cursor = Cursor::new(compressed_buffer);

    {
        let mut compressor = Compressor::new(&mut cursor, "stream_test.txt", None::<&str>);
        compressor.write_all(&test_data)?;
        compressor.finish()?;
    }

    let compressed_buffer = cursor.into_inner();
    println!(
        "Compressed to {} bytes (ratio: {:.1}%)",
        compressed_buffer.len(),
        (compressed_buffer.len() as f64 / test_data.len() as f64) * 100.0
    );

    // Streaming decompression from memory
    println!("\nDecompressing with streaming API...");
    let mut compressed_cursor = Cursor::new(&compressed_buffer);
    let mut decompressor = Decompressor::new(&mut compressed_cursor, None::<&str>);

    let mut decompressed_data = Vec::new();
    decompressor.read_to_end(&mut decompressed_data)?;

    println!("Decompressed to {} bytes", decompressed_data.len());

    // Verify data integrity
    if test_data == decompressed_data {
        println!("✅ Streaming compression/decompression successful!");
    } else {
        println!("❌ Data mismatch after streaming operations!");
    }

    // Example with file streaming
    println!("\n--- File Streaming Example ---");

    // Create input file
    std::fs::write("stream_input.txt", &test_data)?;

    // Compress file using streaming
    let output_file = std::fs::File::create("stream_output.small")?;
    let mut file_compressor = Compressor::new(output_file, "stream_input.txt", None::<&str>);

    let mut input_file = std::fs::File::open("stream_input.txt")?;
    io::copy(&mut input_file, &mut file_compressor)?;
    file_compressor.finish()?;

    println!("File compression completed");

    // Decompress file using streaming
    let compressed_file = std::fs::File::open("stream_output.small")?;
    let mut file_decompressor = Decompressor::new(compressed_file, None::<&str>);

    let mut output_restored = std::fs::File::create("stream_restored.txt")?;
    io::copy(&mut file_decompressor, &mut output_restored)?;

    println!("File decompression completed");

    // Verify file content
    let original = std::fs::read_to_string("stream_input.txt")?;
    let restored = std::fs::read_to_string("stream_restored.txt")?;

    if original == restored {
        println!("✅ File streaming compression/decompression successful!");
    } else {
        println!("❌ File content mismatch!");
    }

    // Cleanup
    std::fs::remove_file("stream_input.txt")?;
    std::fs::remove_file("stream_output.small")?;
    std::fs::remove_file("stream_restored.txt")?;

    println!("\n✅ All streaming operations completed successfully!");

    Ok(())
}
