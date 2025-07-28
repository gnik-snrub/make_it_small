use crate::{headers::write_header, io::BitWriter};

use super::{freq::compute_frequencies, table::generate_code_table, tree::{build_huffman_tree, serialize_tree}};

pub fn encode(src: &Vec<u8>, name: &str) -> Vec<u8> {

    let mut header = write_header(src, name);

    let freq = compute_frequencies(src);

    let (codes, lengths) = match build_huffman_tree(&freq) {
        Some(tree) => { 
            let table = generate_code_table(&tree);
            header.tree = tree;
            (table.map(|c| c.0), table.map(|c| c.1))
        }
        None => {
            ([0u32; 256], [0u8; 256])
        }
    };

    let tree = header.tree.clone();
    let mut serialized_tree = Vec::new();
    serialize_tree(&tree, &mut serialized_tree);
    let header_len = 43 + name.len() + serialized_tree.len();
    let mut out: Vec<u8> = Vec::with_capacity(header_len + src.len());
    out.extend_from_slice(&header.to_bytes());

    let mut writer = BitWriter::new();

    for b in src {
        let code= codes[*b as usize];
        let len = lengths[*b as usize];
        let bits = (code >> (32 - len)) & ((1u32 << len) - 1);
        writer.write_bits(bits, len);
        out.extend_from_slice(&writer.take_bytes());
    }

    writer.finalize();
    out.extend_from_slice(&writer.take_bytes());
    let padding_bits = writer.padding_bits as u8;

    let compressed_size = (out.len() - header_len) as u64;
    let mut final_header = write_header(src, name);
    final_header.compressed_size = compressed_size;
    final_header.padding_bits = padding_bits;
    final_header.tree = tree;

    out[..header_len].copy_from_slice(&final_header.to_bytes());

    out
}

pub struct Encoded {
    pub output: Vec<u8>,
    pub padding_bits: u8,
    pub code_table: [(u32, u8); 256],
    pub lengths: [u8; 256],
}
