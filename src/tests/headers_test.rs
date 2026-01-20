#[cfg(test)]
use crate::headers::{write_header, Headers};

#[test]
fn headers_round_trip() {
    let mut original = write_header(b"test".as_ref(), "testfile.txt");
    original.tree = crate::huffman::tree::Node {
        weight: 0,
        symbol: Some(b'a'),
        left: None,
        right: None,
    };

    let bytes = original.clone().to_bytes();
    let reconstructed = match Headers::from_reader(&mut std::io::Cursor::new(&bytes)) {
        Ok(header) => {
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
    assert_eq!(original.salt, reconstructed.salt);
    assert_eq!(original.iv, reconstructed.iv);
    assert_eq!(original.tag, reconstructed.tag);
    assert_eq!(original.padding_bits, reconstructed.padding_bits);
    assert_eq!(original.checksum, reconstructed.checksum);
}
