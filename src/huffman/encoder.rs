use crate::{flags::{flip_stored_raw, flip_encrypted}, headers::write_header, io::BitWriter, crypto};
use super::{freq::compute_frequencies, table::generate_code_table, tree::{build_huffman_tree}};
use std::io::{Read, Write, Seek, SeekFrom}; // Added SeekFrom for seek operations

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
    let mut actual_payload_size: u64 = 0;
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
    if encrypt_password.is_none() {
        // If Huffman encoding increased size or not effective, use raw storage
        if huffman_encoded && actual_payload_size >= original_size {
            flip_stored_raw(&mut header.flags);
            header.padding_bits = 0;
            header.compressed_size = original_size; // Raw size
            
            // Write header
            writer.write_all(&header.clone().to_bytes())?;
            // Stream original data directly
            reader.seek(SeekFrom::Start(original_position))?;
            std::io::copy(reader, writer)?; // Copy original raw data
        } else {
            // Huffman compressed data is better or no tree built (default to Huffman/raw if smaller)
            header.padding_bits = actual_padding_bits;
            header.compressed_size = actual_payload_size;

            // Write header
            writer.write_all(&header.clone().to_bytes())?;
            // Stream compressed data from temp file
            std::io::copy(&mut temp_payload_file, writer)?;
        }
    } else { // Encryption is present, use streaming encryption
        flip_encrypted(&mut header.flags);
        header.salt = crypto::generate_random_bytes::<{crypto::SALT_LEN}>();
        header.iv = crypto::generate_random_bytes::<{crypto::FIXED_IV_PART_LEN}>(); // This will be our initial_nonce

        let key = crypto::derive_key(encrypt_password.unwrap().as_bytes(), &header.salt);

        // Prepare the reader for crypto::encrypt_stream
        // If Huffman compressed data is better, use temp_payload_file as input.
        // Else, use original reader for raw data.
        let mut reader_for_encryption: Box<dyn Read> = if huffman_encoded && actual_payload_size < original_size {
            header.padding_bits = actual_padding_bits;
            Box::new(temp_payload_file) // Use the temporary file with Huffman compressed data
        } else {
            flip_stored_raw(&mut header.flags);
            header.padding_bits = 0;
            reader.seek(SeekFrom::Start(original_position))?; // Rewind original reader
            Box::new(reader.take(original_size)) // Use original reader for raw data (limited to original_size)
        };
        
        // Write header first, before encrypted stream content
        writer.write_all(&header.clone().to_bytes())?;

        let encrypted_size = crypto::encrypt_stream(
            &mut reader_for_encryption,
            writer, // Write directly to the final writer
            &key,
            header.iv.as_ref(), // Use header.iv as initial_nonce_bytes
            &[], // No AAD for now, but could be added if needed (e.g., filename)
            chunk_size, // Pass the chunk size
        )?;
        
        header.compressed_size = encrypted_size as u64; // Update compressed_size based on encrypted output
        header.tag = [0u8; crypto::TAG_LEN]; // Zero out header.tag as integrity is per-chunk
    }

    Ok(EncodeInfo {
        original_size,
        compressed_size: header.compressed_size,
        padding_bits: header.padding_bits,
    })
}
