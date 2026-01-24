#[cfg(test)]
mod tests {
    use crate::archive::{extract_archive, list_archive_contents};
    use crate::crypto::DEFAULT_CHUNK_SIZE;
    use crate::headers::Headers;
    use std::fs::{self, File};
    use std::io::{BufReader, BufWriter, Seek, SeekFrom, Write};
    use std::path::Path;
    use tempfile::tempdir;

    // Helper function to recursively compare two directories
    fn compare_dirs(path1: &Path, path2: &Path) -> std::io::Result<()> {
        // Check if both paths are directories
        assert!(path1.is_dir(), "Path1 is not a directory: {:?}", path1);
        assert!(path2.is_dir(), "Path2 is not a directory: {:?}", path2);

        for entry1 in fs::read_dir(path1)? {
            let entry1 = entry1?;
            let path1_entry = entry1.path();
            let relative_path = path1_entry.strip_prefix(path1).unwrap();
            let path2_entry = path2.join(relative_path);

            if path1_entry.is_dir() {
                // Only recurse if the directory exists in path2
                if path2_entry.is_dir() {
                    compare_dirs(&path1_entry, &path2_entry)?;
                } else {
                    // This case should ideally not happen for non-empty dirs.
                    // For empty dirs in path1, extract_archive might not create them, which is acceptable.
                    // If an empty directory is critical, consider creating it explicitly during extraction for comparison.
                }
            } else {
                // Compare files
                assert!(
                    path2_entry.is_file(),
                    "File not found in path2: {:?}",
                    path2_entry
                );
                let content1 = fs::read(&path1_entry)?;
                let content2 = fs::read(&path2_entry)?;
                assert_eq!(
                    content1, content2,
                    "File content mismatch for {:?}",
                    relative_path
                );
            }
        }

        // Ensure no extra files/dirs in path2 (that were not in path1)
        for entry2 in fs::read_dir(path2)? {
            let entry2 = entry2?;
            let path2_entry = entry2.path();
            let relative_path = path2_entry.strip_prefix(path2).unwrap();
            let path1_entry = path1.join(relative_path);
            if !path1_entry.exists() {
                // Allow for empty directories not created by extraction, but flag if a file appears out of nowhere
                if path2_entry.is_file() {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Extra file found in extracted archive: {:?}", relative_path),
                    ));
                }
            }
        }
        Ok(())
    }

    #[test]
    fn test_create_and_extract_unencrypted_archive() {
        let tmp_dir = tempdir().unwrap();
        let input_dir = tmp_dir.path().join("input_archive");
        let output_dir = tmp_dir.path().join("output_extraction");

        fs::create_dir_all(&input_dir.join("subdir_a")).unwrap();
        fs::write(input_dir.join("file1.txt"), b"Hello from file1").unwrap();
        fs::write(input_dir.join("subdir_a/file2.txt"), b"Content of file2").unwrap();
        fs::write(input_dir.join("empty_file.bin"), b"").unwrap();

        fs::create_dir_all(&output_dir).unwrap();

        let archive_path = tmp_dir.path().join("test_archive.small");
        {
            let mut archive_writer = BufWriter::new(File::create(&archive_path).unwrap());
            create_archive(&input_dir, None, &mut archive_writer, DEFAULT_CHUNK_SIZE).unwrap();
            archive_writer.flush().unwrap();
        }

        extract_archive(&archive_path, &output_dir, None).unwrap();

        compare_dirs(&input_dir, &output_dir).unwrap();
    }

    #[test]
    fn test_create_and_extract_encrypted_archive() {
        let tmp_dir = tempdir().unwrap();
        let input_dir = tmp_dir.path().join("input_encrypted_archive");
        let output_dir = tmp_dir.path().join("output_encrypted_extraction");
        let password = "archivepassword";
        let wrong_password = "wrongpassword";

        fs::create_dir_all(&input_dir.join("subdir_b")).unwrap();
        fs::write(input_dir.join("secret_file1.txt"), b"Top secret content").unwrap();
        fs::write(
            input_dir.join("subdir_b/secret_file2.bin"),
            &[0xDE, 0xAD, 0xBE, 0xEF],
        )
        .unwrap();

        fs::create_dir_all(&output_dir).unwrap();

        let archive_path = tmp_dir.path().join("encrypted_archive.small");
        {
            let mut archive_writer = BufWriter::new(File::create(&archive_path).unwrap());
            create_archive(
                &input_dir,
                Some(password),
                &mut archive_writer,
                DEFAULT_CHUNK_SIZE,
            )
            .unwrap();
            archive_writer.flush().unwrap();
        }

        // Test with correct password
        extract_archive(&archive_path, &output_dir, Some(password)).unwrap();
        compare_dirs(&input_dir, &output_dir).unwrap();

        // Test with wrong password (should fail)
        let output_dir_wrong_pass = tmp_dir.path().join("output_wrong_pass");
        fs::create_dir_all(&output_dir_wrong_pass).unwrap();
        let result = extract_archive(&archive_path, &output_dir_wrong_pass, Some(wrong_password));
        assert!(result.is_err()); // Expecting an error due to wrong password

        // Test without password (should fail)
        let output_dir_no_pass = tmp_dir.path().join("output_no_pass");
        fs::create_dir_all(&output_dir_no_pass).unwrap();
        let result_no_pass = extract_archive(&archive_path, &output_dir_no_pass, None);
        assert!(result_no_pass.is_err()); // Expecting an error due to no password
    }

    #[test]
    fn test_list_archive_contents() {
        let tmp_dir = tempdir().unwrap();
        let input_dir = tmp_dir.path().join("input_list_archive");
        fs::create_dir_all(&input_dir.join("sub_dir")).unwrap();
        fs::write(input_dir.join("a.txt"), b"file a").unwrap();
        fs::write(input_dir.join("sub_dir/b.txt"), b"file b").unwrap();
        fs::write(input_dir.join("c.bin"), b"").unwrap();

        let archive_path = tmp_dir.path().join("list_test.small");
        {
            let mut archive_writer = BufWriter::new(File::create(&archive_path).unwrap());
            create_archive(&input_dir, None, &mut archive_writer, DEFAULT_CHUNK_SIZE).unwrap();
            archive_writer.flush().unwrap();
        }

        let (_, listed_files) = crate::archive::list_archive_contents(&archive_path).unwrap();

        let mut expected_files = vec![
            "a.txt".to_string(),
            "sub_dir/b.txt".to_string(),
            "c.bin".to_string(),
        ];
        expected_files.sort();

        let mut actual_files_sorted = listed_files;
        actual_files_sorted.sort();

        assert_eq!(actual_files_sorted, expected_files);

        // Test with a single file archive
        let single_file_path = tmp_dir.path().join("single_file.txt");
        fs::write(&single_file_path, b"single file content").unwrap();
        let single_archive_path = tmp_dir.path().join("single_archive.small");
        {
            let mut archive_writer = BufWriter::new(File::create(&single_archive_path).unwrap());
            crate::huffman::encoder::encode(
                &mut BufReader::new(File::open(&single_file_path).unwrap()),
                "single_file.txt",
                None,
                &mut archive_writer,
                DEFAULT_CHUNK_SIZE,
            )
            .unwrap();
            archive_writer.flush().unwrap();
        }
        let (_, listed_single_file) =
            crate::archive::list_archive_contents(&single_archive_path).unwrap();
        let file_names: Vec<String> = listed_single_file.iter().map(|f| f.path.clone()).collect();
        assert_eq!(file_names, vec!["single_file.txt".to_string()]);
    }

    #[test]
    fn test_create_archive_with_multiple_files() {
        let tmp_dir = tempdir().unwrap();

        // Create a new archive with multiple files using ArchiveBuilder
        let final_archive_path = tmp_dir.path().join("multiple_files.small");
        let archive_info = crate::archive::ArchiveBuilder::new()
            .add_file("file1.txt", b"Content of file1")
            .unwrap()
            .add_file("file2.txt", b"Content of new file2")
            .unwrap()
            .build(&final_archive_path)
            .unwrap();

        assert_eq!(archive_info.file_count, 2);

        // Extract and verify
        let output_dir = tmp_dir.path().join("output");
        extract_archive(&final_archive_path, &output_dir, None).unwrap();

        assert!(output_dir.join("file1.txt").exists());
        assert!(output_dir.join("file2.txt").exists());
        assert_eq!(
            fs::read_to_string(output_dir.join("file1.txt")).unwrap(),
            "Content of file1"
        );
        assert_eq!(
            fs::read_to_string(output_dir.join("file2.txt")).unwrap(),
            "Content of new file2"
        );
        encoded_new_file_buffer.seek(SeekFrom::Start(0)).unwrap(); // Rewind

        let final_archive_path = tmp_dir.path().join("final.small");
        {
            let mut existing_archive_reader =
                BufReader::new(File::open(&initial_archive_path).unwrap());
            let mut new_content_reader = encoded_new_file_buffer;
            let mut final_archive_writer =
                BufWriter::new(File::create(&final_archive_path).unwrap());
            crate::archive::add_to_archive(
                &mut existing_archive_reader,
                &mut new_content_reader,
                &mut final_archive_writer,
                DEFAULT_CHUNK_SIZE,
            )
            .unwrap();
            final_archive_writer.flush().unwrap();
        }

        // Extract and verify
        let output_dir = tmp_dir.path().join("output_add_test");
        fs::create_dir_all(&output_dir).unwrap();
        let mut final_archive_reader = BufReader::new(File::open(&final_archive_path).unwrap());
        let master_header = Headers::from_reader(&mut final_archive_reader).unwrap();
        extract_archive(
            master_header,
            &mut final_archive_reader,
            &output_dir,
            None,
            DEFAULT_CHUNK_SIZE,
        )
        .unwrap();

        assert!(output_dir.join("file1.txt").exists());
        assert!(output_dir.join("file2.txt").exists());
        assert_eq!(
            fs::read(output_dir.join("file1.txt")).unwrap(),
            b"Content of file1"
        );
        assert_eq!(
            fs::read(output_dir.join("file2.txt")).unwrap(),
            new_file_content
        );
    }

    #[test]
    fn test_extract_single_file_from_archive() {
        let tmp_dir = tempdir().unwrap();
        let input_dir = tmp_dir.path().join("input_extract_single");
        fs::create_dir_all(&input_dir.join("sub_dir")).unwrap();
        fs::write(input_dir.join("file1.txt"), b"Content of file 1").unwrap();
        fs::write(input_dir.join("sub_dir/file2.log"), b"Log data for file 2").unwrap();
        fs::write(input_dir.join("file3.csv"), b"a,b,c\n1,2,3").unwrap();

        let archive_path = tmp_dir.path().join("single_extract.small");
        {
            use crate::archive::ArchiveBuilder;
            ArchiveBuilder::new()
                .add_file("file1.txt", b"Content of file 1")
                .unwrap()
                .add_file("sub_dir/file2.log", b"Log data for file 2")
                .unwrap()
                .add_file("file3.csv", b"a,b,c\n1,2,3")
                .unwrap()
                .build(&archive_path)
                .unwrap();
        }

        let output_file_path = tmp_dir.path().join("extracted_file2.log");
        let file_to_extract = "sub_dir/file2.log";

        crate::archive::extract_file(&archive_path, file_to_extract, &output_file_path, None)
            .unwrap();

        assert!(output_file_path.exists());
        assert_eq!(fs::read(&output_file_path).unwrap(), b"Log data for file 2");

        // Test extracting a non-existent file (should fail)
        let non_existent_output_path = tmp_dir.path().join("non_existent.txt");
        let result = {
            let mut archive_reader = BufReader::new(File::open(&archive_path).unwrap());
            crate::archive::extract_file(
                &mut archive_reader,
                "non_existent.txt",
                &non_existent_output_path,
                None,
                DEFAULT_CHUNK_SIZE,
            )
        };
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
        assert!(!non_existent_output_path.exists());

        // Test extracting an encrypted file (password required)
        let encrypted_input_dir = tmp_dir.path().join("input_encrypted_extract_single");
        fs::create_dir_all(&encrypted_input_dir).unwrap();
        fs::write(encrypted_input_dir.join("secret.txt"), b"super secret!").unwrap();
        let encrypted_archive_path = tmp_dir.path().join("encrypted_single_extract.small");
        let password = "pass";
        {
            use crate::archive::ArchiveBuilder;
            ArchiveBuilder::new()
                .with_password(password)
                .add_file("secret.txt", b"super secret!")
                .unwrap()
                .build(&encrypted_archive_path)
                .unwrap();
        }

        let encrypted_output_file_path = tmp_dir.path().join("encrypted_output_file.txt");
        crate::archive::extract_file(
            &encrypted_archive_path,
            "secret.txt",
            &encrypted_output_file_path,
            Some(password),
        )
        .unwrap();

        let encrypted_output_file_path = tmp_dir.path().join("extracted_secret.txt");
        let result_encrypted = crate::archive::extract_file(
            &encrypted_archive_path,
            "secret.txt",
            &encrypted_output_file_path,
            Some(password),
        );
        assert!(result_encrypted.is_ok());
        assert_eq!(
            fs::read(&encrypted_output_file_path).unwrap(),
            b"super secret!"
        );

        // Test extracting encrypted file with wrong password (should fail)
        let result_encrypted_wrong_pass = crate::archive::extract_file(
            &encrypted_archive_path,
            "secret.txt",
            &tmp_dir.path().join("fail_secret.txt"),
            Some("wrong"),
        );
        assert!(result_encrypted_wrong_pass.is_err());
    }
}
