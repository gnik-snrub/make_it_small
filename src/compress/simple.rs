use crate::compress::CompressionResult;
use crate::error::{CompressionError, Result};
use crate::huffman::encoder::encode;
use crate::progress::{ProcessingStage, ProgressTracker};
use std::fs::File;
use std::io::{Cursor, Read, Write};
use std::path::Path;

/// Default chunk size for compression (16MB)
pub const DEFAULT_CHUNK_SIZE: usize = 16 * 1024 * 1024;

/// Compress a file with optional encryption
///
/// This is a simple one-liner function for basic compression needs.
/// For more advanced options, use `CompressionBuilder`.
///
/// # Arguments
///
/// * `input_path` - Path to the input file to compress
/// * `password` - Optional password for AES-256-GCM encryption
///
/// # Returns
///
/// Returns `CompressionResult` containing compression statistics
///
/// # Examples
///
/// ```rust
/// use mismall::compress_file;
///
/// // Basic compression
/// let result = compress_file("document.txt", None)?;
/// println!("Compressed {} bytes to {} bytes",
///          result.original_size, result.compressed_size);
///
/// // Compressed with encryption
/// let encrypted_result = compress_file("secret.txt", Some("password"))?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn compress_file<P: AsRef<Path>>(
    input_path: P,
    password: Option<&str>,
) -> Result<CompressionResult> {
    let input_path = input_path.as_ref();
    let filename = input_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| crate::error::MismallError::InvalidInput {
            message: "Invalid filename".to_string(),
            context: None,
            suggestion: None,
        })?
        .to_string();

    let mut input_file =
        File::open(input_path).map_err(|e| crate::error::MismallError::Compression {
            error: CompressionError::InputRead(format!("Failed to open input file: {}", e)),
            context: None,
            suggestion: None,
        })?;

    let mut output_buffer = Cursor::new(Vec::new());

    let encode_info = encode(
        &mut input_file,
        &filename,
        password,
        &mut output_buffer,
        DEFAULT_CHUNK_SIZE,
    )
    .map_err(|e| crate::error::MismallError::Compression {
        error: CompressionError::Encryption(e.to_string()),
        context: None,
        suggestion: None,
    })?;

    Ok(CompressionResult::new(
        encode_info,
        filename,
        password.is_some(),
    ))
}

/// Compress a data stream with optional encryption
///
/// Compresses data from any `Read` implementer and writes to any `Write` implementer.
/// This gives you full control over I/O streams.
///
/// # Arguments
///
/// * `reader` - Input data stream (must implement `Read + Seek`)
/// * `filename` - Filename to store in the compressed header
/// * `password` - Optional password for AES-256-GCM encryption
/// * `writer` - Output destination for compressed data
/// * `chunk_size` - Memory chunk size for processing (64KB to 1GB)
///
/// # Examples
///
/// ```rust
/// use std::io::Cursor;
/// use mismall::compress_stream;
///
/// let input_data = b"Hello, world! This is test data.";
/// let mut reader = Cursor::new(input_data);
/// let mut writer = Cursor::new(Vec::new());
///
/// compress_stream(&mut reader, "test.txt", None, &mut writer, 1024 * 1024)?;
/// let compressed = writer.into_inner();
/// println!("Compressed {} bytes", compressed.len());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn compress_stream<R: Read + std::io::Seek, W: Write>(
    reader: &mut R,
    filename: &str,
    password: Option<&str>,
    writer: &mut W,
    chunk_size: usize,
) -> Result<CompressionResult> {
    crate::compress::validate_chunk_size(chunk_size)?;

    let encode_info = encode(reader, filename, password, writer, chunk_size).map_err(|e| {
        crate::error::MismallError::Compression {
            error: CompressionError::Encryption(e.to_string()),
            context: None,
            suggestion: None,
        }
    })?;

    Ok(CompressionResult::new(
        encode_info,
        filename.to_string(),
        password.is_some(),
    ))
}

