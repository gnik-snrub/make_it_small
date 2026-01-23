use std::io::{Read, Write};

pub struct BitWriter<W: Write> {
    pub buffer: u8,
    pub len: u8,
    pub writer: W,
    pub padding_bits: usize,
}



impl<W: Write> BitWriter<W> {
    pub fn new(writer: W) -> BitWriter<W> {
        BitWriter { buffer: 0, len: 0, writer, padding_bits: 0 }
    }

    pub fn write_bits(&mut self, value: u32, bit_count: u8) -> std::io::Result<()> {
        if bit_count == 0 {
            return Ok(());
        }
        for i in (0..bit_count).rev() {
            let bit = (value >> i) & 1;
            self.buffer <<= 1;
            self.buffer |= bit as u8;
            self.len += 1;

            if self.len == 8 {
                self.writer.write_all(&[self.buffer])?;
                self.buffer = 0;
                self.len = 0;
            }
        }
        Ok(())
    }

    pub fn flush(&mut self) -> std::io::Result<()> {
        for _ in 0..(8 - self.len) {
            self.buffer <<= 1;
            self.buffer |= 0_u8;
            self.padding_bits += 1;
        }
        self.writer.write_all(&[self.buffer])?;
        self.buffer = 0;
        self.len = 0;
        Ok(())
    }

    pub fn finalize(&mut self) -> std::io::Result<()> {
        if self.len == 0 {
            self.padding_bits = 0;
            return Ok(())
        }

        for _ in 0..(8 - self.len) {
            self.buffer <<= 1;
        }

        self.writer.write_all(&[self.buffer])?;
        self.padding_bits = (8 - self.len) as usize;

        self.buffer = 0;
        self.len = 0;
        Ok(())
    }


}

pub struct BitReader<R: Read> {
    pub reader: R,
    buffer: [u8; 4096], // 4KB buffer
    buffer_pos: usize,
    buffer_len: usize,
    current_byte: u8,
    bit_pos: u8,
}

impl<R: Read> BitReader<R> {
    pub fn new(_padding_bits: usize, reader: R) -> BitReader<R> {
        BitReader {
            reader,
            buffer: [0; 4096],
            buffer_pos: 0,
            buffer_len: 0,
            current_byte: 0,
            bit_pos: 8, // Set to 8 so the first read_byte attempt triggers a buffer read
        }
    }

    fn fill_buffer(&mut self) -> std::io::Result<usize> {
        self.buffer_pos = 0;
        self.buffer_len = self.reader.read(&mut self.buffer)?;
        Ok(self.buffer_len)
    }

    pub fn read_bit(&mut self) -> std::io::Result<Option<u8>> {
        if self.bit_pos == 8 {
            // Current byte exhausted, or starting fresh. Need a new byte.
            if self.buffer_pos == self.buffer_len {
                // Buffer exhausted, refill it.
                let bytes_read = self.fill_buffer()?;
                if bytes_read == 0 {
                    return Ok(None); // EOF
                }
            }
            self.current_byte = self.buffer[self.buffer_pos];
            self.buffer_pos += 1;
            self.bit_pos = 0;
        }

        let bit = (self.current_byte >> (7 - self.bit_pos)) & 1;
        self.bit_pos += 1;

        Ok(Some(bit))
    }
}

