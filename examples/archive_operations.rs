use mismall::archive::{list_archive_contents, ArchiveBuilder, ArchiveExtractor};
use std::error::Error;
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    println!("=== Archive Operations Example ===");

    // Create some test files
    fs::write("file1.txt", "This is the first file.")?;
    fs::write(
        "file2.txt",
        "This is the second file with more content.\nMultiple lines!",
    )?;
    fs::write("file3.txt", "Third file content here.")?;

    println!("Created test files: file1.txt, file2.txt, file3.txt");

    // Read file contents for archiving
    let file1_data = fs::read("file1.txt")?;
    let file2_data = fs::read("file2.txt")?;
    let file3_data = fs::read("file3.txt")?;

    // Create an archive with password protection
    println!("\nCreating archive with password protection...");
    ArchiveBuilder::new()
        .add_file("file1.txt", file1_data)?
        .add_file("file2.txt", file2_data)?
        .add_file("file3.txt", file3_data)?
        .with_password("archive_secret")
        .build("test_archive.small")?;

    println!("Archive created: test_archive.small");

    // List archive contents
    println!("\nListing archive contents...");
    let (archive_info, file_infos) = list_archive_contents(Path::new("test_archive.small"))?;
    println!("Archive contains {} files:", file_infos.len());
    for (i, file_info) in file_infos.iter().enumerate() {
        println!(
            "  {}: {} ({} bytes)",
            i + 1,
            file_info.path,
            file_info.original_size
        );
    }

    // Extract a specific file
    println!("\nExtracting file2.txt to extracted_file2.txt...");
    ArchiveExtractor::new("test_archive.small")
        .with_password("archive_secret")
        .extract_file("file2.txt", "extracted_file2.txt")?;

    // Verify the extracted file
    let original_content = fs::read_to_string("file2.txt")?;
    let extracted_content = fs::read_to_string("extracted_file2.txt")?;

    if original_content == extracted_content {
        println!("✅ Extracted file content matches original!");
    } else {
        println!("❌ Extracted file content mismatch!");
    }

    // For extracting all files, we'll extract them one by one since extract_all isn't available
    println!("\nExtracting all files to 'extracted_all' directory...");
    fs::create_dir_all("extracted_all")?;

    for file_info in &file_infos {
        let output_path = format!("extracted_all/{}", file_info.path);
        ArchiveExtractor::new("test_archive.small")
            .with_password("archive_secret")
            .extract_file(&file_info.path, &output_path)?;
    }

    // List extracted files
    println!("Files extracted to extracted_all/:");
    for entry in fs::read_dir("extracted_all")? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            println!("  {}", path.file_name().unwrap().to_string_lossy());
        }
    }

    // Cleanup
    println!("\nCleaning up...");
    fs::remove_file("file1.txt")?;
    fs::remove_file("file2.txt")?;
    fs::remove_file("file3.txt")?;
    fs::remove_file("test_archive.small")?;
    fs::remove_file("extracted_file2.txt")?;

    // Remove extracted directory and its contents
    for entry in fs::read_dir("extracted_all")? {
        let entry = entry?;
        fs::remove_file(entry.path())?;
    }
    fs::remove_dir("extracted_all")?;

    println!("✅ Archive operations example completed successfully!");

    Ok(())
}
