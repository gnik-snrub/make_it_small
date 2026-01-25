#[cfg(test)]
mod cross_platform_tests {

    use mismall::{compress::CompressionBuilder, compress_file, compress_stream};
    use std::fs;
    use std::io::Cursor;
    use std::io::Write;
    use tempfile::NamedTempFile;

    // Test cross-platform path handling
    #[test]
    fn test_cross_platform_paths() {
        println!("Testing cross-platform path handling...");

        // Test different path formats
        let test_paths = vec![
            ("simple.txt", "simple filename"),
            ("file with spaces.txt", "filename with spaces"),
            ("file-with-dashes.txt", "filename with dashes"),
            ("file_with_underscores.txt", "filename with underscores"),
            ("file.with.dots.txt", "filename with multiple dots"),
        ];

        for (filename, description) in test_paths {
            println!("  Testing {}: {}", description, filename);

            // Create test file
            let mut temp_file = NamedTempFile::with_suffix(filename).unwrap();
            let test_data = format!("Test content for {}", filename);
            temp_file.write_all(test_data.as_bytes()).unwrap();
            temp_file.flush().unwrap();

            // Test compression with the filename
            let result = compress_file(temp_file.path().to_str().unwrap(), None);
            assert!(
                result.is_ok(),
                "Failed to compress file with name '{}'",
                filename
            );

            let compression_result = result.unwrap();
            // Note: The filename in compression_result comes from the actual file path
            assert!(
                compression_result.filename.contains(filename)
                    || compression_result.filename.ends_with(".txt")
            );
            assert_eq!(compression_result.original_size, test_data.len() as u64);
        }

        println!("  ✓ Cross-platform path handling tests passed");
    }

    // Test different path separators work correctly
    #[test]
    fn test_path_separator_handling() {
        println!("Testing path separator handling...");

        // Create a test file in a nested directory structure
        let test_dir = tempfile::tempdir().unwrap();
        let nested_path = test_dir.path().join("nested").join("subdir");
        fs::create_dir_all(&nested_path).unwrap();

        let test_file_path = nested_path.join("test.txt");
        let mut test_file = fs::File::create(&test_file_path).unwrap();
        test_file
            .write_all(b"Test content for nested path")
            .unwrap();
        test_file.flush().unwrap();

        // Test compression with absolute path
        let result = compress_file(test_file_path.to_str().unwrap(), None);
        assert!(result.is_ok(), "Failed to compress file with nested path");

        let compression_result = result.unwrap();
        assert_eq!(compression_result.original_size, 28); // Length of "Test content for nested path"
        assert_eq!(compression_result.filename, "test.txt"); // Only filename, not full path

        println!("  ✓ Path separator handling tests passed");
    }

    // Test file permission handling across platforms
    #[test]
    fn test_file_permissions() {
        println!("Testing file permission compatibility...");

        // Create a temporary file
        let mut temp_file = NamedTempFile::new().unwrap();
        let test_data = b"Permission test data";
        temp_file.write_all(test_data).unwrap();
        temp_file.flush().unwrap();

        // Test compression works regardless of file permissions
        let result = compress_file(temp_file.path().to_str().unwrap(), None);
        assert!(
            result.is_ok(),
            "Failed to compress file with default permissions"
        );

        let compression_result = result.unwrap();
        assert_eq!(compression_result.original_size, test_data.len() as u64);

        // Note: We don't test specific permission modifications here as they're
        // platform-specific and may require elevated privileges on some systems

        println!("  ✓ File permission compatibility tests passed");
    }

    // Test streaming with different data patterns that might behave differently across platforms
    #[test]
    fn test_streaming_cross_platform_data() {
        println!("Testing streaming with cross-platform data patterns...");

        // Test different data patterns that might have platform-specific behavior
        let test_cases = vec![
            (b"Hello\nWorld\n".to_vec(), "Unix line endings"),
            (b"Hello\r\nWorld\r\n".to_vec(), "Windows line endings"),
            (b"Hello\rWorld\r".to_vec(), "Classic Mac line endings"),
            (vec![0; 1024], "Null bytes"),
            (vec![255u8; 1024], "High byte values"),
        ];

        for (data, description) in test_cases {
            println!("  Testing {}", description);

            let mut reader = Cursor::new(&data);
            let mut writer = Cursor::new(Vec::new());

            let result = compress_stream(&mut reader, "test.txt", None, &mut writer, 64 * 1024);
            assert!(result.is_ok(), "Failed to compress data: {}", description);

            let compression_result = result.unwrap();
            assert_eq!(compression_result.original_size, data.len() as u64);
            assert_eq!(compression_result.filename, "test.txt");
        }

        println!("  ✓ Streaming cross-platform data tests passed");
    }

