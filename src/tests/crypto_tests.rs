#[cfg(test)]
mod tests {
    use crate::crypto::{self, derive_key, encrypt, decrypt};
    

    #[test]
    fn test_key_derivation() {
        let password = b"mysecretpassword";
        let salt = [0x00; crypto::SALT_LEN]; // All zeros for deterministic testing

        let key = derive_key(password, &salt);
        // Assert that a key is derived and it's of the correct size
        assert_eq!(key.as_slice().len(), crypto::KEY_LEN);

        // Test with a different password, should yield a different key
        let password2 = b"anotherpassword";
        let key2 = derive_key(password2, &salt);
        assert_ne!(key.as_slice(), key2.as_slice());

        // Test with a different salt, should yield a different key
        let salt2 = [0xFF; crypto::SALT_LEN];
        let key3 = derive_key(password, &salt2);
        assert_ne!(key.as_slice(), key3.as_slice());
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let password = b"testpassword";
        let salt = crypto::generate_random_bytes::<{crypto::SALT_LEN}>();
        let iv = crypto::generate_random_bytes::<{crypto::IV_LEN}>();
        let original_data = b"This is some sensitive data to be encrypted.";

        let key = derive_key(password, &salt);

        let (ciphertext, tag) = encrypt(original_data, &key, &iv).expect("Encryption failed");

        // Decrypt with the correct key and IV
        let decrypted_data = decrypt(&ciphertext, &key, &iv, &tag).expect("Decryption failed");

        assert_eq!(original_data, decrypted_data.as_slice());
    }

    #[test]
    fn test_decrypt_with_wrong_password() {
        let password = b"testpassword";
        let wrong_password = b"wrongpassword";
        let salt = crypto::generate_random_bytes::<{crypto::SALT_LEN}>();
        let iv = crypto::generate_random_bytes::<{crypto::IV_LEN}>();
        let original_data = b"Some data.";

        let key = derive_key(password, &salt);
        let (ciphertext, tag) = encrypt(original_data, &key, &iv).expect("Encryption failed");

        let wrong_key = derive_key(wrong_password, &salt);
        let result = decrypt(&ciphertext, &wrong_key, &iv, &tag);

        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_with_wrong_iv() {
        let password = b"testpassword";
        let salt = crypto::generate_random_bytes::<{crypto::SALT_LEN}>();
        let iv = crypto::generate_random_bytes::<{crypto::IV_LEN}>();
        let wrong_iv = crypto::generate_random_bytes::<{crypto::IV_LEN}>();
        let original_data = b"Some data.";

        let key = derive_key(password, &salt);
        let (ciphertext, tag) = encrypt(original_data, &key, &iv).expect("Encryption failed");

        let result = decrypt(&ciphertext, &key, &wrong_iv, &tag);

        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_with_tampered_ciphertext() {
        let password = b"testpassword";
        let salt = crypto::generate_random_bytes::<{crypto::SALT_LEN}>();
        let iv = crypto::generate_random_bytes::<{crypto::IV_LEN}>();
        let original_data = b"Some data.";

        let key = derive_key(password, &salt);
        let (mut ciphertext, tag) = encrypt(original_data, &key, &iv).expect("Encryption failed");

        // Tamper with the ciphertext
        if !ciphertext.is_empty() {
            ciphertext[0] ^= 0x01; // Flip a bit
        }

        let result = decrypt(&ciphertext, &key, &iv, &tag);

        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_with_tampered_tag() {
        let password = b"testpassword";
        let salt = crypto::generate_random_bytes::<{crypto::SALT_LEN}>();
        let iv = crypto::generate_random_bytes::<{crypto::IV_LEN}>();
        let original_data = b"Some data.";

        let key = derive_key(password, &salt);
        let (ciphertext, mut tag) = encrypt(original_data, &key, &iv).expect("Encryption failed");

        // Tamper with the tag
        tag[0] ^= 0x01; // Flip a bit

        let result = decrypt(&ciphertext, &key, &iv, &tag);

        assert!(result.is_err());
    }
}
