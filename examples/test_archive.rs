use mismall::archive::ArchiveBuilder;
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing archive creation and extraction...");

    // Create test files
    fs::write("test_file1.txt", "Hello, this is test file 1!")?;
    fs::write(
        "test_file2.txt",
        "This is test file 2 with different content.",
    )?;

    // Create an archive
    println!("Creating archive...");
    let archive_info = ArchiveBuilder::new()
        .add_file("document1.txt", b"Hello, this is test file 1!")?
        .add_file(
            "document2.txt",
            b"This is test file 2 with different content.",
        )?
        .build("test_archive.small")?;

    println!("Archive created with {} files", archive_info.file_count);
    println!(
        "Total original size: {} bytes",
        archive_info.total_original_size
    );
    println!(
        "Total compressed size: {} bytes",
        archive_info.total_compressed_size
    );
    println!(
        "Compression ratio: {:.1}%",
        archive_info.overall_compression_ratio
    );

    // List archive contents
    println!("\nListing archive contents...");
    let (info, files) = mismall::archive::list_archive_contents(Path::new("test_archive.small"))?;
    println!("Archive contains {} files:", info.file_count);
    for (i, file) in files.iter().enumerate() {
        println!(
            "  {}: {} ({} -> {} bytes, {:.1}% compression)",
            i + 1,
            file.path,
            file.original_size,
            file.compressed_size,
            file.compression_ratio
        );
    }

    // Extract all files
    println!("\nExtracting all files...");
    fs::create_dir_all("extracted")?;
    let extract_info = mismall::archive::extract_archive(
        Path::new("test_archive.small"),
        Path::new("extracted"),
        None,
    )?;
    println!("Extracted {} files", extract_info.file_count);

    // Verify extracted files
    println!("\nVerifying extracted files...");
    let extracted_content1 = fs::read_to_string("extracted/document1.txt")?;
    let extracted_content2 = fs::read_to_string("extracted/document2.txt")?;

    println!("File 1 content: {}", extracted_content1);
    println!("File 2 content: {}", extracted_content2);

    assert_eq!(extracted_content1, "Hello, this is test file 1!");
    assert_eq!(
        extracted_content2,
        "This is test file 2 with different content."
    );

    println!("\n✅ All tests passed! Archive creation and extraction working correctly.");

    // Cleanup
    fs::remove_file("test_archive.small")?;
    fs::remove_file("test_file1.txt")?;
    fs::remove_file("test_file2.txt")?;
    fs::remove_dir_all("extracted")?;

    Ok(())
}
