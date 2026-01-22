use crate::{flags::{flip_stored_raw, flip_encrypted}, headers::write_header, io::BitWriter, crypto::{self, encrypt}};
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
) -> Result<EncodeInfo, Box<dyn std::error::Error>> {
    // --- Pass 1: Compute Frequencies, Checksum, and Original Size ---
    let original_position = reader.stream_position()?;
    let (freq, checksum, original_size) = compute_frequencies(reader)?;
    reader.seek(SeekFrom::Start(original_position))?; // Rewind reader for second pass

    let mut header = write_header(original_size, checksum, name);

    // --- Pass 2: Generate codes and encode to a temporary payload buffer ---
    let mut payload_buffer = Vec::new();
    let mut huffman_encoded = false;

    if let Some(tree) = build_huffman_tree(&freq) {
        header.tree = tree;
        let codes = generate_code_table(&header.tree);
        
        let mut bit_writer = BitWriter::new(&mut payload_buffer);
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
        header.padding_bits = bit_writer.padding_bits as u8;
        huffman_encoded = true;
    }

    // --- Decide on final payload and flags ---
    let mut final_payload = payload_buffer; // Initially, the Huffman compressed data

    // Decide if we should store raw instead
    if huffman_encoded && final_payload.len() as u64 >= original_size && encrypt_password.is_none() {
        flip_stored_raw(&mut header.flags);
        header.padding_bits = 0;
        // Reread original data for the payload
        final_payload.clear();
        reader.seek(SeekFrom::Start(original_position))?;
        reader.read_to_end(&mut final_payload)?;
    }

    // Encrypt if password is provided
    if let Some(password) = encrypt_password {
        flip_encrypted(&mut header.flags);
        header.salt = crypto::generate_random_bytes::<{crypto::SALT_LEN}>();
        header.iv = crypto::generate_random_bytes::<{crypto::IV_LEN}>();
        let key = crypto::derive_key(password.as_bytes(), &header.salt);
        let (encrypted_data, tag) = encrypt(&final_payload, &key, &header.iv)?;
        header.tag = tag;
        final_payload = encrypted_data;
    }

    // Finalize header and write everything
    header.compressed_size = final_payload.len() as u64;

    writer.write_all(&header.clone().to_bytes())?;
    writer.write_all(&final_payload)?;

    Ok(EncodeInfo {
        original_size,
        compressed_size: header.compressed_size,
        padding_bits: header.padding_bits,
    })
}
