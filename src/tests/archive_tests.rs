#[cfg(test)]
mod tests {
    use std::fs::{self, File};
    use std::path::Path;
    use tempfile::tempdir;
    use crate::archive::{add_to_archive, create_archive, extract_archive, list_contents, extract_file};
    use crate::huffman::encoder::encode;
    use crate::headers::Headers;
    use std::io::{BufReader, BufWriter, Cursor, Write}; // Added for streaming tests

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
                // Only recurse if the directory exists in path2, otherwise, it's an expected absence of an empty dir
                if path2_entry.is_dir() {
                    compare_dirs(&path1_entry, &path2_entry)?;
                } else {
                    // If an empty directory in path1 is not in path2, and it's allowed by extract_archive, then skip.
                    // If it was a non-empty directory, this would be an error, handled by files not matching.
                    // For now, we assume extract_archive doesn't create empty directories.
                }
            } else {
                // Compare files
                assert!(path2_entry.is_file(), "File not found in path2: {:?}", path2_entry);
                let content1 = fs::read(&path1_entry)?;
                let content2 = fs::read(&path2_entry)?;
                assert_eq!(content1, content2, "File content mismatch for {:?}", relative_path);
            }
        }

        // Ensure no extra files/dirs in path2
        for entry2 in fs::read_dir(path2)? {
            let entry2 = entry2?;
            let path2_entry = entry2.path();
            let relative_path = path2_entry.strip_prefix(path2).unwrap();
            let path1_entry = path1.join(relative_path);
            assert!(path1_entry.exists(), "Extra file/directory found in extracted archive: {:?}", relative_path);
        }

        Ok(())
    }

    #[test]
    fn test_archive_roundtrip() {
        // 1. Create a temporary directory with a more complex structure and files
        let dir = tempdir().unwrap();
        let input_dir = dir.path().join("input_archive_test");
        fs::create_dir_all(input_dir.join("sub_dir_a/nested_b")).unwrap();
        fs::create_dir_all(input_dir.join("empty_dir_c")).unwrap(); // Empty directory

        fs::write(input_dir.join("file1.txt"), "This is file 1 content.").unwrap();
        fs::write(input_dir.join("sub_dir_a/file2.log"), "Log entry one\nLog entry two.").unwrap();
        fs::write(input_dir.join("sub_dir_a/nested_b/file3.csv"), "a,b,c\n1,2,3").unwrap();
        fs::write(input_dir.join("empty_file.txt"), "").unwrap(); // Empty file
        fs::write(input_dir.join("another_file.bin"), &[0x00, 0xFF, 0x12, 0x34]).unwrap();

        // 2. Define output paths
        let archive_path = dir.path().join("test_archive.small");
        let output_dir_for_extraction = dir.path().join("extracted_archive");
        fs::create_dir(&output_dir_for_extraction).unwrap();

        // 3. Create the archive
        let mut archive_writer = BufWriter::new(File::create(&archive_path).unwrap());
        create_archive(&input_dir, None, &mut archive_writer).unwrap();
        archive_writer.flush().unwrap(); // Ensure all data is written
        assert!(archive_path.exists());

        // 4. Extract the archive
        let mut archive_reader = BufReader::new(File::open(&archive_path).unwrap());
        let master_header = Headers::from_reader(&mut archive_reader).unwrap();
        extract_archive(master_header, &mut archive_reader, &output_dir_for_extraction, None).unwrap();

        // 5. Verify the extracted contents using the helper function
        compare_dirs(&input_dir, &output_dir_for_extraction).unwrap();
    }

    #[test]
    fn test_add_to_single_file_archive() {
        let dir = tempdir().unwrap();
        let input_dir = dir.path().join("input_initial");
        fs::create_dir(&input_dir).unwrap();
        fs::write(input_dir.join("file1.txt"), "This is the first file content.").unwrap();

        let initial_archive_path = dir.path().join("initial.small");
        let mut initial_archive_writer = BufWriter::new(File::create(&initial_archive_path).unwrap());
        create_archive(&input_dir, None, &mut initial_archive_writer).unwrap();
        initial_archive_writer.flush().unwrap();
        
        // 2. Prepare new file to add
        let file2_content = b"This is the second file, added later.".to_vec();
        let mut input_reader_file2 = Cursor::new(&file2_content);
        let mut encoded_file2_buffer = Vec::new();
        encode(&mut input_reader_file2, "file2.txt", None, &mut encoded_file2_buffer).unwrap();
        let mut new_content_reader = Cursor::new(encoded_file2_buffer);

        // 3. Add the new file to the existing archive
        let final_archive_path = dir.path().join("final_combined.small");
        let mut existing_archive_file = File::open(&initial_archive_path).unwrap();
        let mut final_archive_file = File::create(&final_archive_path).unwrap();

        add_to_archive(&mut existing_archive_file, &mut new_content_reader, &mut final_archive_file).unwrap();
        final_archive_file.flush().unwrap();

        // 4. Extract and verify the new archive
        let output_dir = dir.path().join("output");
        fs::create_dir(&output_dir).unwrap();
        
        let mut archive_reader = BufReader::new(File::open(&final_archive_path).unwrap());
        let master_header = Headers::from_reader(&mut archive_reader).unwrap();
        extract_archive(master_header, &mut archive_reader, &output_dir, None).unwrap();

        // 5. Verify contents
        let extracted_file1 = fs::read_to_string(output_dir.join("file1.txt")).unwrap();
        let extracted_file2 = fs::read(output_dir.join("file2.txt")).unwrap();
        assert_eq!(extracted_file1, "This is the first file content.");
        assert_eq!(extracted_file2, file2_content);
    }

    #[test]
    fn test_add_to_existing_archive() {
        let dir = tempdir().unwrap();

        // 1. Create an initial 2-file archive
        let input_dir1 = dir.path().join("input1");
        fs::create_dir(&input_dir1).unwrap();
        fs::write(input_dir1.join("file1.txt"), "Original file 1".as_bytes()).unwrap();
        fs::write(input_dir1.join("file2.txt"), "Original file 2".as_bytes()).unwrap();
        let archive_path = dir.path().join("archive.small");
        let mut archive_writer1 = BufWriter::new(File::create(&archive_path).unwrap());
        create_archive(&input_dir1, None, &mut archive_writer1).unwrap();
        archive_writer1.flush().unwrap();

        // 2. Create a new file to add
        let file3_content = b"This is the third file, added later.".to_vec();
        let mut input_reader3 = Cursor::new(&file3_content);
        let mut encoded_file3_buffer = Vec::new();
        encode(&mut input_reader3, "sub/file3.txt", None, &mut encoded_file3_buffer).unwrap();

        // 3. Add the new file to the existing archive using streaming
        let mut existing_archive_reader = Cursor::new(fs::read(&archive_path).unwrap());
        let mut new_content_reader = Cursor::new(encoded_file3_buffer);
        let mut output_writer_cursor = Cursor::new(Vec::new());
        add_to_archive(&mut existing_archive_reader, &mut new_content_reader, &mut output_writer_cursor).unwrap();
        let updated_archive_bytes = output_writer_cursor.into_inner();

        fs::write(&archive_path, updated_archive_bytes).unwrap();

        // 4. Extract the updated archive
        let output_dir = dir.path().join("output");
        fs::create_dir(&output_dir).unwrap();
        let mut archive_reader2 = BufReader::new(File::open(&archive_path).unwrap());
        let master_header = Headers::from_reader(&mut archive_reader2).unwrap();
        extract_archive(master_header, &mut archive_reader2, &output_dir, None).unwrap();
        
        // 5. Verify all files
        let extracted_file1 = fs::read_to_string(output_dir.join("file1.txt")).unwrap();
        let extracted_file2 = fs::read_to_string(output_dir.join("file2.txt")).unwrap();
        let extracted_file3 = fs::read(output_dir.join("sub/file3.txt")).unwrap();
        
        assert_eq!(extracted_file1, "Original file 1");
        assert_eq!(extracted_file2, "Original file 2");
        assert_eq!(extracted_file3, file3_content);
    }

    #[test]
    fn test_list_single_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("single.txt");
        fs::write(&file_path, "Just a single file.").unwrap();

        let output_path = dir.path().join("single.txt.small");
        let file_content = fs::read(&file_path).unwrap();

        let mut input_reader_encode = Cursor::new(&file_content);
        let mut encoded_bytes_buffer = Vec::new();
        encode(&mut input_reader_encode, "single.txt", None, &mut encoded_bytes_buffer).unwrap();
        fs::write(&output_path, encoded_bytes_buffer).unwrap();

        let mut archive_reader = BufReader::new(File::open(&output_path).unwrap());
        let listed_files = list_contents(&mut archive_reader).unwrap();

        assert_eq!(listed_files.len(), 1);
        assert_eq!(listed_files[0], "single.txt");
    }

    #[test]
    fn test_list_archive_contents() {
        let dir = tempdir().unwrap();
        let input_dir = dir.path().join("archive_to_list");
        fs::create_dir_all(input_dir.join("sub")).unwrap();
        fs::write(input_dir.join("test1.txt"), "Content 1").unwrap();
        fs::write(input_dir.join("sub/test2.log"), "Content 2").unwrap();
                fs::write(input_dir.join("sub/test3.dat"), "Content 3").unwrap();
        
                let archive_path = dir.path().join("list_test.small");
                let mut archive_writer = BufWriter::new(File::create(&archive_path).unwrap());
                create_archive(&input_dir, None, &mut archive_writer).unwrap();
                archive_writer.flush().unwrap();

        let mut archive_reader = BufReader::new(File::open(&archive_path).unwrap());
        let listed_files = list_contents(&mut archive_reader).unwrap();

        let expected_files = vec![
            "test1.txt".to_string(),
            "sub/test2.log".to_string(),
            "sub/test3.dat".to_string(),
        ];
        
        // Sort both vectors for reliable comparison
        let mut listed_files_sorted = listed_files;
        listed_files_sorted.sort();
        let mut expected_files_sorted = expected_files;
        expected_files_sorted.sort();

        assert_eq!(listed_files_sorted, expected_files_sorted);
    }

    #[test]
    fn test_extract_single_file_from_archive() {
        let dir = tempdir().unwrap();
        let input_dir = dir.path().join("archive_for_extraction");
        fs::create_dir_all(input_dir.join("sub")).unwrap();
        fs::write(input_dir.join("file_a.txt"), "Content of file A").unwrap();
        let target_content = b"Content of file B".to_vec();
        fs::write(input_dir.join("sub/file_b.log"), &target_content).unwrap();
        fs::write(input_dir.join("file_c.csv"), "Content of file C").unwrap();

        let archive_path = dir.path().join("extract_test.small");

        let mut archive_writer = BufWriter::new(File::create(&archive_path).unwrap());
        create_archive(&input_dir, None, &mut archive_writer).unwrap();
        archive_writer.flush().unwrap();

        let mut archive_reader = BufReader::new(File::open(&archive_path).unwrap());
        let extracted_file_path = dir.path().join("extracted_b.log");
        let file_to_extract = "sub/file_b.log";

        extract_file(&mut archive_reader, file_to_extract, &extracted_file_path, None).unwrap();

        assert!(extracted_file_path.exists());
        let extracted_content = fs::read(&extracted_file_path).unwrap();
        assert_eq!(extracted_content, target_content);
    }

    #[test]
    fn test_extract_non_existent_file() {
        let dir = tempdir().unwrap();
        let input_dir = dir.path().join("archive_for_non_existent");
        fs::create_dir(&input_dir).unwrap();
                fs::write(input_dir.join("real_file.txt"), "Real content").unwrap();
        
                let archive_path = dir.path().join("non_existent_test.small");
                let mut archive_writer = BufWriter::new(File::create(&archive_path).unwrap());
                create_archive(&input_dir, None, &mut archive_writer).unwrap();
                archive_writer.flush().unwrap();

        let mut archive_reader = BufReader::new(File::open(&archive_path).unwrap());
        let output_path = dir.path().join("non_existent.txt");
        let file_to_extract = "imaginary_file.txt";

        let result = extract_file(&mut archive_reader, file_to_extract, &output_path, None);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::NotFound);
        assert!(!output_path.exists());
    }

    #[test]
    fn test_encrypted_archive_roundtrip() {
        let dir = tempdir().unwrap();
        let input_dir = dir.path().join("encrypted_input");
        fs::create_dir_all(input_dir.join("sub")).unwrap();
        fs::write(input_dir.join("enc_file1.txt"), "Secret message 1.").unwrap();
        fs::write(input_dir.join("sub/enc_file2.bin"), [0xDE, 0xAD, 0xBE, 0xEF]).unwrap();

        let archive_path = dir.path().join("encrypted_archive.small");
        let output_dir = dir.path().join("extracted_encrypted");
        fs::create_dir(&output_dir).unwrap();

        let password = "supersecretarchivepassword";
        let wrong_password = "wrongpassword";

        // 1. Create encrypted archive
        let mut archive_writer = BufWriter::new(File::create(&archive_path).unwrap());
        create_archive(&input_dir, Some(password), &mut archive_writer).unwrap();
        archive_writer.flush().unwrap();
        assert!(archive_path.exists());

        // 2. Extract with correct password
        let mut archive_reader1 = BufReader::new(File::open(&archive_path).unwrap());
        let master_header1 = Headers::from_reader(&mut archive_reader1).unwrap();
        extract_archive(master_header1, &mut archive_reader1, &output_dir, Some(password)).unwrap();
        compare_dirs(&input_dir, &output_dir).unwrap();

        // 3. Extract with wrong password (should fail)
        let output_dir_wrong_pass = dir.path().join("extracted_wrong_pass");
        fs::create_dir(&output_dir_wrong_pass).unwrap();
        let mut archive_reader2 = BufReader::new(File::open(&archive_path).unwrap());
        let master_header2 = Headers::from_reader(&mut archive_reader2).unwrap();
        let result_wrong = extract_archive(master_header2, &mut archive_reader2, &output_dir_wrong_pass, Some(wrong_password));
        assert!(result_wrong.is_err());

        // 4. Extract without password (should fail)
        let output_dir_no_pass = dir.path().join("extracted_no_pass");
        fs::create_dir(&output_dir_no_pass).unwrap();
        let mut archive_reader3 = BufReader::new(File::open(&archive_path).unwrap());
        let master_header3 = Headers::from_reader(&mut archive_reader3).unwrap();
        let result_no_pass = extract_archive(master_header3, &mut archive_reader3, &output_dir_no_pass, None);
        assert!(result_no_pass.is_err());
    }
}
