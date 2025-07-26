use crate::{headers::write_header, io::BitWriter};

use super::{freq::compute_frequencies, table::generate_code_table, tree::build_huffman_tree};

pub fn encode(src: &Vec<u8>, name: &str) -> Vec<u8> {
    let header_len = 41 + name.len() + 256; // 41 is the total sum of bytes for header
                                                   // variables sans lengths. name length varies.
                                                   // 256 represents the canonical code table
                                                   // lengths.
    let mut out: Vec<u8> = Vec::with_capacity(header_len + src.len());

    let header = write_header(src, name);
    out.extend_from_slice(&header.to_bytes());

    let freq = compute_frequencies(src);

    let (codes, lengths) = match build_huffman_tree(&freq) {
        Some(tree) => { 
            let table = generate_code_table(&tree);
            (table, table.map(|c| c.1))
        }
        None => {
            ([(0u32, 0u8); 256], [0u8; 256])
        }
    };

    let mut writer = BitWriter::new();

    for b in src {
        let (code, code_len) = codes[*b as usize];
        let bits = (code >> (32 - code_len)) & ((1u32 << code_len) - 1);
        writer.write_bits(bits, code_len);
        out.extend_from_slice(&writer.take_bytes());
    }

    if writer.len > 0 {
        writer.flush();
        out.extend_from_slice(&writer.take_bytes());
    }

    let compressed_size = (out.len() - header_len) as u64;
    let padding_bits = writer.padding_bits as u8;
    let mut final_header = write_header(src, name);
    final_header.compressed_size = compressed_size;
    final_header.padding_bits = padding_bits;
    final_header.lengths = lengths;

    out[..header_len].copy_from_slice(&final_header.to_bytes());

    out
}

pub struct Encoded {
    pub output: Vec<u8>,
    pub padding_bits: u8,
    pub code_table: [(u32, u8); 256],
    pub lengths: [u8; 256],
}
