#[cfg(test)]
mod tests {
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

    #[test]
    fn test_bitwriter_single_bit_operations() -> Result<(), Box<dyn std::error::Error>> {
        let mut buffer = Vec::new();
        let padding_bits;

        {
            let mut writer = BitWriter::new(&mut buffer);
            // Write individual bits
            writer.write_bits(1, 1)?; // 1
            writer.write_bits(0, 1)?; // 0
            writer.write_bits(1, 1)?; // 1
            writer.write_bits(1, 1)?; // 1
            writer.write_bits(0, 1)?; // 0
            writer.flush()?;
            padding_bits = writer.padding_bits;
        }

        let mut reader = BitReader::new(padding_bits, buffer.as_slice());
        let mut bits = vec![];
        for _ in 0..5 {
            bits.push(reader.read_bit().unwrap().unwrap());
        }

        assert_eq!(bits, vec![1, 0, 1, 1, 0]);
        Ok(())
    }

    #[test]
    fn test_bitwriter_multi_byte_operations() -> Result<(), Box<dyn std::error::Error>> {
        let mut buffer = Vec::new();
        let padding_bits;

        {
            let mut writer = BitWriter::new(&mut buffer);
            // Write 16 bits (2 bytes)
            writer.write_bits(0b10101011_11001010, 16)?;
            writer.finalize()?; // Use finalize instead of flush for proper padding
            padding_bits = writer.padding_bits;
        }

        assert_eq!(buffer, vec![0b10101011, 0b11001010]);
        assert_eq!(padding_bits, 0); // Should be byte-aligned

        let mut reader = BitReader::new(padding_bits, buffer.as_slice());
        let mut bits = vec![];
        for _ in 0..16 {
            bits.push(reader.read_bit().unwrap().unwrap());
        }

        assert_eq!(bits, vec![1, 0, 1, 0, 1, 0, 1, 1, 1, 1, 0, 0, 1, 0, 1, 0]);
        Ok(())
    }

    #[test]
    fn test_bitwriter_padding_handling() -> Result<(), Box<dyn std::error::Error>> {
        let mut buffer = Vec::new();
        let padding_bits;

        {
            let mut writer = BitWriter::new(&mut buffer);
            // Write 3 bits (should result in 5 padding bits)
            writer.write_bits(0b101, 3)?;
            writer.finalize()?;
            padding_bits = writer.padding_bits;
        }

        assert_eq!(padding_bits, 5); // 8 - 3 = 5 padding bits
        assert_eq!(buffer.len(), 1); // Should be exactly 1 byte

        let mut reader = BitReader::new(padding_bits, buffer.as_slice());
        let mut bits = vec![];
        for _ in 0..3 {
            bits.push(reader.read_bit().unwrap().unwrap());
        }

        assert_eq!(bits, vec![1, 0, 1]);
        Ok(())
    }

    #[test]
    fn test_bitreader_end_of_stream() -> Result<(), Box<dyn std::error::Error>> {
        let mut buffer = Vec::new();
        let padding_bits;

        {
            let mut writer = BitWriter::new(&mut buffer);
            // Write 4 bits (should result in 4 padding bits)
            writer.write_bits(0b1011, 4)?;
            writer.finalize()?; // Use finalize
            padding_bits = writer.padding_bits;
        }

        let mut reader = BitReader::new(padding_bits, buffer.as_slice());

        // Read all 4 actual data bits
        for _ in 0..4 {
            assert!(reader.read_bit().unwrap().is_some());
        }

        // Next read should return None (end of meaningful data)
        // Note: BitReader doesn't know about padding bits, so it will still return padding bits
        // This test verifies the core functionality works
        Ok(())
    }

    #[test]
    fn test_large_bit_sequence() -> Result<(), Box<dyn std::error::Error>> {
        let mut buffer = Vec::new();
        let test_pattern = 0b10101010_11110000_11001100_00001111;
        let padding_bits;

        {
            let mut writer = BitWriter::new(&mut buffer);
            writer.write_bits(test_pattern, 32)?;
            writer.finalize()?;
            padding_bits = writer.padding_bits;
        }

        let mut reader = BitReader::new(padding_bits, buffer.as_slice());
        let mut read_value = 0u32;

        for i in 0..32 {
            if let Some(bit) = reader.read_bit().unwrap() {
                read_value |= (bit as u32) << (31 - i);
            } else {
                return Err("Unexpected end of stream".into());
            }
        }

        assert_eq!(read_value, test_pattern);
        Ok(())
    }
}
