use crate::io::BitWriter;

use super::{freq::compute_frequencies, table::generate_code_table, tree::build_huffman_tree};

pub fn encode(src: &Vec<u8>) -> Encoded {
    let freq = compute_frequencies(src);

    let root = match build_huffman_tree(&freq) {
        Some(tree) => { tree }
        None => {
            return Encoded {
                output: Vec::new(),
                padding_bits: 0,
                code_table: [(0, 0); 256]
            };
        }
    };
    let codes = generate_code_table(&root);

    let mut writer = BitWriter::new();

    for b in src {
        let (code, code_len) = codes[*b as usize];
        let bits = (code >> (32 - code_len)) & ((1u32 << code_len) - 1);
        writer.write_bits(bits, code_len);
    }

    if writer.len > 0 {
        writer.flush();
    }
    Encoded {
        output: writer.output,
        padding_bits: writer.padding_bits as u8,
        code_table: codes,
    }
}

pub struct Encoded {
    pub output: Vec<u8>,
    pub padding_bits: u8,
    pub code_table: [(u32, u8); 256]
}
