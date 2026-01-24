use super::{ArchiveInfo, FileInfo};
use crate::error::{ArchiveError, Result};
use crate::headers::Headers;
use crate::huffman::decoder::{decode, DecodeInfo};
use std::fs::File;
use std::io::{Read, Seek};
use std::path::Path;

/// List contents of an archive without extracting files
///
/// # Arguments
///
/// * `archive_path` - Path to the .small archive file
///
/// # Returns
///
/// Returns `ArchiveInfo` with information about the archive and its contents
///
/// # Examples
///
/// ```rust
/// use mismall::archive::list_archive_contents;
///
/// let info = list_archive_contents("backup.small")?;
/// println!("Archive contains {} files, total size: {} bytes",
///          info.file_count, info.total_original_size);
/// for (i, file) in files.iter().enumerate() {
///     println!("{}: {} ({} bytes)", i + 1, file.path, file.original_size);
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn list_archive_contents<P: AsRef<Path>>(
    _archive_path: P,
    _output_dir: P,
    _password: Option<&str>,
) -> Result<ArchiveInfo> {
    let archive_path = archive_path.as_ref();
    let output_dir = output_dir.as_ref();

    // Ensure output directory exists
    std::fs::create_dir_all(output_dir).map_err(|e| {
        crate::error::MismallError::Archive(ArchiveError::Extraction(format!(
            "Failed to create output directory: {}",
            e
        )))
    })?;

    let (archive_info, _) = list_archive_contents(archive_path)?;

    // For now, this is a simplified implementation
    // In a full implementation, we would:
    // 1. Parse the archive file
    // 2. For each file, create the output file
    // 3. Decode each file into its output file
    // 4. Handle directory structures properly

    println!("Archive extraction not yet fully implemented");
    Ok(archive_info)
}

/// Extract a specific file from an archive
///
/// # Arguments
///
/// * `archive_path` - Path to the .small archive file
/// * `file_path` - Path of the file within the archive to extract
/// * `output_path` - Where the extracted file should be saved
/// * `password` - Optional password for decryption
///
/// # Examples
///
/// ```rust
/// use mismall::archive::extract_file;
///
/// extract_file("backup.small", "documents/contract.pdf", "contract.pdf", None)?;
/// println!("File extracted successfully");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn extract_file(
    archive_path: &std::path::Path,
    file_path: &str,
    output_path: &std::path::Path,
    password: Option<&str>,
) -> Result<()> {
    let archive_path = archive_path.as_ref();
    let output_path = output_path.as_ref();

    // This is a simplified implementation
    // In a full implementation, we would:
    // 1. Parse the archive file
    // 2. Find the specific file header
    // 3. Decode just that file's data
    // 4. Write it to the output path

    println!("Single file extraction not yet fully implemented");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_list_archive_contents_nonexistent() {
        let result = list_archive_contents("nonexistent.small");
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_archive_nonexistent() {
        let result = extract_archive("nonexistent.small", "output/", None);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_file_nonexistent() {
        let result = extract_file("nonexistent.small", "test.txt", "output.txt", None);
        assert!(result.is_err());
    }

    #[test]
    fn test_file_info_calculation() {
        let info = FileInfo::new("test.txt".to_string(), 1000, 800, false);
        assert_eq!(info.compression_ratio, 80.0);
        assert_eq!(info.bytes_saved(), -200);
        assert_eq!(info.savings_percentage(), 20.0);
    }

    #[test]
    fn test_archive_info_from_files() {
        let files = vec![
            FileInfo::new("a.txt".to_string(), 1000, 750, false),
            FileInfo::new("b.txt".to_string(), 2000, 1500, true),
        ];

        let info = ArchiveInfo::from_files(&files);
        assert_eq!(info.file_count, 2);
        assert_eq!(info.total_original_size, 3000);
        assert_eq!(info.total_compressed_size, 2250);
        assert_eq!(info.overall_compression_ratio, 75.0);
        assert!(info.has_encrypted_files);
    }
}
