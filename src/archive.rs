use std::io::Read;
use std::fs;
use std::io::{Write, Cursor, Seek};
use std::path::{Path, PathBuf};

use crate::huffman::{encoder::encode, decoder::decode};
use crate::headers::Headers;
use crate::flags;

/// Creates an archive from a directory.
pub fn create_archive<W: Write>(
    dir_path: &Path,
    encrypt_password: Option<&str>,
    writer: &mut W,
) -> std::io::Result<()> {
    let all_files = get_all_files(dir_path)?;

    let mut total_original_size: u64 = 0;
    let mut all_encoded_files_buf = Vec::new();

    for file_path in all_files {
        let relative_path = file_path.strip_prefix(dir_path).map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Failed to get relative path for {:?}", file_path),
            )
        })?;

        let file_content = fs::read(&file_path)?;
        total_original_size += file_content.len() as u64;

        let mut file_content_cursor = Cursor::new(&file_content);
        let mut encoded_file_bytes_buffer = Vec::new();

        let _encode_info = encode(
            &mut file_content_cursor,
            &relative_path.to_string_lossy(),
            encrypt_password,
            &mut encoded_file_bytes_buffer,
        ).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

        all_encoded_files_buf.extend_from_slice(&encoded_file_bytes_buffer);
    }

    let mut master_header = Headers::new();
    flags::flip_is_archive(&mut master_header.flags);
    if encrypt_password.is_some() {
        flags::flip_encrypted(&mut master_header.flags);
    }
    master_header.original_size = total_original_size;
    master_header.compressed_size = all_encoded_files_buf.len() as u64;
    master_header.original_file_name = dir_path.file_name().unwrap_or_default().to_string_lossy().to_string();

    let master_header_bytes = master_header.to_bytes();

    writer.write_all(&master_header_bytes)?;
    writer.write_all(&all_encoded_files_buf)?;

    Ok(())
}

/// Extracts an archive to a directory.
pub fn extract_archive<R: Read + Seek>(
    reader: &mut R,
    output_path: &Path,
    decrypt_password: Option<&str>,
) -> std::io::Result<Headers> { // Return the master header
    let master_header = Headers::from_reader(reader)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Failed to parse master header: {}", e)))?;

    if !flags::is_archive(master_header.flags) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Input file is not an archive.",
        ));
    }

    let initial_body_start_position = reader.stream_position()?;
    let archive_body_end_position = initial_body_start_position + master_header.compressed_size;

    // Now loop through embedded files
    loop {
        if reader.stream_position()? >= archive_body_end_position {
            break; // Reached the end of the archive body
        }
        let embedded_header_start_pos = reader.seek(std::io::SeekFrom::Current(0))
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to seek reader: {}", e)))?;

        let embedded_header_result = Headers::from_reader(reader);
        let embedded_header = match embedded_header_result {
            Ok(h) => h,
            Err(e) => {
                if let Some(io_err) = e.downcast_ref::<std::io::Error>() {
                    // Break if we hit EOF precisely at the end of the archive body.
                    // If stream_position is exactly archive_body_end_position and we get EOF, it's fine.
                    // If we get EOF before archive_body_end_position, it means a truncated archive.
                    if io_err.kind() == std::io::ErrorKind::UnexpectedEof {
                        // Check if it's expected EOF at the boundary
                        if reader.stream_position()? == archive_body_end_position {
                            break; // Expected EOF at the end of the archive body
                        } else {
                            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Truncated archive: Unexpected end of file before archive body end.".to_string()));
                        }
                    }
                }
                return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Failed to parse embedded header: {}", e)));
            }
        };

        // Create output path
        let output_file_path = output_path.join(&embedded_header.original_file_name);
        if let Some(parent) = output_file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut output_file = fs::File::create(&output_file_path)?;

        // Reconstruct the stream for decode: embedded_header + compressed_data
        let mut full_file_bytes = embedded_header.clone().to_bytes();
        let mut compressed_data_buf = vec![0u8; embedded_header.compressed_size as usize];
        reader.read_exact(&mut compressed_data_buf)?;
        full_file_bytes.extend_from_slice(&compressed_data_buf);

        let mut full_file_reader = Cursor::new(full_file_bytes);
        
        let _decode_info = decode(&mut full_file_reader, decrypt_password, &mut output_file)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
    }

    Ok(master_header)
}

/// Recursively walks a directory and returns a list of all files.
fn get_all_files(dir_path: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    if dir_path.is_dir() {
        for entry in fs::read_dir(dir_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                files.extend(get_all_files(&path)?);
            } else {
                files.push(path);
            }
        }
    }
    Ok(files)
}

