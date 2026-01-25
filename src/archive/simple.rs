use super::{ArchiveInfo, FileInfo};
use crate::crypto::DEFAULT_CHUNK_SIZE;
use crate::error::Result;
use crate::flags;
use crate::headers::Headers;
use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
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
/// use std::path::Path;
///
/// // Note: This requires an existing archive file
/// // let (info, files) = list_archive_contents(Path::new("backup.small"))?;
/// // println!("Archive contains {} files, total size: {} bytes",
/// //          info.file_count, info.total_original_size);
/// // for (i, file) in files.iter().enumerate() {
/// //     println!("{}: {} ({} bytes)", i + 1, file.path, file.original_size);
/// // }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn list_archive_contents(archive_path: &Path) -> Result<(ArchiveInfo, Vec<FileInfo>)> {
    let file = File::open(archive_path).map_err(|e| crate::error::MismallError::Io {
        error: e,
        context: None,
        suggestion: None,
    })?;

    let mut reader = std::io::BufReader::new(file);

    // Read master header
    let master_header = Headers::from_reader(&mut reader).map_err(|e| {
        crate::error::MismallError::Decompression {
            error: crate::error::DecompressionError::InvalidFormat(format!(
                "Failed to parse master header: {}",
                e
            )),
            context: None,
            suggestion: None,
        }
    })?;

    // Check if this is actually an archive
    if !flags::is_archive(master_header.flags) {
        return Err(crate::error::MismallError::Decompression {
            error: crate::error::DecompressionError::InvalidFormat(
                "Input file is not an archive".to_string(),
            ),
            context: None,
            suggestion: None,
        });
    }

    let initial_body_position =
        reader
            .stream_position()
            .map_err(|e| crate::error::MismallError::Io {
                error: e,
                context: None,
                suggestion: None,
            })?;

    let archive_body_end_position = initial_body_position + master_header.compressed_size;
    let mut files = Vec::new();
    let mut total_original_size = 0u64;
    let mut total_compressed_size = 0u64;
    let mut has_encrypted_files = false;

    // Read all embedded file headers
    while reader
        .stream_position()
        .map_err(|e| crate::error::MismallError::Io {
            error: e,
            context: None,
            suggestion: None,
        })?
        < archive_body_end_position
    {
        let embedded_header_result = Headers::from_reader(&mut reader);
        let embedded_header = match embedded_header_result {
            Ok(header) => header,
            Err(_) => break, // End of archive or corrupted header
        };

        // Create file info for this embedded file
        let file_info = FileInfo::new(
            embedded_header.original_file_name.clone(),
            embedded_header.original_size,
            embedded_header.compressed_size,
            flags::is_encrypted(embedded_header.flags),
        );

        total_original_size += embedded_header.original_size;
        total_compressed_size += embedded_header.compressed_size;
        if flags::is_encrypted(embedded_header.flags) {
            has_encrypted_files = true;
        }

        files.push(file_info);

        // Skip the compressed data to get to next header
        let current_position =
            reader
                .stream_position()
                .map_err(|e| crate::error::MismallError::Io {
                    error: e,
                    context: None,
                    suggestion: None,
                })?;
        let next_position = current_position + embedded_header.compressed_size;
        reader.seek(SeekFrom::Start(next_position)).map_err(|e| {
            crate::error::MismallError::Io {
                error: e,
                context: None,
                suggestion: None,
            }
        })?;
    }

    let archive_info = ArchiveInfo::new(
        files.len(),
        total_original_size,
        total_compressed_size,
        has_encrypted_files,
    );

    Ok((archive_info, files))
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
/// use std::path::Path;
///
/// // Note: This requires an existing archive file
/// // extract_file(Path::new("backup.small"), "documents/contract.pdf", Path::new("contract.pdf"), None)?;
/// // println!("File extracted successfully");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn extract_file(
    archive_path: &std::path::Path,
    file_path: &str,
    output_path: &std::path::Path,
    password: Option<&str>,
) -> Result<()> {
    let file = File::open(archive_path).map_err(|e| crate::error::MismallError::Io {
        error: e,
        context: None,
        suggestion: None,
    })?;

    let mut reader = std::io::BufReader::new(file);

    // Read master header
    let master_header = Headers::from_reader(&mut reader).map_err(|e| {
        crate::error::MismallError::Decompression {
            error: crate::error::DecompressionError::InvalidFormat(format!(
                "Failed to parse master header: {}",
                e
            )),
            context: None,
            suggestion: None,
        }
    })?;

    // Check if this is actually an archive
    if !flags::is_archive(master_header.flags) {
        return Err(crate::error::MismallError::Decompression {
            error: crate::error::DecompressionError::InvalidFormat(
                "Input file is not an archive".to_string(),
            ),
            context: None,
            suggestion: None,
        });
    }

    let initial_body_position =
        reader
            .stream_position()
            .map_err(|e| crate::error::MismallError::Io {
                error: e,
                context: None,
                suggestion: None,
            })?;

    let archive_body_end_position = initial_body_position + master_header.compressed_size;

    // Search for the target file
    while reader
        .stream_position()
        .map_err(|e| crate::error::MismallError::Io {
            error: e,
            context: None,
            suggestion: None,
        })?
        < archive_body_end_position
    {
        let current_position =
            reader
                .stream_position()
                .map_err(|e| crate::error::MismallError::Io {
                    error: e,
                    context: None,
                    suggestion: None,
                })?;

        let embedded_header_result = Headers::from_reader(&mut reader);
        let embedded_header = match embedded_header_result {
            Ok(header) => header,
            Err(_) => break, // End of archive or corrupted header
        };

        // Check if this is the file we're looking for
        if embedded_header.original_file_name == file_path {
            // Found the target file, decode it
            let output_file =
                std::fs::File::create(output_path).map_err(|e| crate::error::MismallError::Io {
                    error: e,
                    context: None,
                    suggestion: None,
                })?;

            let mut output_writer = std::io::BufWriter::new(output_file);

            crate::huffman::decoder::decode(
                embedded_header,
                &mut reader,
                password,
                &mut output_writer,
                DEFAULT_CHUNK_SIZE,
            )
            .map_err(|e| crate::error::MismallError::Decompression {
                error: crate::error::DecompressionError::DecompressionFailed(e.to_string()),
                context: None,
                suggestion: None,
            })?;

            output_writer
                .flush()
                .map_err(|e| crate::error::MismallError::Io {
                    error: e,
                    context: None,
                    suggestion: None,
                })?;

            return Ok(());
        }

        // Skip this file's data to get to the next header
        let next_position = current_position + embedded_header.compressed_size;
        reader.seek(SeekFrom::Start(next_position)).map_err(|e| {
            crate::error::MismallError::Io {
                error: e,
                context: None,
                suggestion: None,
            }
        })?;
    }

    // File not found
    Err(crate::error::MismallError::Archive {
        error: crate::error::ArchiveError::FileNotFound(file_path.to_string()),
        context: None,
        suggestion: None,
    })
}