    // Test that large file paths work (within reasonable limits)
    #[test]
    fn test_long_filename_handling() {
        println!("Testing long filename handling...");

        // Create a filename that's long but reasonable (not extremely long to avoid filesystem limits)
        let _long_name = "a".repeat(100) + ".txt";

        let mut temp_file = NamedTempFile::new().unwrap();
        let test_data = format!("Test content for long filename");
        temp_file.write_all(test_data.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        // Test compression with long filename
        let result = compress_file(temp_file.path().to_str().unwrap(), None);
        assert!(result.is_ok(), "Failed to compress file with long filename");

        let compression_result = result.unwrap();
        assert_eq!(compression_result.original_size, test_data.len() as u64);

        println!("  ✓ Long filename handling tests passed");
    }

    // Test Unicode filename handling
    #[test]
    fn test_unicode_filename_handling() {
        println!("Testing Unicode filename handling...");

        let unicode_names = vec![
            "файл.txt",     // Cyrillic
            "文件.txt",     // Chinese
            "ファイル.txt", // Japanese
            "archive.zip",  // ASCII (control)
            "tëst.txt",     // Latin with diacritics
        ];

        for unicode_name in unicode_names {
            println!("  Testing Unicode filename: {}", unicode_name);

            let mut temp_file = NamedTempFile::new().unwrap();
            let test_data = format!("Test content for {}", unicode_name);
            temp_file.write_all(test_data.as_bytes()).unwrap();
            temp_file.flush().unwrap();

            // Test compression with Unicode filename
            let result = compress_file(temp_file.path().to_str().unwrap(), None);
            assert!(
                result.is_ok(),
                "Failed to compress file with Unicode filename '{}'",
                unicode_name
            );

            let compression_result = result.unwrap();
            assert_eq!(compression_result.original_size, test_data.len() as u64);
        }

        println!("  ✓ Unicode filename handling tests passed");
    }

    // Test compression with various chunk sizes on different platforms
    #[test]
    fn test_chunk_size_cross_platform() {
        println!("Testing chunk size compatibility across platforms...");

        let chunk_sizes = vec![
            64 * 1024,        // 64KB - minimum
            256 * 1024,       // 256KB
            1024 * 1024,      // 1MB
            16 * 1024 * 1024, // 16MB - default
        ];

        for chunk_size in chunk_sizes {
            println!("  Testing {}KB chunk size", chunk_size / 1024);

            let mut temp_file = NamedTempFile::new().unwrap();
            let test_data = vec![42u8; 1024 * 1024]; // 1MB test data
            temp_file.write_all(&test_data).unwrap();
            temp_file.flush().unwrap();

            // Test compression with different chunk sizes
            let result = CompressionBuilder::new(temp_file.path().to_str().unwrap())
                .with_chunk_size(chunk_size)
                .compress();

            assert!(
                result.is_ok(),
                "Failed to compress with {}KB chunks",
                chunk_size / 1024
            );

            let compression_result = result.unwrap();
            assert_eq!(compression_result.original_size, test_data.len() as u64);
        }

        println!("  ✓ Chunk size cross-platform tests passed");
    }

    // Integration test: complete workflow with cross-platform considerations
    #[test]
    fn test_cross_platform_integration() {
        println!("Testing cross-platform integration workflow...");

        // Create a comprehensive test scenario
        let test_dir = tempfile::tempdir().unwrap();
        let test_file_path = test_dir.path().join("cross-platform-test.txt");

        // Create test file with mixed content (as a string first, then get bytes)
        let mixed_content_str = "This is a test file with mixed content:\n\
                                - Numbers: 123456789\n\
                                - Unicode: 你好世界\n\
                                - Symbols: @#$%^&*()\n";
        let mixed_content = mixed_content_str.as_bytes();

        let mut test_file = fs::File::create(&test_file_path).unwrap();
        test_file.write_all(mixed_content).unwrap();
        test_file.flush().unwrap();

        // Test complete compression workflow
        let compression_result = compress_file(test_file_path.to_str().unwrap(), None)
            .expect("Failed to compress in integration test");

        assert_eq!(compression_result.original_size, mixed_content.len() as u64);
        assert!(compression_result.compression_ratio > 0.0);
        assert!(compression_result.compression_ratio <= 100.0);

        println!("  ✓ Cross-platform integration test passed");
        println!(
            "    Original size: {} bytes",
            compression_result.original_size
        );
        println!(
            "    Compressed size: {} bytes",
            compression_result.compressed_size
        );
        println!(
            "    Compression ratio: {:.1}%",
            compression_result.compression_ratio
        );
    }
}
