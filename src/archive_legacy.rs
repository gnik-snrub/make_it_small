use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use crate::flags;
use crate::headers::Headers;
use crate::huffman::{decoder::decode, encoder::encode};

/// Creates an archive from a directory.
pub fn create_archive<W: Write + Seek>(
    dir_path: &Path,
    encrypt_password: Option<&str>,
    writer: &mut W,
    chunk_size: usize,
) -> std::io::Result<()> {
    let all_files = get_all_files(dir_path)?;

    let mut total_original_size: u64 = 0;

    // Create a temporary file to store the archive body
    let mut temp_body_file = tempfile::tempfile()?;

    for file_path in all_files {
        let relative_path = file_path.strip_prefix(dir_path).map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Failed to get relative path for {:?}", file_path),
            )
        })?;

        let mut current_file = File::open(&file_path)?; // Open file directly for streaming

        let encode_info = encode(
            &mut current_file,
            &relative_path.to_string_lossy(),
            encrypt_password,
            &mut temp_body_file, // Encode directly to the temporary file
            chunk_size,          // Pass chunk_size
        )
        .map_err(|e| std::io::Error::other(e.to_string()))?;

        total_original_size += encode_info.original_size;
    }

    // Get the size of the temporary file, which is the total compressed size of the body
    temp_body_file.flush()?;
    let master_compressed_size = temp_body_file.stream_position()?;
    temp_body_file.seek(SeekFrom::Start(0))?; // Rewind the temporary file

    // Now construct the master header with final, correct sizes
    let mut master_header = Headers::new();
    flags::flip_is_archive(&mut master_header.flags);
    if encrypt_password.is_some() {
        flags::flip_encrypted(&mut master_header.flags);
    }

    master_header.original_size = total_original_size;
    master_header.compressed_size = master_compressed_size;
    master_header.original_file_name = dir_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    // Write the master header to the main writer
    writer.write_all(&master_header.to_bytes())?;
    // Copy the contents of the temporary file to the main writer
    std::io::copy(&mut temp_body_file, writer)?;

    Ok(())
}

/// Extracts an archive to a directory.
pub fn extract_archive<R: Read + Seek>(
    master_header: Headers,
    reader: &mut R,
    output_path: &Path,
    decrypt_password: Option<&str>,
    chunk_size: usize,
) -> std::io::Result<()> {
    // Return type changed to Result<()> as master_header is passed in

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
        let embedded_header_result = Headers::from_reader(reader);
        let embedded_header = match embedded_header_result {
            Ok(h) => h,
            Err(e) => {
                if let Some(io_err) = e.downcast_ref::<std::io::Error>() {
                    // Break if we hit EOF precisely at the end of the archive body.
                    if io_err.kind() == std::io::ErrorKind::UnexpectedEof {
                        if reader.stream_position()? == archive_body_end_position {
                            break; // Expected EOF at the end of the archive body
                        } else {
                            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Truncated archive: Unexpected end of file before archive body end.".to_string()));
                        }
                    }
                }
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Failed to parse embedded header: {}", e),
                ));
            }
        };

        // Get the current position of the main reader, which is now at the start of the payload
        let payload_start_position = reader.stream_position()?;

        // Create output path
        let output_file_path = output_path.join(&embedded_header.original_file_name);
        if let Some(parent) = output_file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut output_file = fs::File::create(&output_file_path)?;

        let embedded_header_compressed_size = embedded_header.compressed_size;

        // Create a limited reader for the decode function
        let mut limited_reader = reader.take(embedded_header_compressed_size);

        let _decode_info = decode(
            embedded_header,
            &mut limited_reader, // Pass the limited reader to decode
            decrypt_password,
            &mut output_file,
            chunk_size, // Pass chunk_size
        )
        .map_err(|e| std::io::Error::other(e.to_string()))?;

        // After decode, explicitly advance the main reader past the payload
        // This is necessary because reader.take() only provides a limited view,
        // it doesn't advance the underlying reader's position.
        reader.seek(SeekFrom::Start(
            payload_start_position + embedded_header_compressed_size,
        ))?;
    }

    Ok(())
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
pub fn add_to_archive<R1: Read + Seek, R2: Read + Seek, W: Write + Seek>(
    existing_reader: &mut R1,
    new_reader: &mut R2,
    writer: &mut W,
    _chunk_size: usize,
) -> std::io::Result<()> {
    // 1. Read master header of existing archive
    let master_header_existing = Headers::from_reader(existing_reader).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Failed to parse header of existing file: {}", e),
        )
    })?;
    let _master_header_existing_len = existing_reader.stream_position()?;

    // 2. Read header of new content
    let new_content_header = Headers::from_reader(new_reader).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Failed to parse header of new content: {}", e),
        )
    })?;
    let _new_content_header_len = new_reader.stream_position()?;

    // 3. Create a temporary file to build the new archive body
    let mut temp_new_body_file = tempfile::tempfile()?;

    // 4. Copy existing archive body (all embedded files) to temporary file
    // We only copy up to master_header_existing.compressed_size
    // This is the entire body, which is all embedded headers + payloads.
    std::io::copy(
        &mut existing_reader.take(master_header_existing.compressed_size),
        &mut temp_new_body_file,
    )?;

    // 5. Append new content (header + payload) to temporary file
    // The new_reader is already positioned after new_content_header
    // We copy the remaining data, which is new_content_header.compressed_size
    new_reader.seek(SeekFrom::Start(0))?; // Rewind new_reader to copy full new content (header + payload)
    std::io::copy(new_reader, &mut temp_new_body_file)?; // Copy header + payload of new content

    // 6. Get the total compressed size of the new archive body
    temp_new_body_file.flush()?;
    let new_body_compressed_size = temp_new_body_file.stream_position()?;
    temp_new_body_file.seek(SeekFrom::Start(0))?; // Rewind temporary file

    // 7. Construct new master header
    let mut new_master_header = Headers::new();
    flags::flip_is_archive(&mut new_master_header.flags);

    if flags::is_encrypted(master_header_existing.flags)
        || flags::is_encrypted(new_content_header.flags)
    {
        flags::flip_encrypted(&mut new_master_header.flags);
    }

    new_master_header.original_size =
        master_header_existing.original_size + new_content_header.original_size;
    new_master_header.compressed_size = new_body_compressed_size;
    new_master_header.original_file_name = master_header_existing.original_file_name; // Keep the original archive name

    // 8. Write new master header and new body to the final writer
    writer.write_all(&new_master_header.to_bytes())?;
    std::io::copy(&mut temp_new_body_file, writer)?;

    Ok(())
}