/// Adds a new compressed file or archive to an existing one.
pub fn add_to_archive(
    existing_bytes: Vec<u8>,
    new_bytes: Vec<u8>,
) -> std::io::Result<Vec<u8>> {
    let mut existing_reader = Cursor::new(&existing_bytes);
    let master_header = Headers::from_reader(&mut existing_reader)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Failed to parse header of existing file: {}", e)))?;
    let master_header_len = existing_reader.position() as usize;

    let mut new_content_reader = Cursor::new(&new_bytes);
    let new_content_header = Headers::from_reader(&mut new_content_reader)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Failed to parse header of new content: {}", e)))?;

    let mut new_master_header = Headers::new();
    flags::flip_is_archive(&mut new_master_header.flags);

    if flags::is_encrypted(master_header.flags) || flags::is_encrypted(new_content_header.flags) {
        flags::flip_encrypted(&mut new_master_header.flags);
    }

    let new_body;

    if flags::is_archive(master_header.flags) {
        // Already an archive, just append
        let old_body = &existing_bytes[master_header_len..];
        new_body = [old_body, new_bytes.as_slice()].concat();

        new_master_header.original_size = master_header.original_size + new_content_header.original_size;
    } else {
        // It's a single file, convert to archive
        new_body = [existing_bytes.as_slice(), new_bytes.as_slice()].concat();
        
        let mut old_content_reader_for_single = Cursor::new(&existing_bytes);
        let old_content_header = Headers::from_reader(&mut old_content_reader_for_single)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Failed to parse header of existing file: {}", e)))?;

        new_master_header.original_size = old_content_header.original_size + new_content_header.original_size;
    }

    new_master_header.compressed_size = new_body.len() as u64;
    new_master_header.original_file_name = master_header.original_file_name; // Keep the original archive name

    let mut final_archive_bytes = new_master_header.to_bytes();
    final_archive_bytes.extend_from_slice(&new_body);

    Ok(final_archive_bytes)
}

/// Lists the contents of a .small file or archive.
pub fn list_contents<R: Read + Seek>(reader: &mut R) -> std::io::Result<Vec<String>> {
    let mut file_names = Vec::new();

    let master_header = Headers::from_reader(reader)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Failed to parse master header: {}", e)))?;

    if flags::is_archive(master_header.flags) {
        loop {
            let embedded_header_start_pos = reader.seek(std::io::SeekFrom::Current(0))
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to seek reader: {}", e)))?;

            let embedded_header_result = Headers::from_reader(reader);
            let embedded_header = match embedded_header_result {
                Ok(h) => h,
                Err(e) => {
                    if let Some(io_err) = e.downcast_ref::<std::io::Error>() {
                        if io_err.kind() == std::io::ErrorKind::UnexpectedEof {
                            break; // End of archive body
                        }
                    }
                    return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Failed to parse embedded header: {}", e)));
                }
            };
            
            file_names.push(embedded_header.original_file_name);

            reader.seek(std::io::SeekFrom::Current(embedded_header.compressed_size as i64))?;
        }
    } else {
        // It's a single file, just list its name
        file_names.push(master_header.original_file_name);
    }

    Ok(file_names)
}

/// Extracts a single file from an archive.
pub fn extract_file<R: Read + Seek>(
    reader: &mut R,
    file_to_extract: &str,
    output_path: &Path,
    decrypt_password: Option<&str>,
) -> std::io::Result<()> {
    let master_header = Headers::from_reader(reader)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Failed to parse master header: {}", e)))?;

    if !flags::is_archive(master_header.flags) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Input file is not an archive. Cannot extract individual file.",
        ));
    }

    loop {
        let embedded_header_start_pos = reader.seek(std::io::SeekFrom::Current(0))
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to seek reader: {}", e)))?;

        let embedded_header_result = Headers::from_reader(reader);
        let embedded_header = match embedded_header_result {
            Ok(h) => h,
            Err(e) => {
                // If it's EOF, it means we've processed all embedded files
                if let Some(io_err) = e.downcast_ref::<std::io::Error>() {
                    if io_err.kind() == std::io::ErrorKind::UnexpectedEof {
                        break; // End of archive body
                    }
                }
                return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Failed to parse embedded header: {}", e)));
            }
        };

        // If the file matches, extract it
                    if embedded_header.original_file_name == file_to_extract {
                        if let Some(parent) = output_path.parent() {
                            fs::create_dir_all(parent)?;
                        }
                        let mut output_file = fs::File::create(output_path)?;
        
                        // Reconstruct the stream for decode: embedded_header + compressed_data
                        let mut full_file_bytes = embedded_header.clone().to_bytes();
                        let mut compressed_data_buf = vec![0u8; embedded_header.compressed_size as usize];
                        reader.read_exact(&mut compressed_data_buf)?;
                        full_file_bytes.extend_from_slice(&compressed_data_buf);
        
                        let mut full_file_reader = Cursor::new(full_file_bytes);
        
                        let _decode_info = decode(&mut full_file_reader, decrypt_password, &mut output_file)
                            .map_err(|e| std::io::Error::other(e.to_string()))?;
                        
                        return Ok(()); // File found and extracted
                    }
        // If not the target file, skip its compressed payload
        reader.seek(std::io::SeekFrom::Current(embedded_header.compressed_size as i64))?;
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        format!("File '{}' not found in archive.", file_to_extract),
    ))
}


