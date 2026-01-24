use aes_gcm::aead::rand_core::RngCore; // Added RngCore
use aes_gcm::{
    aead::{AeadInPlace, KeyInit, Nonce, OsRng, Tag}, // Added Nonce and GenericArray
    Aes256Gcm,
    Key,
};
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;
use std::io::{Read, Write};

// Key, Salt, IV, Tag lengths for AES-256 GCM
pub const KEY_LEN: usize = 32; // 256 bits
pub const SALT_LEN: usize = 16;
pub const IV_LEN: usize = 12; // 96 bits for GCM
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
    initial_nonce: &[u8; IV_LEN], // Full 12-byte initial nonce from header
    aad: &[u8],                   // Additional authenticated data
    chunk_size: usize,
) -> Result<usize, Box<dyn std::error::Error>> {
    let cipher = Aes256Gcm::new(key);
    let mut total_bytes_written = 0;

    // Buffer for plaintext. Encrypted data is written back into this buffer.
    let mut plaintext_buffer = vec![0u8; chunk_size];

    // Extract fixed IV part (first 4 bytes) and initial counter value (last 8 bytes)
    let fixed_iv_part: [u8; 4] = initial_nonce[0..4].try_into().unwrap(); // Should not panic due to IV_LEN
    let mut counter_bytes = [0u8; 8];
    counter_bytes.copy_from_slice(&initial_nonce[4..12]);
    let mut current_nonce_value = u64::from_be_bytes(counter_bytes);

    loop {
        // Read plaintext into the buffer
        let bytes_read = reader.read(&mut plaintext_buffer[..])?;
        if bytes_read == 0 {
            break; // EOF
        }

        // Construct the nonce for this chunk
        let mut full_nonce_bytes = [0u8; IV_LEN];
        full_nonce_bytes[0..4].copy_from_slice(&fixed_iv_part); // Fixed part
        full_nonce_bytes[4..12].copy_from_slice(&current_nonce_value.to_be_bytes()); // Incrementing counter

        let nonce = Nonce::<Aes256Gcm>::from_slice(&full_nonce_bytes);

        // Encrypt in place. `plaintext_buffer` will contain ciphertext.
        let tag = cipher
            .encrypt_in_place_detached(
                nonce,
                aad,                                 // Associated data for this chunk
                &mut plaintext_buffer[..bytes_read], // Buffer contains plaintext and will be overwritten with ciphertext
            )
            .map_err(|e| {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string(),
                )) as Box<dyn std::error::Error>
            })?;

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
    initial_nonce: &[u8; IV_LEN], // Full 12-byte initial nonce from header
    aad: &[u8],                   // Additional authenticated data
    chunk_size: usize,
) -> Result<usize, Box<dyn std::error::Error>> {
    let cipher = Aes256Gcm::new(key);
    let mut total_bytes_written = 0;

    // Buffer for decrypted plaintext.
    let mut plaintext_buffer = vec![0u8; chunk_size];
    // Temporary buffer to read encrypted chunk + tag
    let mut ciphertext_read_buffer = vec![0u8; chunk_size + TAG_LEN];

    // Extract fixed IV part (first 4 bytes) and initial counter value (last 8 bytes)
    let fixed_iv_part: [u8; 4] = initial_nonce[0..4].try_into().unwrap(); // Should not panic due to IV_LEN
    let mut counter_bytes = [0u8; 8];
    counter_bytes.copy_from_slice(&initial_nonce[4..12]);
    let mut current_nonce_value = u64::from_be_bytes(counter_bytes);

    loop {
        // Read encrypted chunk (ciphertext + tag) into the combined buffer
        let bytes_read_combined = reader.read(&mut ciphertext_read_buffer[..])?;
        if bytes_read_combined == 0 {
            break; // EOF
        }

        if bytes_read_combined < TAG_LEN {
            return Err("Truncated data: encrypted chunk too short to contain a tag".into());
        }

        let current_ciphertext_len = bytes_read_combined - TAG_LEN;

        // Extract ciphertext and tag parts
        let mut tag_buffer = [0u8; TAG_LEN];
        tag_buffer
            .copy_from_slice(&ciphertext_read_buffer[current_ciphertext_len..bytes_read_combined]);

        // Copy ciphertext part to the plaintext_buffer (which will be modified in-place)
        plaintext_buffer[..current_ciphertext_len]
            .copy_from_slice(&ciphertext_read_buffer[..current_ciphertext_len]);

        // Reconstruct the nonce for this chunk
        let mut full_nonce_bytes = [0u8; IV_LEN];
        full_nonce_bytes[0..4].copy_from_slice(&fixed_iv_part); // Fixed part
        full_nonce_bytes[4..12].copy_from_slice(&current_nonce_value.to_be_bytes()); // Incrementing counter

        let nonce = Nonce::<Aes256Gcm>::from_slice(&full_nonce_bytes);

        // Decrypt in place. `plaintext_buffer` contains ciphertext and will be overwritten with plaintext.
        cipher
            .decrypt_in_place_detached(
                nonce,
                aad,                                             // Associated data for this chunk
                &mut plaintext_buffer[..current_ciphertext_len], // Buffer contains ciphertext
                &Tag::<Aes256Gcm>::from_slice(&tag_buffer),      // Tag
            )
            .map_err(|e| {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Decryption error: {}", e.to_string()),
                )) as Box<dyn std::error::Error>
            })?;

        writer.write_all(&plaintext_buffer[..current_ciphertext_len])?; // Write only the plaintext part
        total_bytes_written += current_ciphertext_len;

        current_nonce_value += 1;
    }

    Ok(total_bytes_written)
}

pub fn generate_random_bytes<const N: usize>() -> [u8; N] {
    let mut bytes = [0u8; N];
    OsRng.fill_bytes(&mut bytes);
    bytes
}
