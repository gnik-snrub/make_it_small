use super::{freq::compute_frequencies, table::generate_code_table, tree::build_huffman_tree};
use crate::{
    crypto,
    flags::{flip_encrypted, flip_stored_raw},
    headers::write_header,
    io::BitWriter,
};
use std::io::{Read, Seek, SeekFrom, Write}; // Added SeekFrom for seek operations

pub struct EncodeInfo {
    pub original_size: u64,
    pub compressed_size: u64,
    pub padding_bits: u8,
}

pub fn encode<R: Read + Seek, W: Write>(
    reader: &mut R,
    name: &str,
    encrypt_password: Option<&str>,
    writer: &mut W,
    chunk_size: usize, // New parameter
) -> Result<EncodeInfo, Box<dyn std::error::Error>> {
    // --- Pass 1: Compute Frequencies, Checksum, and Original Size ---
    let original_position = reader.stream_position()?;
    let (freq, checksum, original_size) = compute_frequencies(reader)?;
    reader.seek(SeekFrom::Start(original_position))?; // Rewind reader for second pass

    let mut header = write_header(original_size, checksum, name);

    // Create a temporary file to store the payload during the first pass
    let mut temp_payload_file = tempfile::tempfile()?;
    let actual_payload_size: u64;
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
    actual_payload_size = temp_payload_file.stream_position()?;
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
