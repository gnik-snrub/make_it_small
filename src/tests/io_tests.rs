#[cfg(test)]
use crate::io::{BitReader, BitWriter};

#[test]

fn test_bitwriter_bitreader_roundtrip() -> Result<(), Box<dyn std::error::Error>> {

    // 1. Write a sequence of bits

    let mut buffer = Vec::new(); // Our mock writer

    let padding_bits;

    {

        let mut writer = BitWriter::new(&mut buffer);



        writer.write_bits(0b101, 3)?;

        writer.write_bits(0b11, 2)?;

        writer.flush()?;

        padding_bits = writer.padding_bits;

    } // writer is dropped here, releasing the mutable borrow on buffer



    assert_eq!(buffer, vec![0b10111000]);



    // 2. Read them back

    let mut reader = BitReader::new(padding_bits, buffer.as_slice());



    let mut bits = vec![];

    for _ in 0..5 {

        bits.push(reader.read_bit().unwrap().unwrap()); // Unwrap Result and then Option

    }



    // 3. Verify read bits match original sequence

    assert_eq!(bits, vec![1, 0, 1, 1, 1]);

    Ok(())

}
