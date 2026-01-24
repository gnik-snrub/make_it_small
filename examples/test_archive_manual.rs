use mismall::archive::{extract_archive, list_archive_contents, ArchiveBuilder};
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing archive functionality with manual test data...");

    // Test archive creation with known data
    println!("Creating archive with test data...");
    let archive_info = ArchiveBuilder::new()
        .add_file(
            "file1.txt",
            b"This is file 1 content. It's some text to test compression.",
        )?
        .add_file(
            "file2.txt",
            b"File 2 has different content: Hello World! 123456",
        )?
        .build("test_archive.small")?;

    println!("✅ Archive created successfully!");
    println!("   Files: {}", archive_info.file_count);
    println!(
        "   Original size: {} bytes",
        archive_info.total_original_size
    );
    println!(
        "   Compressed size: {} bytes",
        archive_info.total_compressed_size
    );
    println!(
        "   Compression: {:.1}%",
        archive_info.overall_compression_ratio
    );

    // List archive contents
    println!("\n📋 Listing archive contents...");
    let (info, files) = list_archive_contents(Path::new("test_archive.small"))?;
    println!("Archive contains {} files:", info.file_count);
    for (i, file) in files.iter().enumerate() {
        println!(
            "   {}: {} ({} -> {} bytes, {:.1}% compression)",
            i + 1,
            file.path,
            file.original_size,
            file.compressed_size,
            file.compression_ratio
        );
    }

    // Extract archive
    println!("\n📂 Extracting archive...");
    fs::create_dir_all("extracted")?;
    let extract_info = extract_archive(
        Path::new("test_archive.small"),
        Path::new("extracted"),
        None,
    )?;
    println!("✅ Extracted {} files", extract_info.file_count);

    // Verify extracted content
    println!("\n🔍 Verifying extracted files...");
    let content1 = fs::read_to_string("extracted/file1.txt")?;
    let content2 = fs::read_to_string("extracted/file2.txt")?;

    println!("File 1 content: \"{}\"", content1);
    println!("File 2 content: \"{}\"", content2);

    // Verify content matches
    let expected1 = "This is file 1 content. It's some text to test compression.";
    let expected2 = "File 2 has different content: Hello World! 123456";

    if content1 == expected1 && content2 == expected2 {
        println!("✅ All content matches expected values!");
    } else {
        println!("❌ Content mismatch detected!");
        println!("Expected file 1: \"{}\"", expected1);
        println!("Expected file 2: \"{}\"", expected2);
        return Err("Content verification failed".into());
    }

    // Cleanup
    println!("\n🧹 Cleaning up...");
    fs::remove_file("test_archive.small")?;
    fs::remove_dir_all("extracted")?;

    println!("✅ Archive creation and extraction test completed successfully!");

    Ok(())
}
