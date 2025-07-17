
pub fn read_file(path: &str) -> Vec<u8> {
    match std::fs::read(path) {
        Ok(bytes) => {
            bytes
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            vec![]
        }
    }
}

pub fn write_file(file: Vec<u8>, path: &str) {
    let result = std::fs::write(path, file);
    match result {
        Ok(_) => {
            println!("File successfully created at {path}");
        }
        Err(e) => {
            eprintln!("Error: {e}");
        }
    }
}

struct BitWriter {
    buffer: u8,
    len: u8,
    output: Vec<u8>,
    padding_bits: usize,
}

impl BitWriter {
    fn new() -> BitWriter {
        BitWriter { buffer: 0, len: 0, output: vec![], padding_bits: 0 }
    }

    fn write_bits(&mut self, value: u32, bit_count: u8) {
        if bit_count == 0 {
            return;
        }
        for i in (0..bit_count).rev() {
            let bit = (value >> i) & 1;
            self.buffer <<= 1;
            self.buffer |= bit as u8;
            self.len += 1;

            if self.len == 8 {
                self.output.push(self.buffer);
                self.buffer = 0;
                self.len = 0;
            }
        }
    }

    fn flush(&mut self) {
        for _ in 0..(8 - self.len) {
            self.buffer <<= 1;
            self.buffer |= 0 as u8;
            self.padding_bits += 1;
        }
        self.output.push(self.buffer);
        self.buffer = 0;
        self.len = 0;
    }
}

struct BitReader {
    input: Vec<u8>,
    byte_pos: usize,
    bit_pos: u8,
    total_bits: usize,
    bits_read: usize,
}

impl BitReader {
    fn new(padding_bits: usize, input: &[u8]) -> BitReader {
        BitReader {
            input: input.to_vec(),
            byte_pos: 0,
            bit_pos: 0,
            total_bits: (input.len() * 8).saturating_sub(padding_bits),
            bits_read: 0
        }
    }

    fn read_bit(&mut self) -> Option<u8> {
        if self.bits_read >= self.total_bits {
            return None
        }

        let current_byte = self.input[self.byte_pos];
        let bit = (current_byte >> (7 - self.bit_pos)) & 1;

        self.bit_pos += 1;
        if self.bit_pos == 8 {
            self.bit_pos = 0;
            self.byte_pos += 1;
        }

        self.bits_read += 1;

        Some(bit)
    }
}

