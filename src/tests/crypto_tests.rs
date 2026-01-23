#[cfg(test)]
mod tests {
    use crate::crypto::{self, derive_key};
    

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
}
