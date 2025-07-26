#[cfg(test)]
use crate::headers::{write_header, Headers};

#[test]
fn headers_round_trip() {
    let original = write_header(&b"test".to_vec(), "testfile.txt");

    let bytes = original.clone().to_bytes();
    let reconstructed = match Headers::from_bytes(&bytes) {
        Ok((header, _)) => {
            header
        }
        Err(e) => {
            panic!("Error deserializing headers: {e}");
        }
    };

    assert_eq!(original.magic_bytes, reconstructed.magic_bytes);
    assert_eq!(original.version, reconstructed.version);
    assert_eq!(original.flags, reconstructed.flags);
    assert_eq!(original.original_size, reconstructed.original_size);
    assert_eq!(original.compressed_size, reconstructed.compressed_size);
    assert_eq!(original.original_file_name, reconstructed.original_file_name);
    assert_eq!(original.salt_and_iv, reconstructed.salt_and_iv);
    assert_eq!(original.padding_bits, reconstructed.padding_bits);
}
