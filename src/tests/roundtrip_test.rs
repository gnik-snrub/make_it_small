#[cfg(test)]
#[test]
fn roundtrip_file_compression() {
    use crate::io::{read_file, BitReader, BitWriter};
    use crate::headers::{write_header, Headers};

    let test_files = vec![
        "src/tests/testdata/null_bytes.bin",
        "src/tests/testdata/weird_bytes.bin",
        "src/tests/testdata/test.txt",
    ];

    for path in test_files {

        let original = read_file(path);

        let mut writer = BitWriter::new();

        for byte in &original {
            writer.write_bits(*byte as u32, 8);
        }

        let compressed = writer.output.clone();

        let mut header = write_header(&original, "test.txt");
        header.tree = crate::huffman::tree::Node {
            weight: 0,
            symbol: Some(b'a'),
            left: None,
            right: None,
        };
        let mut stitched = header.to_bytes();
        stitched.extend_from_slice(&compressed);

        let (rebuilt_header, header_len) = Headers::from_bytes(&stitched).unwrap();
        let compressed_section = &stitched[header_len..];

        let mut reader = BitReader::new(rebuilt_header.padding_bits as usize, compressed_section);

        let mut output = vec![];
        for _ in 0..rebuilt_header.original_size {
            let mut byte = 0u8;
            for _ in 0..8 {
                byte = (byte << 1) | reader.read_bit().unwrap();
            }
            output.push(byte);
        }


        assert_eq!(original, output);
    }
}
