use super::{freq::compute_frequencies, table::generate_code_table, tree::build_huffman_tree};
use crate::{
    crypto,
    flags::{flip_encrypted, flip_stored_raw},
    headers::write_header,
    io::BitWriter,
};
use std::io::{Read, Seek, SeekFrom, Write}; // Added SeekFrom for seek operations

/// Information about the encoding operation result
///
/// # Fields
///
/// * `original_size` - Size of the original uncompressed file in bytes
/// * `compressed_size` - Size of the final output (includes encryption overhead if applicable)
/// * `padding_bits` - Number of padding bits added to complete the final byte (0-7)
#[derive(Debug)]
pub struct EncodeInfo {
    pub original_size: u64,
    pub compressed_size: u64,
    pub padding_bits: u8,
}

/// Encodes a file using Huffman coding with streaming architecture and optional encryption
///
/// This function performs a two-pass encoding process:
/// 1. First pass: Compute symbol frequencies and checksum in configurable chunks
/// 2. Second pass: Encode data using bit-level packing with optional AES-256-GCM encryption
///
/// # Arguments
///
/// * `reader` - Input data source implementing Read + Seek (must support rewinding)
/// * `name` - Original filename for storage in the header
/// * `encrypt_password` - Optional password for AES-256-GCM encryption (None = no encryption)
/// * `writer` - Output destination implementing Write
/// * `chunk_size` - Memory chunk size for processing (affects memory usage, not compression ratio)
///
/// # Returns
///
/// Returns `EncodeInfo` containing original size, compressed size, and padding bits
///
/// # Memory Usage
///
/// Maximum memory usage = `chunk_size` + ~50KB overhead. Uses temporary files for
/// intermediate processing to avoid loading entire files into memory.
///
/// # Performance Tips
///
/// For best performance with large files, use larger chunk sizes:
/// - 1MB chunks: More memory usage, fewer I/O operations
/// - 16MB chunks: Balanced performance for most systems
/// - 64MB chunks: Better throughput on systems with ample RAM
///
/// # Examples
///
/// ```rust
/// use std::io::Cursor;
/// use mismall::compress::compress_stream;
/// use mismall::DEFAULT_CHUNK_SIZE;
///
/// let input = b"Hello, world!";
/// let mut reader = Cursor::new(input);
/// let mut output = Cursor::new(Vec::new());
///
/// // For large files, consider using 64MB chunk size:
/// let large_chunk_size = 64 * 1024 * 1024;
/// let info = compress_stream(&mut reader, "test.txt", None, &mut output, large_chunk_size)?;
/// println!("Compressed {} bytes to {} bytes", info.original_size, info.compressed_size);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
///
/// Returns errors for I/O failures, invalid input, or encryption failures
pub fn encode<R: Read + Seek, W: Write>(
    reader: &mut R,
    name: &str,
    encrypt_password: Option<&str>,
    writer: &mut W,
    chunk_size: usize,
) -> Result<EncodeInfo, Box<dyn std::error::Error>> {
    // --- Pass 1: Compute Frequencies, Checksum, and Original Size ---
    let original_position = reader.stream_position()?;
    let (freq, checksum, original_size) = compute_frequencies(reader)?;
    reader.seek(SeekFrom::Start(original_position))?; // Rewind reader for second pass

    let mut header = write_header(original_size, checksum, name);

    // Create a temporary file to store the payload during the first pass
    let mut temp_payload_file = tempfile::tempfile()?;

    let mut actual_padding_bits: u8 = 0;
    let mut huffman_encoded = false;

    if let Some(tree) = build_huffman_tree(&freq) {
        header.tree = tree;
        let codes = generate_code_table(&header.tree);

        let mut bit_writer = BitWriter::new(&mut temp_payload_file);
        let mut buffer = [0; 4096];
        loop {
            let bytes_read = reader.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            for &byte in &buffer[..bytes_read] {
                let (code, len) = codes[byte as usize];
                if len > 0 {
                    let bits = (code >> (32 - len)) & ((1u32 << len) - 1);
                    bit_writer.write_bits(bits, len)?;
                }
            }
        }
        bit_writer.finalize()?;
        actual_padding_bits = bit_writer.padding_bits as u8;
        huffman_encoded = true;
    }
    temp_payload_file.flush()?;
    let actual_payload_size = temp_payload_file.stream_position()?;
    temp_payload_file.seek(SeekFrom::Start(0))?; // Rewind

    // --- Decide on final payload source and flags ---
    // Determine the size of the payload before encryption (Huffman compressed or raw)
    let unencrypted_payload_size: u64;
    let reader_for_payload: Box<dyn Read> =
        if huffman_encoded && actual_payload_size < original_size {
            header.padding_bits = actual_padding_bits;
            unencrypted_payload_size = actual_payload_size;
            Box::new(temp_payload_file) // Use the temporary file with Huffman compressed data
        } else {
            flip_stored_raw(&mut header.flags);
            header.padding_bits = 0;
            unencrypted_payload_size = original_size;
            reader.seek(SeekFrom::Start(original_position))?; // Rewind original reader
            Box::new(reader.take(unencrypted_payload_size)) // Use original reader for raw data, limited to its size
        };
    header.payload_actual_size = unencrypted_payload_size; // Set the new header field

    if encrypt_password.is_none() {
        // If not encrypted, compressed_size is the same as unencrypted_payload_size
        header.compressed_size = unencrypted_payload_size;

        // Write header
        writer.write_all(&header.clone().to_bytes())?;

        // Stream data to final writer
        std::io::copy(
            &mut reader_for_payload.take(unencrypted_payload_size),
            writer,
        )?;
    } else {
        // Encryption is present, use streaming encryption
        flip_encrypted(&mut header.flags);
        header.salt = crypto::generate_random_bytes::<{ crypto::SALT_LEN }>();
        header.iv = crypto::generate_random_bytes::<{ crypto::IV_LEN }>();

        #[allow(clippy::unnecessary_unwrap)]
        // Safe - we're in the else branch after is_none() check
        let key = crypto::derive_key(encrypt_password.unwrap().as_bytes(), &header.salt);

        // Create a temporary file for the encrypted payload
        let mut temp_encrypted_file = tempfile::tempfile()?;

        let encrypted_size = crypto::encrypt_stream(
            &mut reader_for_payload.take(unencrypted_payload_size), // Stream from the unencrypted payload source
            &mut temp_encrypted_file,                               // Encrypt to the temporary file
            &key,
            &header.iv,
            &[],
            chunk_size,
        )?;

        // Update header.compressed_size with the actual encrypted size
        header.compressed_size = encrypted_size as u64;
        header.tag = [0u8; crypto::TAG_LEN]; // Zero out header.tag as integrity is per-chunk

        // Write header to the final writer
        writer.write_all(&header.clone().to_bytes())?;

        // Rewind temporary encrypted file and copy its content to the final writer
        temp_encrypted_file.seek(SeekFrom::Start(0))?;
        std::io::copy(&mut temp_encrypted_file, writer)?;
    }

    Ok(EncodeInfo {
        original_size,
        compressed_size: header.compressed_size,
        padding_bits: header.padding_bits,
    })
}
