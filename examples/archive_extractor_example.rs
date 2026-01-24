//! Example demonstrating ArchiveExtractor usage
//!
//! This example shows how to use the ArchiveExtractor to list contents,
//! extract specific files, and extract all files from an archive.

use mismall::archive::{ArchiveBuilder, ArchiveExtractor};
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🗂️  ArchiveExtractor Example");
    println!("============================");

    // First, create a test archive to work with
    println!("\n📁 Creating test archive...");
    create_test_archive()?;

    // Example 1: List archive contents
    println!("\n📋 Example 1: Listing archive contents");
    println!("------------------------------------");
    list_archive_example()?;

    // Example 2: Extract a specific file
    println!("\n📤 Example 2: Extracting a specific file");
    println!("---------------------------------------");
    extract_specific_file_example()?;

    // Example 3: Extract all files to a directory
    println!("\n📂 Example 3: Extracting all files to directory");
    println!("-----------------------------------------------");
    extract_all_files_example()?;

    // Example 4: Extract with password protection
    println!("\n🔐 Example 4: Password-protected archive");
    println!("--------------------------------------");
    extract_with_password_example()?;

    // Cleanup
    cleanup_files()?;

    println!("\n✅ All ArchiveExtractor examples completed successfully!");
    Ok(())
}

fn create_test_archive() -> Result<(), Box<dyn std::error::Error>> {
    let archive_info = ArchiveBuilder::new()
        .add_file("documents/contract.pdf", b"PDF contract content here")?
        .add_file("images/photo.jpg", b"JPEG image data here")?
        .add_file("notes.txt", b"Important notes about the project")?
        .build("example_archive.small")?;

    println!("   Created archive with {} files", archive_info.file_count);
    println!(
        "   Original size: {} bytes",
        archive_info.total_original_size
    );
    println!(
        "   Compressed size: {} bytes",
        archive_info.total_compressed_size
    );
    println!(
        "   Compression ratio: {:.1}%",
        archive_info.overall_compression_ratio
    );
    Ok(())
}

fn list_archive_example() -> Result<(), Box<dyn std::error::Error>> {
    let (info, files) = ArchiveExtractor::new("example_archive.small").list_contents()?;

    println!("   Archive contains {} files:", info.file_count);
    println!(
        "   Total size: {} -> {} bytes ({:.1}% compression)",
        info.total_original_size, info.total_compressed_size, info.overall_compression_ratio
    );

    for (i, file) in files.iter().enumerate() {
        let status = if file.compressed_size < file.original_size {
            "compressed"
        } else {
            "stored"
        };

        println!(
            "   {}. {} ({} bytes, {} {} -> {})",
            i + 1,
            file.path,
            file.original_size,
            if file.encrypted { "🔒" } else { "📄" },
            status,
            file.compressed_size
        );
    }
    Ok(())
}

fn extract_specific_file_example() -> Result<(), Box<dyn std::error::Error>> {
    // Extract just the notes.txt file
    ArchiveExtractor::new("example_archive.small")
        .extract_file("notes.txt", "extracted_notes.txt")?;

    let content = fs::read_to_string("extracted_notes.txt")?;
    println!("   Extracted 'notes.txt' content: {:?}", content);

    // Cleanup extracted file
    fs::remove_file("extracted_notes.txt")?;
    Ok(())
}

fn extract_all_files_example() -> Result<(), Box<dyn std::error::Error>> {
    // Create output directory
    fs::create_dir_all("extracted_files")?;

    // Extract all files to the directory
    let info = ArchiveExtractor::new("example_archive.small")
        .with_output_dir("extracted_files")
        .extract_all()?;

    println!(
        "   Extracted {} files to 'extracted_files/'",
        info.file_count
    );

    // List extracted files
    for entry in fs::read_dir("extracted_files")? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let metadata = fs::metadata(&path)?;
            println!(
                "   📄 {} ({} bytes)",
                path.file_name().unwrap().to_string_lossy(),
                metadata.len()
            );
        }
    }

    // Cleanup
    fs::remove_dir_all("extracted_files")?;
    Ok(())
}

fn extract_with_password_example() -> Result<(), Box<dyn std::error::Error>> {
    // Create a password-protected archive
    println!("   Creating password-protected archive...");
    ArchiveBuilder::new()
        .add_file("secret.txt", b"This is confidential information")?
        .with_password("my_secret_password")
        .build("protected_archive.small")?;

    // Try to extract without password (should fail)
    let result = ArchiveExtractor::new("protected_archive.small")
        .extract_file("secret.txt", "failed_extract.txt");

    if result.is_err() {
        println!("   ❌ Extraction without password failed (expected)");
    }

    // Extract with correct password
    ArchiveExtractor::new("protected_archive.small")
        .with_password("my_secret_password")
        .extract_file("secret.txt", "secret_extracted.txt")?;

    let content = fs::read_to_string("secret_extracted.txt")?;
    println!("   ✅ Successfully extracted with password: {:?}", content);

    // Cleanup
    fs::remove_file("protected_archive.small")?;
    fs::remove_file("secret_extracted.txt")?;
    Ok(())
}

fn cleanup_files() -> Result<(), Box<dyn std::error::Error>> {
    let files_to_remove = ["example_archive.small", "extracted_notes.txt"];

    for file in &files_to_remove {
        if Path::new(file).exists() {
            fs::remove_file(file)?;
        }
    }

    // Remove directory if it exists
    if Path::new("extracted_files").exists() {
        fs::remove_dir_all("extracted_files")?;
    }

    Ok(())
}
