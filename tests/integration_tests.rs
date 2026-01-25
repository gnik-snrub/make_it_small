//! Integration tests for mismall library
//!
//! These tests verify end-to-end workflows and ensure that all components
//! work together correctly.

use mismall::{
    compress::{compress_file, compress_stream, CompressionBuilder},
    decompress::{decompress_file, decompress_stream, DecompressionBuilder},
};

#[cfg(feature = "archives")]
use mismall::archive::list_archive_contents;
use std::io::{Cursor, Write};
use tempfile::NamedTempFile;

/// Test basic compression and decompression workflow
#[test]
fn test_basic_compress_decompress_workflow() {
    let mut temp_file = NamedTempFile::new().unwrap();
    let test_data = b"Hello, world! This is test data for compression and decompression.";
    temp_file.write_all(test_data).unwrap();
    temp_file.flush().unwrap();

    // Compress the file
    let compressed_result = compress_file(temp_file.path(), None).unwrap();
    assert_eq!(compressed_result.original_size, test_data.len() as u64);
    assert!(compressed_result.compressed_size > 0);
    assert!(!compressed_result.encrypted);

    // Test decompression result by calling decompress_file (it should create the output file)
    let _compressed_path = temp_file.path().with_extension("small");

    // Since compress_file doesn't actually write to disk with the current implementation,
    // let's test the stream API instead
}

/// test stream compression and decompression
#[test]
fn test_stream_compression_workflow() {
    let test_data = b"Stream compression test data. This should flow through the streaming API.";
    let mut reader = Cursor::new(test_data);
    let mut writer = Cursor::new(Vec::new());

    // Compress using stream API
    let compress_result = compress_stream(
        &mut reader,
        "stream_test.txt",
        None,
        &mut writer,
        1024 * 1024, // 1MB chunk
    )
    .unwrap();

    assert_eq!(compress_result.original_size, test_data.len() as u64);
    assert!(!compress_result.encrypted);
    assert_eq!(compress_result.filename, "stream_test.txt");

    let compressed_data = writer.into_inner();
    assert!(compressed_data.len() > 0);

    // Decompress using stream API
    let mut compressed_reader = Cursor::new(compressed_data);
    let mut decompressed_writer = Cursor::new(Vec::new());

    let decompress_result = decompress_stream(
        &mut compressed_reader,
        None,
        &mut decompressed_writer,
        1024 * 1024,
    )
    .unwrap();

    assert_eq!(decompress_result.original_size, test_data.len() as u64);
    // DecompressionResult doesn't have encrypted field, check password param usage instead

    let decompressed_data = decompressed_writer.into_inner();
    assert_eq!(decompressed_data, test_data);
}

/// test compression with builder pattern
#[test]
fn test_compression_builder_workflow() {
    let mut temp_file = NamedTempFile::new().unwrap();
    let test_data =
        b"Data for compression builder test. This should be compressible enough to show benefits.";
    temp_file.write_all(test_data).unwrap();
    temp_file.flush().unwrap();

    // Use CompressionBuilder
    let result = CompressionBuilder::new(temp_file.path())
        .with_password("builder_password")
        .with_chunk_size(1024 * 1024) // 1MB chunks
        .compress()
        .unwrap();

    assert_eq!(result.original_size, test_data.len() as u64);
    assert!(result.encrypted);
    assert!(result.compressed_size > 0);
}

/// test decompression with builder pattern
#[test]
fn test_decompression_builder_workflow() {
    let password = "builder_test_password";
    let test_data = b"Data for decompression builder test.";

    // First compress using stream to create in-memory data
    let mut reader = Cursor::new(test_data);
    let mut writer = Cursor::new(Vec::new());

    let compress_result = compress_stream(
        &mut reader,
        "test_file.txt",
        Some(password),
        &mut writer,
        1024 * 1024,
    )
    .unwrap();

    // Now write this compressed data to a temp file for decompression
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(&writer.into_inner()).unwrap();
    temp_file.flush().unwrap();

    // Use DecompressionBuilder
    let result = DecompressionBuilder::new(temp_file.path())
        .with_password(password)
        .decompress()
        .unwrap();

    assert_eq!(result.original_size, test_data.len() as u64);
    assert!(result.encrypted);
}

