use crate::{constants::{MAGIC_BYTES, VERSION}, huffman::tree::{deserialize_tree, serialize_tree, Node}};

#[derive(Debug, Clone)]
pub struct Headers {
    pub magic_bytes: [u8; 4],
    pub version: u8,
    pub flags: u8,
    pub original_size: u64,
    pub original_file_name: String,
    pub compressed_size: u64,
    pub salt_and_iv: [u64; 2],
    pub padding_bits: u8,
    pub tree: Node,
}

pub fn write_header(file: &Vec<u8>, name: &str) -> Headers {
    Headers {
        magic_bytes: MAGIC_BYTES,
        version: VERSION,
        flags: 0b0000_0000, // TODO - Create a bitmask from selected flags using consts
        original_size: file.len() as u64,
        original_file_name: name.to_string(),
        compressed_size: 0,
        // TODO - Randomize both values with a secure RNG
        salt_and_iv: [1234432112344321, 4321123443211234],
        padding_bits: 0,
        tree: Node { weight: 0, symbol: None, left: None, right: None },
    }
}

impl Headers {
    pub fn to_bytes(self) -> Vec<u8> {
        let mut bytes = vec![];

        bytes.extend_from_slice(&self.magic_bytes);
        bytes.extend_from_slice(&self.version.to_le_bytes());
        bytes.extend_from_slice(&self.flags.to_le_bytes());
        bytes.extend_from_slice(&self.original_size.to_le_bytes());
        bytes.extend_from_slice(&(self.original_file_name.len() as u16).to_le_bytes());
        bytes.extend_from_slice(&self.original_file_name.as_bytes());
        bytes.extend_from_slice(&self.compressed_size.to_le_bytes());
        bytes.extend_from_slice(&(self.salt_and_iv[0] as u64).to_le_bytes());
        bytes.extend_from_slice(&(self.salt_and_iv[1] as u64).to_le_bytes());
        bytes.extend_from_slice(&self.padding_bits.to_le_bytes());

        let mut out: Vec<u8> = Vec::new();
        serialize_tree(&self.tree, &mut out);

        let tree_length = out.len() as u16;
        bytes.extend_from_slice(&tree_length.to_le_bytes());
        bytes.extend_from_slice(&out);

        bytes
    }

    pub fn from_bytes(bytes: &Vec<u8>) -> Result<(Headers, usize), Box<dyn std::error::Error>> {
        let mut cursor = 0;

        let magic_bytes = <[u8; 4]>::try_from(&bytes[cursor..cursor+4])?;
        cursor += 4;

        let version = bytes[cursor];
        cursor += 1;

        let flags = bytes[cursor];
        cursor += 1;

        let original_size = u64::from_le_bytes(bytes[cursor..cursor+8].try_into()?);
        cursor += 8;

        let file_name_len = u16::from_le_bytes(bytes[cursor..cursor+2].try_into()?) as usize;
        cursor += 2;

        let name_bytes = &bytes[cursor..cursor+file_name_len];
        let original_file_name = String::from_utf8(name_bytes.to_vec())?;
        cursor += file_name_len;

        let compressed_size = u64::from_le_bytes(bytes[cursor..cursor+8].try_into()?);
        cursor += 8;

        let salt = u64::from_le_bytes(bytes[cursor..cursor+8].try_into()?);
        cursor += 8;
        let iv = u64::from_le_bytes(bytes[cursor..cursor+8].try_into()?);
        cursor += 8;

        let padding_bits = bytes[cursor];
        cursor += 1;

        let tree_len = u16::from_le_bytes([bytes[cursor], bytes[cursor + 1]]);
        cursor += 2;

        let tree_slice = &bytes[cursor..cursor + tree_len as usize];
        cursor += tree_len as usize;
        let mut stream = tree_slice.iter().copied();
        let tree = deserialize_tree(&mut stream);

        Ok((Headers {
           magic_bytes,
           version,
           flags,
           original_size,
           original_file_name,
           compressed_size,
           salt_and_iv: [salt, iv],
           padding_bits,
           tree
        }, cursor))
    }
}

