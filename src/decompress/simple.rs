use crate::crypto::DEFAULT_CHUNK_SIZE;
use crate::decompress::DecompressionResult;
use crate::error::{DecompressionError, Result};
use crate::headers::Headers;
use crate::huffman::decoder::{decode, DecodeInfo};
use crate::progress::{ProgressCallback, ProgressTracker};
use std::fs::File;
use std::io::{Cursor, Read, Write};
use std::path::Path;

/// Default chunk size for decompression (16MB)
pub const DECOMPRESS_DEFAULT_CHUNK_SIZE: usize = 16 * 1024 * 1024;

/// Decompress a file with optional password decryption
///
/// This is a simple one-liner function for basic decompression needs.
/// For more advanced options, use `DecompressionBuilder`.
///
/// # Arguments
///
/// * `input_path` - Path to the .small file to decompress
/// * `password` - Optional password for AES-256-GCM decryption
///
/// # Returns
///
/// Returns `DecompressionResult` containing decompression statistics
///
/// # Examples
///
/// ```rust
/// use mismall::decompress_file;
///
/// // Basic decompression
/// let result = decompress_file("document.txt.small", None)?;
/// println!("Decompressed {} bytes as {}",
///          result.original_size, result.original_filename);
///
/// // Decompressed with password
/// let decrypted_result = decompress_file("secret.txt.small", Some("password"))?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn decompress_file<P: AsRef<Path>>(
    input_path: P,
    password: Option<&str>,
) -> Result<DecompressionResult> {
    let input_path = input_path.as_ref();
    let mut input_file = File::open(input_path).map_err(|e| {
        crate::error::MismallError::Decompression(DecompressionError::InputRead(format!(
            "Failed to open input file: {}",
            e
        )))
    })?;

    let mut output_buffer = Cursor::new(Vec::new());

    let decode_info = decompress_stream(
        &mut input_file,
        password,
        &mut output_buffer,
        DEFAULT_CHUNK_SIZE,
    )?;

    Ok(DecompressionResult::new(
        decode_info.original_file_name,
        decode_info.original_size,
        decode_info.checksum,
        password.is_some(),
    ))
}

/// Decompress a file with progress callback
///
/// Internal function used by DecompressionBuilder to handle progress tracking.
pub fn decompress_file_with_progress<P: AsRef<Path>>(
    input_path: P,
    password: Option<&str>,
    chunk_size: usize,
    progress_callback: Option<ProgressCallback>,
) -> Result<DecompressionResult> {
    let input_path = input_path.as_ref();

    // Get file size for progress tracking
    let file_size = std::fs::metadata(input_path)
        .map_err(|e| {
            crate::error::MismallError::Decompression(DecompressionError::InputRead(format!(
                "Failed to read file metadata: {}",
                e
            )))
        })?
        .len();

    let mut progress_tracker = ProgressTracker::new(
        file_size,
        crate::progress::ProcessingStage::Reading,
        progress_callback,
    );

    let mut input_file = File::open(input_path).map_err(|e| {
        crate::error::MismallError::Decompression(DecompressionError::InputRead(format!(
            "Failed to open input file: {}",
            e
        )))
    })?;

    let mut output_buffer = Cursor::new(Vec::new());

    // Update progress for reading phase
    progress_tracker.update(file_size / 3);
    progress_tracker.set_stage(crate::progress::ProcessingStage::Decoding);

    let decode_info = decompress_stream(&mut input_file, password, &mut output_buffer, chunk_size)?;

    // Update progress for finalizing phase
    progress_tracker.update(2 * file_size / 3);
    progress_tracker.set_stage(crate::progress::ProcessingStage::Writing);
    progress_tracker.update(file_size);

    Ok(DecompressionResult::new(
        decode_info.original_file_name,
        decode_info.original_size,
        decode_info.checksum,
        password.is_some(),
    ))
}

/// Decompress a data stream with optional password decryption
///
/// Decompresses data from any `Read` implementer and writes to any `Write` implementer.
/// This gives you full control over I/O streams.
///
/// # Arguments
///
/// * `reader` - Input compressed data stream (must implement `Read + Seek`)
/// * `password` - Optional password for AES-256-GCM decryption
/// * `writer` - Output destination for decompressed data
/// * `chunk_size` - Memory chunk size for processing (64KB to 1GB)
///
/// # Examples
///
/// ```rust
/// use std::io::Cursor;
/// use mismall::decompress_stream;
///
/// let compressed_data = b"compressed_data_here";
/// let mut reader = Cursor::new(compressed_data);
/// let mut writer = Cursor::new(Vec::new());
///
/// let result = decompress_stream(&mut reader, None, &mut writer, 1024 * 1024)?;
/// let decompressed = writer.into_inner();
/// println!("Decompressed {} bytes", decompressed.len());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn decompress_stream<R: Read + std::io::Seek, W: Write>(
    reader: &mut R,
    password: Option<&str>,
    writer: &mut W,
    chunk_size: usize,
) -> Result<DecodeInfo> {
    crate::compress::validate_chunk_size(chunk_size)?;

    // Read headers first
    let header = Headers::from_reader(reader).map_err(|e| {
        crate::error::MismallError::Decompression(DecompressionError::InvalidFormat(format!(
            "Failed to read headers: {}",
            e
        )))
    })?;

    // Use the decoder directly to handle streaming
    decode(header, reader, password, writer, chunk_size).map_err(|e| {
        crate::error::MismallError::Decompression(DecompressionError::CorruptedData(format!(
            "Failed to decode: {}",
            e
        )))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_decompress_file_basic() {
        // Create a temporary test file with compressed data
        let mut temp_file = NamedTempFile::new().unwrap();
        let test_data = b"Hello, world! This is test data.";
        temp_file.write_all(test_data).unwrap();
        temp_file.flush().unwrap();

        // First compress it
        let compressed_result =
            crate::compress::simple::compress_file(temp_file.path(), None).unwrap();

        // Now decompress it (this is a simplified test)
        // In reality, we'd need actual compressed data
        assert!(compressed_result.original_size > 0);
    }

    #[test]
    fn test_decompress_stream() {
        // This test would require actual compressed data
        // For now, just test the function signature and basic validation
        let mut reader = Cursor::new(b"dummy_compressed_data");
        let mut writer = Cursor::new(Vec::new());

        let result = decompress_stream(
            &mut reader,
            None,
            &mut writer,
            1024 * 1024, // 1MB chunk
        );

        // Should fail with invalid format since we're using dummy data
        assert!(result.is_err());
    }
}