/// Extract all files from an archive
///
/// # Arguments
///
/// * `archive_path` - Path to the .small archive file
/// * `output_dir` - Directory where files should be extracted
/// * `password` - Optional password for decryption
///
/// # Examples
///
/// ```rust
/// use mismall::archive::extract_archive;
/// use std::path::Path;
///
/// // Note: This requires an existing archive file
/// // let info = extract_archive(Path::new("backup.small"), Path::new("extracted/"), None)?;
/// // println!("Extracted {} files to extracted/", info.file_count);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn extract_archive(
    archive_path: &Path,
    output_dir: &Path,
    password: Option<&str>,
) -> Result<ArchiveInfo> {
    // Ensure output directory exists
    std::fs::create_dir_all(output_dir).map_err(|e| crate::error::MismallError::Io {
        error: e,
        context: None,
        suggestion: None,
    })?;

    let file = File::open(archive_path).map_err(|e| crate::error::MismallError::Io {
        error: e,
        context: None,
        suggestion: None,
    })?;

    let mut reader = std::io::BufReader::new(file);

    // Read master header
    let master_header = Headers::from_reader(&mut reader).map_err(|e| {
        crate::error::MismallError::Decompression {
            error: crate::error::DecompressionError::InvalidFormat(format!(
                "Failed to parse master header: {}",
                e
            )),
            context: None,
            suggestion: None,
        }
    })?;

    // Check if this is actually an archive
    if !flags::is_archive(master_header.flags) {
        return Err(crate::error::MismallError::Decompression {
            error: crate::error::DecompressionError::InvalidFormat(
                "Input file is not an archive".to_string(),
            ),
            context: None,
            suggestion: None,
        });
    }

    let initial_body_position =
        reader
            .stream_position()
            .map_err(|e| crate::error::MismallError::Io {
                error: e,
                context: None,
                suggestion: None,
            })?;

    let archive_body_end_position = initial_body_position + master_header.compressed_size;
    let mut files = Vec::new();
    let mut total_original_size = 0u64;
    let mut total_compressed_size = 0u64;
    let mut has_encrypted_files = false;

    // Extract all embedded files
    while reader
        .stream_position()
        .map_err(|e| crate::error::MismallError::Io {
            error: e,
            context: None,
            suggestion: None,
        })?
        < archive_body_end_position
    {
        let _current_position =
            reader
                .stream_position()
                .map_err(|e| crate::error::MismallError::Io {
                    error: e,
                    context: None,
                    suggestion: None,
                })?;

        let embedded_header_result = Headers::from_reader(&mut reader);
        let embedded_header = match embedded_header_result {
            Ok(header) => header,
            Err(_) => break, // End of archive or corrupted header
        };

        // Create output path for this file
        let file_output_path = output_dir.join(&embedded_header.original_file_name);

        // Ensure parent directories exist
        if let Some(parent) = file_output_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| crate::error::MismallError::Io {
                error: e,
                context: None,
                suggestion: None,
            })?;
        }

        // Extract the file
        let output_file = std::fs::File::create(&file_output_path).map_err(|e| {
            crate::error::MismallError::Io {
                error: e,
                context: None,
                suggestion: None,
            }
        })?;

        let mut output_writer = std::io::BufWriter::new(output_file);

        crate::huffman::decoder::decode(
            embedded_header.clone(),
            &mut reader,
            password,
            &mut output_writer,
            DEFAULT_CHUNK_SIZE,
        )
        .map_err(|e| crate::error::MismallError::Decompression {
            error: crate::error::DecompressionError::DecompressionFailed(e.to_string()),
            context: None,
            suggestion: None,
        })?;

        output_writer
            .flush()
            .map_err(|e| crate::error::MismallError::Io {
                error: e,
                context: None,
                suggestion: None,
            })?;

        // Track file info - use original header before moving it
        let file_info = FileInfo::new(
            embedded_header.original_file_name.clone(),
            embedded_header.original_size,
            embedded_header.compressed_size,
            flags::is_encrypted(embedded_header.flags),
        );

        total_original_size += embedded_header.original_size;
        total_compressed_size += embedded_header.compressed_size;
        if flags::is_encrypted(embedded_header.flags) {
            has_encrypted_files = true;
        }

        files.push(file_info);
    }

    Ok(ArchiveInfo::new(
        files.len(),
        total_original_size,
        total_compressed_size,
        has_encrypted_files,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_archive_contents_nonexistent() {
        let result = list_archive_contents(Path::new("nonexistent.small"));
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_archive_nonexistent() {
        let result = extract_archive(Path::new("nonexistent.small"), Path::new("output/"), None);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_file_nonexistent() {
        let result = extract_file(
            Path::new("nonexistent.small"),
            "test.txt",
            Path::new("output.txt"),
            None,
        );
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
