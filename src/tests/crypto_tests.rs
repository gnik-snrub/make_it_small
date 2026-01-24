#[cfg(test)]
mod tests {
    use crate::crypto::{
        self, decrypt_stream, derive_key, encrypt_stream, generate_random_bytes, DEFAULT_CHUNK_SIZE,
    };
    use std::io::Cursor;

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
    fn test_encrypt_decrypt_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let original_data =
            b"This is a secret message that should be encrypted and then decrypted successfully.";
        let password = b"test_password_123";
        let salt = generate_random_bytes::<{ crypto::SALT_LEN }>();
        let iv = generate_random_bytes::<{ crypto::IV_LEN }>();

        let key = derive_key(password, &salt);

        // Encrypt the data
        let mut input_cursor = Cursor::new(original_data.to_vec());
        let mut encrypted_buffer = Cursor::new(Vec::new());

        let encrypted_size = encrypt_stream(
            &mut input_cursor,
            &mut encrypted_buffer,
            &key,
            &iv,
            &[],
            DEFAULT_CHUNK_SIZE,
        )?;

        // Decrypt the data
        encrypted_buffer.set_position(0);
        let mut decrypted_buffer = Cursor::new(Vec::new());

        let decrypted_size = decrypt_stream(
            &mut encrypted_buffer,
            &mut decrypted_buffer,
            &key,
            &iv,
            &[],
            DEFAULT_CHUNK_SIZE,
        )?;

        // Verify roundtrip integrity
        // Note: AES-GCM adds authentication tags, so encrypted_size > original_data.len()
        assert!(encrypted_size > original_data.len());
        assert_eq!(decrypted_size, original_data.len());
        assert_eq!(decrypted_buffer.into_inner(), original_data.to_vec());

        Ok(())
    }

    #[test]
    fn test_encrypt_decrypt_with_different_chunk_sizes() -> Result<(), Box<dyn std::error::Error>> {
        let original_data =
            b"Test data for different chunk sizes during encryption and decryption.";
        let password = b"chunk_test_password";
        let salt = generate_random_bytes::<{ crypto::SALT_LEN }>();
        let iv = generate_random_bytes::<{ crypto::IV_LEN }>();
        let key = derive_key(password, &salt);

        let chunk_sizes = [1024, 4096, 16384];

        for chunk_size in chunk_sizes.iter() {
            // Encrypt
            let mut input_cursor = Cursor::new(original_data.to_vec());
            let mut encrypted_buffer = Cursor::new(Vec::new());

            let encrypted_size = encrypt_stream(
                &mut input_cursor,
                &mut encrypted_buffer,
                &key,
                &iv,
                &[],
                *chunk_size,
            )?;

            // Decrypt
            encrypted_buffer.set_position(0);
            let mut decrypted_buffer = Cursor::new(Vec::new());

            let decrypted_size = decrypt_stream(
                &mut encrypted_buffer,
                &mut decrypted_buffer,
                &key,
                &iv,
                &[],
                *chunk_size,
            )?;

            // Note: AES-GCM adds authentication tags, so encrypted_size > original_data.len()
            assert!(encrypted_size > original_data.len());
            assert_eq!(decrypted_size, original_data.len());
            assert_eq!(decrypted_buffer.into_inner(), original_data.to_vec());
        }

        Ok(())
    }

    #[test]
    fn test_encryption_with_wrong_key_fails() -> Result<(), Box<dyn std::error::Error>> {
        let original_data = b"Secret message";
        let correct_password = b"correct_password";
        let wrong_password = b"wrong_password";
        let salt = generate_random_bytes::<{ crypto::SALT_LEN }>();
        let iv = generate_random_bytes::<{ crypto::IV_LEN }>();

        let correct_key = derive_key(correct_password, &salt);
        let wrong_key = derive_key(wrong_password, &salt);

        // Encrypt with correct key
        let mut input_cursor = Cursor::new(original_data.to_vec());
        let mut encrypted_buffer = Cursor::new(Vec::new());

        encrypt_stream(
            &mut input_cursor,
            &mut encrypted_buffer,
            &correct_key,
            &iv,
            &[],
            DEFAULT_CHUNK_SIZE,
        )?;

        // Try to decrypt with wrong key
        encrypted_buffer.set_position(0);
        let mut decrypted_buffer = Cursor::new(Vec::new());

        let result = decrypt_stream(
            &mut encrypted_buffer,
            &mut decrypted_buffer,
            &wrong_key,
            &iv,
            &[],
            DEFAULT_CHUNK_SIZE,
        );

        // Should fail with decryption error
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_random_byte_generation() {
        let bytes1 = generate_random_bytes::<32>();
        let bytes2 = generate_random_bytes::<32>();

        // Should be different (very high probability)
        assert_ne!(bytes1, bytes2);
        assert_eq!(bytes1.len(), 32);
        assert_eq!(bytes2.len(), 32);
    }

    #[test]
    fn test_empty_data_encryption() -> Result<(), Box<dyn std::error::Error>> {
        let original_data = b"";
        let password = b"empty_test_password";
        let salt = generate_random_bytes::<{ crypto::SALT_LEN }>();
        let iv = generate_random_bytes::<{ crypto::IV_LEN }>();
        let key = derive_key(password, &salt);

        // Encrypt empty data
        let mut input_cursor = Cursor::new(original_data.to_vec());
        let mut encrypted_buffer = Cursor::new(Vec::new());

        let encrypted_size = encrypt_stream(
            &mut input_cursor,
            &mut encrypted_buffer,
            &key,
            &iv,
            &[],
            DEFAULT_CHUNK_SIZE,
        )?;

        // Decrypt empty data
        encrypted_buffer.set_position(0);
        let mut decrypted_buffer = Cursor::new(Vec::new());

        let decrypted_size = decrypt_stream(
            &mut encrypted_buffer,
            &mut decrypted_buffer,
            &key,
            &iv,
            &[],
            DEFAULT_CHUNK_SIZE,
        )?;

        assert_eq!(encrypted_size, 0);
        assert_eq!(decrypted_size, 0);
        assert_eq!(decrypted_buffer.into_inner(), Vec::<u8>::new());

        Ok(())
    }
}
