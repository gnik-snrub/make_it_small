use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce, Key // Or `aes_gcm::Key`
};
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;
use aes_gcm::aead::rand_core::RngCore;

// Key, Salt, IV, Tag lengths for AES-256 GCM
pub const KEY_LEN: usize = 32; // 256 bits
pub const SALT_LEN: usize = 16;
pub const IV_LEN: usize = 12; // 96 bits for GCM
pub const TAG_LEN: usize = 16; // Authentication Tag

pub fn derive_key(password: &[u8], salt: &[u8]) -> Key<Aes256Gcm> {
    let mut key_bytes = [0u8; KEY_LEN];
    pbkdf2_hmac::<Sha256>(password, salt, 100_000, &mut key_bytes); // 100k iterations
    key_bytes.into() // Use into()
}

pub fn encrypt(data: &[u8], key: &Key<Aes256Gcm>, iv: &[u8]) -> Result<(Vec<u8>, [u8; TAG_LEN]), Box<dyn std::error::Error>> {
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(iv); // 96-bit; unique per message!

    let ciphertext_with_tag = cipher.encrypt(nonce, data).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    
    // The `encrypt` method returns ciphertext concatenated with the tag.
    // We need to separate them.
    let ciphertext_len = ciphertext_with_tag.len() - TAG_LEN;
    let ciphertext = ciphertext_with_tag[..ciphertext_len].to_vec();
    let mut tag_bytes = [0u8; TAG_LEN];
    tag_bytes.copy_from_slice(&ciphertext_with_tag[ciphertext_len..]);

    Ok((ciphertext, tag_bytes))
}

pub fn decrypt(
    ciphertext: &[u8],
    key: &Key<Aes256Gcm>,
    iv: &[u8],
    tag: &[u8],
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(iv); // 96-bit

    // Concatenate ciphertext and tag for decryption
    let mut ciphertext_with_tag = ciphertext.to_vec();
    ciphertext_with_tag.extend_from_slice(tag);

    let plaintext = cipher.decrypt(nonce, ciphertext_with_tag.as_slice()).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    Ok(plaintext)
}

pub fn generate_random_bytes<const N: usize>() -> [u8; N] {
    let mut bytes = [0u8; N];
    OsRng.fill_bytes(&mut bytes);
    bytes
}