/// Compress a file with progress tracking
///
/// Internal function used by CompressionBuilder for progress callbacks
pub(crate) fn compress_file_with_progress<P: AsRef<Path>>(
    input_path: P,
    password: Option<&str>,
    _chunk_size: usize,
    progress_callback: Option<crate::progress::ProgressCallback>,
) -> Result<CompressionResult> {
    let input_path = input_path.as_ref();
    let _filename = input_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| crate::error::MismallError::InvalidInput {
            message: "Invalid filename".to_string(),
            context: None,
            suggestion: None,
        })?
        .to_string();

    // Get file size for progress tracking
    let file_size = std::fs::metadata(input_path)
        .map_err(|e| crate::error::MismallError::Compression {
            error: CompressionError::InputRead(format!("Failed to read file metadata: {}", e)),
            context: None,
            suggestion: None,
        })?
        .len();

    let mut progress_tracker =
        ProgressTracker::new(file_size, ProcessingStage::Reading, progress_callback);

    // For now, we'll use the basic compress_file and fake progress
    // In a real implementation, we'd modify the encode function to accept progress callbacks
    progress_tracker.update(file_size / 2);
    progress_tracker.set_stage(ProcessingStage::Encoding);

    let result = compress_file(input_path, password)?;

    progress_tracker.set_stage(ProcessingStage::Finalizing);
    progress_tracker.update(file_size);

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::huffman::encoder::EncodeInfo;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_compress_file_basic() {
        // Create a temporary test file
        let mut temp_file = NamedTempFile::new().unwrap();
        let test_data = b"Hello, world! This is test data for compression.";
        temp_file.write_all(test_data).unwrap();
        temp_file.flush().unwrap();

        // Test compression
        let result = compress_file(temp_file.path(), None).unwrap();

        assert_eq!(result.original_size, test_data.len() as u64);
        assert!(result.compressed_size > 0);
        assert_eq!(
            result.filename,
            temp_file.path().file_name().unwrap().to_str().unwrap()
        );
        assert!(!result.encrypted);
    }

    #[test]
    fn test_compress_file_encrypted() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let test_data = b"Secret data that needs encryption";
        temp_file.write_all(test_data).unwrap();
        temp_file.flush().unwrap();

        let result = compress_file(temp_file.path(), Some("password123")).unwrap();

        assert_eq!(result.original_size, test_data.len() as u64);
        assert!(result.encrypted);
    }

    #[test]
    fn test_compress_stream() {
        let input_data = b"Test data for stream compression";
        let mut reader = Cursor::new(input_data);
        let mut writer = Cursor::new(Vec::new());

        let result = compress_stream(
            &mut reader,
            "test.txt",
            None,
            &mut writer,
            1024 * 1024, // 1MB chunk
        )
        .unwrap();

        assert_eq!(result.original_size, input_data.len() as u64);
        assert_eq!(result.filename, "test.txt");
        assert!(!result.encrypted);

        let compressed = writer.into_inner();
        assert!(compressed.len() > 0);
    }

    #[test]
    fn test_invalid_chunk_size() {
        let mut reader = Cursor::new(b"test");
        let mut writer = Cursor::new(Vec::new());

        let result = compress_stream(&mut reader, "test.txt", None, &mut writer, 32 * 1024); // 32KB - too small
        assert!(result.is_err());
    }

    #[test]
    fn test_compression_result_beneficial() {
        let encode_info = EncodeInfo {
            original_size: 1000,
            compressed_size: 800, // Smaller = beneficial
            padding_bits: 0,
        };
        let result = CompressionResult::new(encode_info, "test.txt".to_string(), false);
        assert!(result.is_beneficial());

        let encode_info = EncodeInfo {
            original_size: 1000,
            compressed_size: 1200, // Larger = not beneficial
            padding_bits: 0,
        };
        let result = CompressionResult::new(encode_info, "test.txt".to_string(), false);
        assert!(!result.is_beneficial());
    }
}
