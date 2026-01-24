#[cfg(test)]
mod tests {
    use crate::checksum::adler32;

    #[test]
    fn test_empty_data() {
        assert_eq!(adler32(b""), 1);
    }

    #[test]
    fn test_single_byte() {
        assert_eq!(adler32(b"a"), 6422626);
    }

    #[test]
    fn test_hello_world() {
        assert_eq!(adler32(b"Hello, World!"), 530449514);
    }

    #[test]
    fn test_long_string() {
        let data = b"The quick brown fox jumps over the lazy dog";
        assert_eq!(adler32(data), 1541148634);
    }

    #[test]
    fn test_all_zeros() {
        let data = [0u8; 10];
        assert_eq!(adler32(&data), 655361);
    }

    #[test]
    fn test_different_data() {
        assert_ne!(adler32(b"abc"), adler32(b"abd"));
    }
}