/// Lists the contents of a .small file or archive.
pub fn list_contents<R: Read + Seek>(reader: &mut R) -> std::io::Result<Vec<String>> {
    let mut file_names = Vec::new();

    let master_header = Headers::from_reader(reader).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Failed to parse master header: {}", e),
        )
    })?;

    if flags::is_archive(master_header.flags) {
        loop {
            let _embedded_header_start_pos = reader
                .stream_position()
                .map_err(|e| std::io::Error::other(format!("Failed to seek reader: {}", e)))?;

            let embedded_header_result = Headers::from_reader(reader);
            let embedded_header = match embedded_header_result {
                Ok(h) => h,
                Err(e) => {
                    if let Some(io_err) = e.downcast_ref::<std::io::Error>()
                        && io_err.kind() == std::io::ErrorKind::UnexpectedEof
                    {
                        break; // End of archive body
                    }
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Failed to parse embedded header: {}", e),
                    ));
                }
            };

            file_names.push(embedded_header.original_file_name);

            reader.seek(std::io::SeekFrom::Current(
                embedded_header.compressed_size as i64,
            ))?;
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
    chunk_size: usize,
) -> std::io::Result<()> {
    let master_header = Headers::from_reader(reader).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Failed to parse master header: {}", e),
        )
    })?;

    if !flags::is_archive(master_header.flags) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Input file is not an archive. Cannot extract individual file.",
        ));
    }

    loop {
        let _embedded_header_start_pos = reader
            .stream_position()
            .map_err(|e| std::io::Error::other(format!("Failed to seek reader: {}", e)))?;

        let embedded_header_result = Headers::from_reader(reader);
        let embedded_header = match embedded_header_result {
            Ok(h) => h,
            Err(e) => {
                // If it's EOF, it means we've processed all embedded files
                if let Some(io_err) = e.downcast_ref::<std::io::Error>()
                    && io_err.kind() == std::io::ErrorKind::UnexpectedEof
                {
                    break; // End of archive body
                }
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Failed to parse embedded header: {}", e),
                ));
            }
        };

        // If the file matches, extract it
        if embedded_header.original_file_name == file_to_extract {
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut output_file = fs::File::create(output_path)?;

            let _decode_info = decode(
                embedded_header, // Pass the already parsed header
                reader,          // Pass the main reader, which is correctly positioned
                decrypt_password,
                &mut output_file,
                chunk_size, // Pass chunk_size
            )
            .map_err(|e| std::io::Error::other(e.to_string()))?;

            return Ok(()); // File found and extracted
        }
        // If not the target file, skip its compressed payload
        reader.seek(std::io::SeekFrom::Current(
            embedded_header.compressed_size as i64,
        ))?;
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        format!("File '{}' not found in archive.", file_to_extract),
    ))
}
