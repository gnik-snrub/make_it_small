#[cfg(test)]
use crate::io::{BitReader, BitWriter};

#[test]
fn test_bitwriter_bitreader_roundtrip() {
    // 1. Write a sequence of bits
    let mut writer = BitWriter::new();

    writer.write_bits(0b101, 3);
    writer.write_bits(0b11, 2);
    writer.flush();

    assert_eq!(writer.output, vec![0b10111000]);

    // 2. Read them back
    let mut reader = BitReader::new(writer.padding_bits, &writer.output);

    let mut bits = vec![];
    for _ in 0..5 {
        bits.push(reader.read_bit().unwrap());
    }

    // 3. Verify read bits match original sequence
    assert_eq!(bits, vec![1, 0, 1, 1, 1]);
    assert_eq!(reader.bits_read, 5);
    assert_eq!(reader.total_bits, 8 - writer.padding_bits);
}
