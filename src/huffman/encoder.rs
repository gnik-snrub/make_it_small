use crate::{flags::{flip_stored_raw, flip_encrypted}, headers::write_header, io::BitWriter, crypto::{self, encrypt}};
use super::{freq::compute_frequencies, table::generate_code_table, tree::{build_huffman_tree}};
use std::io::{Read, Write}; // Added for streaming traits

pub struct EncodeInfo {
    pub original_size: u64,
    pub compressed_size: u64,
    pub padding_bits: u8,
}

pub fn encode<R: Read, W: Write>(
    reader: &mut R,
    name: &str,
    encrypt_password: Option<&str>,
    writer: &mut W,
) -> Result<EncodeInfo, Box<dyn std::error::Error>> {
    // 1. Read entire input into memory to compute frequencies and build tree
    // (This is a temporary step until a streaming frequency computation is implemented)
    let mut src_data = Vec::new();
    reader.read_to_end(&mut src_data)?;

    let mut header = write_header(&src_data, name);
    let mut flags = header.flags;

    let freq = compute_frequencies(&src_data);

    let (codes, lengths) = match build_huffman_tree(&freq) {
        Some(tree) => {
            let table = generate_code_table(&tree);
            header.tree = tree;
            (table.map(|c| c.0), table.map(|c| c.1))
        }
        None => {
            ([0u32; 256], [0u8; 256])
        }
    };

    // Buffer to hold compressed data before potential encryption
    let mut compressed_buffer_for_encryption = Vec::new();
    {
        // Use BitWriter to write compressed data to an in-memory buffer
        let mut bit_writer = BitWriter::new(&mut compressed_buffer_for_encryption);

        for b in &src_data {
            let code = codes[*b as usize];
            let len = lengths[*b as usize];
            let bits = (code >> (32 - len)) & ((1u32 << len) - 1);
            bit_writer.write_bits(bits, len)?;
        }

        bit_writer.finalize()?;
        header.padding_bits = bit_writer.padding_bits as u8;
    }


    let mut final_payload_bytes = compressed_buffer_for_encryption;


    // Apply encryption if password is provided
    if let Some(password) = encrypt_password {
        let salt = crypto::generate_random_bytes::<{crypto::SALT_LEN}>();
        let iv = crypto::generate_random_bytes::<{crypto::IV_LEN}>();

        let key = crypto::derive_key(password.as_bytes(), &salt);

        let (encrypted_data, tag) = encrypt(&final_payload_bytes, &key, &iv)?;

        final_payload_bytes = encrypted_data;
        header.salt = salt;
        header.iv = iv;
        header.tag = tag;
        flip_encrypted(&mut flags);
    }

    // Determine final header values based on compression results and encryption
    let compressed_size = final_payload_bytes.len() as u64;

    let mut final_header_to_write = write_header(&src_data, name);
    final_header_to_write.tree = header.tree;
    final_header_to_write.flags = flags;
    final_header_to_write.padding_bits = header.padding_bits;
    final_header_to_write.salt = header.salt;
    final_header_to_write.iv = header.iv;
    final_header_to_write.tag = header.tag;


    if compressed_size >= header.original_size && encrypt_password.is_none() {
        // Only store raw if not encrypted and compression makes it larger
        // If encrypted, we always use the encrypted payload, even if larger
        flip_stored_raw(&mut final_header_to_write.flags);

        final_header_to_write.compressed_size = header.original_size;
        final_header_to_write.padding_bits = 0;

        // Write the final header
        writer.write_all(&final_header_to_write.clone().to_bytes())?;
        
        // Write original raw data
        writer.write_all(&src_data)?;
    } else {
        final_header_to_write.compressed_size = compressed_size;

        // Write the final header
        writer.write_all(&final_header_to_write.clone().to_bytes())?;
        
        // Write the compressed (or encrypted compressed) payload
        writer.write_all(&final_payload_bytes)?;
    }
    
    Ok(EncodeInfo {
        original_size: header.original_size, // Original uncompressed size
        compressed_size: final_header_to_write.compressed_size, // Final (potentially encrypted) compressed size
        padding_bits: final_header_to_write.padding_bits, // Final padding
    })
}
