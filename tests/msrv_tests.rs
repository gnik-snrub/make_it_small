#[cfg(test)]
mod msrv_tests {
    use mismall::{compress::CompressionBuilder, compress_file};
    use std::collections::HashMap;
    use std::fs;
    use std::io::Write;
    use tempfile::NamedTempFile;

    // Test that core functionality works with MSRV
    #[test]
    fn test_msrv_basic_functionality() {
        println!("Testing MSRV basic functionality...");

        // Test basic compression
        let mut temp_file = NamedTempFile::new().unwrap();
        let test_data = b"Hello, MSRV test!";
        temp_file.write_all(test_data).unwrap();
        temp_file.flush().unwrap();

        let result = compress_file(temp_file.path().to_str().unwrap(), None);
        assert!(result.is_ok(), "Basic compression should work with MSRV");

        let compression_result = result.unwrap();
        assert_eq!(compression_result.original_size, test_data.len() as u64);
        assert!(compression_result.compressed_size > 0);

        println!("  ✓ Basic compression works with MSRV");
    }

    // Test builder pattern works with MSRV
    #[test]
    fn test_msrv_builder_pattern() {
        println!("Testing MSRV builder pattern...");

        let mut temp_file = NamedTempFile::new().unwrap();
        let test_data = b"MSRV builder test data with some entropy to compress well";
        temp_file.write_all(test_data).unwrap();
        temp_file.flush().unwrap();

        let result = CompressionBuilder::new(temp_file.path().to_str().unwrap())
            .with_chunk_size(64 * 1024) // Minimum chunk size
            .compress();

        assert!(result.is_ok(), "Builder pattern should work with MSRV");

        let compression_result = result.unwrap();
        assert_eq!(compression_result.original_size, test_data.len() as u64);

        println!("  ✓ Builder pattern works with MSRV");
    }

    // Test that standard library features used are available in MSRV
    #[test]
    fn test_msrv_std_features() {
        println!("Testing MSRV standard library features...");

        // Test HashMap (should be available)
        let mut map = HashMap::new();
        map.insert("test", "value");
        assert_eq!(map.get("test"), Some(&"value"));

        // Test file operations
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"MSRV std test").unwrap();
        temp_file.flush().unwrap();

        let metadata = fs::metadata(temp_file.path()).unwrap();
        assert!(metadata.len() > 0);

        // Test path operations
        let path = temp_file.path();
        assert!(path.exists());
        assert!(path.file_name().is_some());

        println!("  ✓ Standard library features available in MSRV");
    }

    // Test that modern Rust features we use are available
    #[test]
    fn test_msrv_modern_features() {
        println!("Testing MSRV modern Rust features...");

        // Test Option::is_some_and (available since Rust 1.70)
        let opt = Some(42);
        assert!(opt.is_some_and(|x| x > 40));

        // Test Result::is_ok_and (available since Rust 1.76) - but we use older pattern
        let result: Result<i32, &str> = Ok(42);
        assert!(result.is_ok());

        // Test const generics and other features
        let array: [u8; 4] = [1, 2, 3, 4];
        assert_eq!(array.len(), 4);

        println!("  ✓ Modern Rust features available in MSRV");
    }

    // Test that our dependencies' features work with MSRV
    #[test]
    fn test_msrv_dependencies() {
        println!("Testing MSRV dependency compatibility...");

        // Test that we can use key dependencies without issues
        let mut temp_file = NamedTempFile::new().unwrap();

        // Test compression (uses rand, aes-gcm, pbkdf2, etc.)
        let test_data = b"MSRV dependency test with various byte patterns: \x00\xFF\x80\xFE";
        temp_file.write_all(test_data).unwrap();
        temp_file.flush().unwrap();

        let result = compress_file(temp_file.path().to_str().unwrap(), Some("test_password"));
        assert!(result.is_ok(), "Dependencies should work with MSRV");

        let compression_result = result.unwrap();
        assert!(
            compression_result.encrypted,
            "Password-based encryption should work"
        );
        assert_eq!(compression_result.original_size, test_data.len() as u64);

        println!("  ✓ Dependencies compatible with MSRV");
    }

    // Test that our error handling works with MSRV
    #[test]
    fn test_msrv_error_handling() {
        println!("Testing MSRV error handling...");

        // Test with non-existent file
        let result = compress_file("non_existent_file_for_msrv_test.txt", None);
        assert!(result.is_err(), "Should error on non-existent file");

        // Test error handling features we use
        if let Err(error) = result {
            // Test that we can work with our error type
            let error_string = error.to_string();
            assert!(!error_string.is_empty(), "Error should have description");

            // Test that error provides context
            // Note: These features depend on our error module implementation
            println!("  Error: {}", error_string);
        }

        println!("  ✓ Error handling works with MSRV");
    }

    // Integration test for complete MSRV workflow
    #[test]
    fn test_msrv_integration() {
        println!("Testing MSRV integration workflow...");

        // Create test file with realistic content
        let mut temp_file = NamedTempFile::new().unwrap();
        let content = "This is a comprehensive MSRV integration test.\n\
                     It tests various features:\n\
                     - Text compression\n\
                     - Unicode support: 你好世界\n\
                     - Numbers: 123456789\n\
                     - Symbols: @#$%^&*()\n";

        temp_file.write_all(content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        // Test complete compression workflow
        let compression_result = compress_file(
            temp_file.path().to_str().unwrap(),
            Some("msrv_test_password"),
        )
        .expect("MSRV integration compression should work");

        // Verify results
        assert_eq!(compression_result.original_size, content.len() as u64);
        assert!(compression_result.compressed_size > 0);
        assert!(compression_result.compression_ratio > 0.0);
        assert!(compression_result.compression_ratio <= 100.0);
        assert!(compression_result.encrypted);

        println!("  ✓ MSRV integration test passed");
        println!(
            "    Content size: {} bytes",
            compression_result.original_size
        );
        println!(
            "    Compressed: {} bytes",
            compression_result.compressed_size
        );
        println!("    Ratio: {:.1}%", compression_result.compression_ratio);
        println!("    Encrypted: {}", compression_result.encrypted);
    }
}
