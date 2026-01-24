use mismall::compress_file;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing compression only...");

    // Create test file
    fs::write(
        "test_compress.txt",
        "Hello, this is a test for basic compression!",
    )?;

    // Compress it
    println!("Compressing file...");
    let result = compress_file("test_compress.txt", None)?;
    println!(
        "Compressed {} -> {} bytes",
        result.original_size, result.compressed_size
    );

    // Check what files exist
    println!("Files in current directory:");
    for entry in std::fs::read_dir(".")? {
        let entry = entry?;
        let filename = entry.file_name();
        if let Some(name) = filename.to_str() {
            if name.contains("test_compress") {
                println!("  {}", name);
            }
        }
    }

    // Cleanup
    fs::remove_file("test_compress.txt")?;

    Ok(())
}
