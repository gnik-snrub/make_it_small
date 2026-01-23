use aes_gcm::{
    aead::{KeyInit, OsRng, AeadInPlace, Nonce, Tag}, // Added Nonce and GenericArray
    Aes256Gcm, Key 
};
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;
use aes_gcm::aead::rand_core::RngCore; // Added RngCore
use std::io::{Read, Write};

// Key, Salt, IV, Tag lengths for AES-256 GCM
pub const KEY_LEN: usize = 32; // 256 bits
pub const SALT_LEN: usize = 16;
pub const IV_LEN: usize = 12; // 96 bits for GCM
pub const FIXED_IV_PART_LEN: usize = IV_LEN - 8; // The part of the IV that is fixed per file (4 bytes)
pub const TAG_LEN: usize = 16; // Authentication Tag (for each chunk in streaming)
pub const DEFAULT_CHUNK_SIZE: usize = 16 * 1024 * 1024; // 16 MB

pub fn derive_key(password: &[u8], salt: &[u8]) -> Key<Aes256Gcm> {
    let mut key_bytes = [0u8; KEY_LEN];
    pbkdf2_hmac::<Sha256>(password, salt, 100_000, &mut key_bytes); // 100k iterations
    key_bytes.into() // Use into()
}

pub fn encrypt_stream<R: Read, W: Write>(
    reader: &mut R,
    writer: &mut W,
    key: &Key<Aes256Gcm>,
    initial_nonce_bytes: &[u8], // Base IV for the file
    aad: &[u8], // Additional authenticated data
    chunk_size: usize,
) -> Result<usize, Box<dyn std::error::Error>> {
    let cipher = Aes256Gcm::new(key);
    let mut total_bytes_written = 0;
    
    // Buffer for plaintext. Encrypted data is written back into this buffer.
    let mut plaintext_buffer = vec![0u8; chunk_size];

    if initial_nonce_bytes.len() > FIXED_IV_PART_LEN {
        return Err(format!("Initial nonce bytes too long. Must be <= FIXED_IV_PART_LEN ({} bytes).", FIXED_IV_PART_LEN).into());
    }

    let mut current_nonce_value = 0u64; // Use u64 for counter

    loop {
        // Read plaintext into the buffer
        let bytes_read = reader.read(&mut plaintext_buffer[..])?;
        if bytes_read == 0 {
            break; // EOF
        }

        // Construct the nonce for this chunk
        let mut full_nonce_bytes = [0u8; IV_LEN];
        let initial_part_len = initial_nonce_bytes.len().min(IV_LEN);
        full_nonce_bytes[..initial_part_len].copy_from_slice(&initial_nonce_bytes[..initial_part_len]);

        let counter_bytes_start = IV_LEN - 8; // Assuming 8 bytes for counter
        if counter_bytes_start < initial_part_len {
            return Err("Initial nonce bytes overlap with counter part. Adjust initial_nonce_bytes length.".into());
        }
        full_nonce_bytes[counter_bytes_start..].copy_from_slice(&current_nonce_value.to_be_bytes());

        let nonce = Nonce::<Aes256Gcm>::from_slice(&full_nonce_bytes);

        // Encrypt in place. `plaintext_buffer` will contain ciphertext.
        let tag = cipher.encrypt_in_place_detached(
            nonce,
            aad, // Associated data for this chunk
            &mut plaintext_buffer[..bytes_read], // Buffer contains plaintext and will be overwritten with ciphertext
        )
        .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())) as Box<dyn std::error::Error>)?;
        
        writer.write_all(&plaintext_buffer[..bytes_read])?; // Write ciphertext
        writer.write_all(tag.as_ref())?; // Write tag
        total_bytes_written += bytes_read + TAG_LEN;

        current_nonce_value += 1;
    }

    Ok(total_bytes_written)
}

pub fn decrypt_stream<R: Read, W: Write>(
    reader: &mut R,
    writer: &mut W,
    key: &Key<Aes256Gcm>,
    initial_nonce_bytes: &[u8],
    aad: &[u8], // Additional authenticated data
    chunk_size: usize,
) -> Result<usize, Box<dyn std::error::Error>> {
    let cipher = Aes256Gcm::new(key);
    let mut total_bytes_written = 0;

    // Buffer for ciphertext + tag, and then plaintext after decryption.
    let mut buffer = vec![0u8; chunk_size]; // Buffer for plaintext/ciphertext, not including tag.
    let mut tag_buffer = [0u8; TAG_LEN]; // Separate buffer for the tag.

    if initial_nonce_bytes.len() > FIXED_IV_PART_LEN {
        return Err(format!("Initial nonce bytes too long. Must be <= FIXED_IV_PART_LEN ({} bytes).", FIXED_IV_PART_LEN).into());
    }

    let mut current_nonce_value = 0u64; // Counter for nonce generation

    loop {
        // Read ciphertext into the buffer.
        let bytes_read_ciphertext = reader.read(&mut buffer[..])?; // Read into plaintext/ciphertext buffer
        if bytes_read_ciphertext == 0 {
            break; // EOF
        }
        
        // Read the tag for this chunk
        let bytes_read_tag = reader.read(&mut tag_buffer)?;
        if bytes_read_tag != TAG_LEN {
            return Err("Truncated data: unable to read full tag for chunk".into());
        }

        // Reconstruct the nonce for this chunk, similar to encryption
        let mut full_nonce_bytes = [0u8; IV_LEN];
        let initial_part_len = initial_nonce_bytes.len().min(IV_LEN);
        full_nonce_bytes[..initial_part_len].copy_from_slice(&initial_nonce_bytes[..initial_part_len]);

        let counter_bytes_start = IV_LEN - 8;
        if counter_bytes_start < initial_part_len {
            return Err("Initial nonce bytes overlap with counter part. Adjust initial_nonce_bytes length.".into());
        }
        full_nonce_bytes[counter_bytes_start..].copy_from_slice(&current_nonce_value.to_be_bytes());

        let nonce = Nonce::<Aes256Gcm>::from_slice(&full_nonce_bytes);

        // Decrypt in place. `buffer` contains ciphertext and will be overwritten with plaintext.
        cipher.decrypt_in_place_detached(
            nonce,
            aad, // Associated data for this chunk
            &mut buffer[..bytes_read_ciphertext], // Buffer contains ciphertext
            &Tag::<Aes256Gcm>::from_slice(&tag_buffer), // Tag
        )
        .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Decryption error: {}", e.to_string()))) as Box<dyn std::error::Error>)?;

        writer.write_all(&buffer[..bytes_read_ciphertext])?; // Write only the plaintext part
        total_bytes_written += bytes_read_ciphertext;

        current_nonce_value += 1;
    }

    Ok(total_bytes_written)
}

pub fn generate_random_bytes<const N: usize>() -> [u8; N] {
    let mut bytes = [0u8; N];
    OsRng.fill_bytes(&mut bytes);
    bytes
}