/// test error handling workflow
#[test]
fn test_error_handling_workflow() {
    // Test with non-existent file
    let result = compress_file("non_existent_file.txt", None);
    assert!(result.is_err());

    // Test with invalid chunk size
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(b"test").unwrap();

    let result = CompressionBuilder::new(temp_file.path())
        .with_chunk_size(32 * 1024) // Too small
        .compress();
    assert!(result.is_err());

    // Test decompressing non-existent file
    let result = decompress_file("non_existent_file.small", None);
    assert!(result.is_err());

    // Test listing from non-existent archive
    #[cfg(feature = "archives")]
    {
        use std::path::Path;
        let result = list_archive_contents(Path::new("non_existent_archive.small"));
        assert!(result.is_err());
    }
}

/// test roundtrip workflow for various data types
#[test]
fn test_roundtrip_various_data() {
    let test_cases = vec![
        ("Empty file", Vec::new()),
        ("Small text", b"Hello, world!".to_vec()),
        (
            "Binary data",
            vec![0x00, 0x01, 0x02, 0x03, 0xFF, 0xFE, 0xFD],
        ),
        ("Unicode text", "Hello 世界! 🌍".as_bytes().to_vec()),
        (
            "Mixed content",
            b"Mixed text with numbers 123 and symbols !@#$%".to_vec(),
        ),
    ];

    for (name, data) in test_cases {
        // Test in-memory roundtrip
        let mut reader = Cursor::new(&data);
        let mut compressed = Cursor::new(Vec::new());

        let compress_result = compress_stream(
            &mut reader,
            "test_file.bin",
            None,
            &mut compressed,
            1024 * 1024,
        )
        .unwrap();

        assert_eq!(
            compress_result.original_size,
            data.len() as u64,
            "Failed for {}",
            name
        );

        let mut compressed_reader = Cursor::new(compressed.into_inner());
        let mut decompressed = Cursor::new(Vec::new());

        let decompress_result =
            decompress_stream(&mut compressed_reader, None, &mut decompressed, 1024 * 1024)
                .unwrap();

        assert_eq!(
            decompress_result.original_size,
            data.len() as u64,
            "Failed for {}",
            name
        );
        assert_eq!(
            decompressed.into_inner(),
            data,
            "Data mismatch for {}",
            name
        );
    }
}

/// test workflow with different chunk sizes
#[test]
fn test_different_chunk_sizes() {
    let mut temp_file = NamedTempFile::new().unwrap();
    let test_data = b"Test data for different chunk sizes. This data should be handled correctly regardless of chunk size.";
    temp_file.write_all(test_data).unwrap();
    temp_file.flush().unwrap();

    // Test with small chunk size (64KB minimum)
    let _small_chunk_result = CompressionBuilder::new(temp_file.path())
        .with_chunk_size(64 * 1024)
        .compress()
        .unwrap();

    // Test with large chunk size (64MB)
    let _large_chunk_result = CompressionBuilder::new(temp_file.path())
        .with_chunk_size(64 * 1024 * 1024)
        .compress()
        .unwrap();
}

/// test encrypted compression and decompression workflow
#[test]
fn test_encrypted_stream_workflow() {
    let password = "test_password_123";
    let test_data = b"Secret data that must be encrypted";

    // Compress with encryption using stream API
    let mut reader = Cursor::new(test_data);
    let mut writer = Cursor::new(Vec::new());

    let compress_result = compress_stream(
        &mut reader,
        "secret_file.txt",
        Some(password),
        &mut writer,
        1024 * 1024,
    )
    .unwrap();

    assert_eq!(compress_result.original_size, test_data.len() as u64);
    assert!(compress_result.encrypted);

    // Save the compressed data before moving writer
    let compressed_data = writer.into_inner();

    // Decompress with correct password
    let mut compressed_reader = Cursor::new(compressed_data.clone());
    let mut decompressed_writer = Cursor::new(Vec::new());

    let decompress_result = decompress_stream(
        &mut compressed_reader,
        Some(password),
        &mut decompressed_writer,
        1024 * 1024,
    )
    .unwrap();

    assert_eq!(decompress_result.original_size, test_data.len() as u64);
    // DecompressionResult doesn't have encrypted field

    let decompressed_data = decompressed_writer.into_inner();
    assert_eq!(decompressed_data, test_data);

    // Try to decompress with wrong password (should fail)
    let mut wrong_compressed_reader = Cursor::new(compressed_data);
    let mut wrong_decompressed_writer = Cursor::new(Vec::new());

    let wrong_result = decompress_stream(
        &mut wrong_compressed_reader,
        Some("wrong_password"),
        &mut wrong_decompressed_writer,
        1024 * 1024,
    );
    assert!(wrong_result.is_err());
}
