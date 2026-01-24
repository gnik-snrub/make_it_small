use crate::constants::{MAGIC_BYTES, VERSION};
use crate::crypto;
use crate::crypto::decrypt_stream;
use crate::flags::{is_encrypted, is_stored_raw};
use crate::headers::Headers;
use crate::io::BitReader;
use std::io::{Read, Seek, Write}; // Added Seek for tempfile ops
use tempfile::tempfile; // Added for temporary file

/// Information about a decoded file
///
/// # Fields
///
/// * `original_file_name` - Filename extracted from header
/// * `checksum` - Adler-32 checksum of original data for integrity verification
/// * `original_size` - Size of original uncompressed file in bytes
#[derive(Debug)]
pub struct DecodeInfo {
    pub original_file_name: String,
    pub checksum: u32,
    pub original_size: u64,
}

/// Decodes a file using Huffman coding with streaming architecture and optional decryption
///
/// This function performs streaming decompression with optional AES-256-GCM decryption:
/// 1. Validates file format (magic bytes, version, flags)
/// 2. Decrypts data if encrypted flag is set
/// 3. Decompresses Huffman-encoded data or copies raw data
///
/// # Arguments
///
/// * `header` - Parsed header containing metadata and Huffman tree
/// * `reader` - Input data stream positioned after header (must implement Read + Seek)
/// * `decrypt_password` - Optional password for AES-256-GCM decryption (None if not encrypted)
/// * `writer` - Output destination for decompressed data
/// * `chunk_size` - Memory chunk size for decryption processing
///
/// # Returns
///
/// Returns `DecodeInfo` containing original filename, checksum, and size
///
/// # Memory Usage
///
/// Maximum memory usage = `chunk_size` + ~50KB overhead. Uses temporary files
/// for decryption when encryption is present.
///
/// # Errors
///
/// Returns errors for:
/// - Invalid file format (wrong magic bytes/version)
/// - Missing password for encrypted files
/// - Decryption failures (wrong password, corrupted data)
/// - Corruption detection during Huffman decoding
/// - I/O failures during read/write operations
pub fn decode<R: Read + Seek, W: Write>(
    header: Headers,
    reader: &mut R,
    decrypt_password: Option<&str>,
    writer: &mut W,
    chunk_size: usize,
) -> Result<DecodeInfo, Box<dyn std::error::Error>> {
    if header.magic_bytes != MAGIC_BYTES {
        return Err("Error: Not a valid .small file".into());
    }

    if header.version != VERSION {
        return Err("Error: Incorrect version".into());
    }

    let mut payload_reader: Box<dyn Read> = Box::new(reader.take(header.compressed_size)); // Added Seek to Box<dyn Read>

    // Handle decryption if encrypted flag is set
    if is_encrypted(header.flags) {
        let password = decrypt_password.ok_or("Error: File is encrypted, password required.")?;
        let key = crypto::derive_key(password.as_bytes(), &header.salt);

        // Decrypt stream to a temporary file
        let mut decrypted_temp_file = tempfile()?; // Use tempfile directly
        let _decrypted_bytes_written = decrypt_stream(
            &mut payload_reader,      // Input is the encrypted payload stream
            &mut decrypted_temp_file, // Output decrypted data to a temporary file
            &key,
            &header.iv,
            &[],        // No AAD for now
            chunk_size, // Pass chunk size
        )?;
        decrypted_temp_file.seek(std::io::SeekFrom::Start(0))?; // Rewind temp file

        // Replace payload_reader with the decrypted stream from the temporary file
        payload_reader = Box::new(decrypted_temp_file.take(header.payload_actual_size));
    }

    if is_stored_raw(header.flags) {
        // Read original data directly from the (potentially decrypted) payload reader
        // Limit the read to original_size to prevent reading data of subsequent files in an archive
        std::io::copy(&mut payload_reader.take(header.payload_actual_size), writer)?;
    } else {
        // Decompress Huffman data
        // Pass the payload_reader to BitReader
        let mut bit_reader = BitReader::new(header.padding_bits as usize, payload_reader);
        let mut current = &header.tree;
        let mut decoded_bytes_count = 0;

        // Calculate total number of actual data bits
        let total_data_bits = (header.payload_actual_size * 8) - header.padding_bits as u64;
        let mut bits_read_count: u64 = 0; // New counter for bits read

        while decoded_bytes_count < header.original_size {
            if let Some(byte) = current.symbol {
                writer.write_all(&[byte])?;
                decoded_bytes_count += 1;
                current = &header.tree;
                continue;
            }

            // Check if we have read all actual data bits before attempting to read more
            if bits_read_count >= total_data_bits {
                // If we are here, it means we needed more bits to decode, but ran out of actual data bits.
                // This indicates a corrupted stream or incorrect header.
                return Err(
                    "Corrupted: Unexpected end of compressed data (read past padding)".into(),
                );
            }

            match bit_reader.read_bit()? {
                // read_bit now returns Result<Option<u8>>
                Some(0) => {
                    current = current
                        .left
                        .as_ref()
                        .ok_or("Corrupted: Missing left node")?;
                    bits_read_count += 1; // Increment bit counter
                }
                Some(1) => {
                    current = current
                        .right
                        .as_ref()
                        .ok_or("Corrupted: Missing right node")?;
                    bits_read_count += 1; // Increment bit counter
                }
                None => {
                    return Err(
                        "Corrupted: Unexpected end of file (BitReader returned None)".into(),
                    );
                }
                _ => unreachable!(),
            }
        }
    }

    Ok(DecodeInfo {
        original_file_name: header.original_file_name,
        checksum: header.checksum,
        original_size: header.original_size,
    })
}
