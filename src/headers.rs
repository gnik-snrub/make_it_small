use crate::{constants::{MAGIC_BYTES, VERSION}, huffman::tree::{deserialize_tree, serialize_tree, Node}, crypto::IV_LEN};
use std::io::Read;

#[derive(Debug, Clone)]
pub struct Headers {
    pub magic_bytes: [u8; 4],
    pub version: u8,
    pub flags: u8,
    pub original_size: u64,
    pub original_file_name: String,
    pub compressed_size: u64,
    pub payload_actual_size: u64,
    pub salt: [u8; 16],
    pub iv: [u8; IV_LEN],
    pub tag: [u8; 16],
    pub padding_bits: u8,
    pub checksum: u32,
    pub tree: Node,
}

pub fn write_header(original_size: u64, checksum: u32, name: &str) -> Headers {
    Headers {
        magic_bytes: MAGIC_BYTES,
        version: VERSION,
        flags: 0b0000_0000, // TODO - Create a bitmask from selected flags using consts
        original_size,
        original_file_name: name.to_string(),
        compressed_size: 0,
        payload_actual_size: 0,
        salt: [0u8; 16], // Initialized to zeros
        iv: [0u8; IV_LEN],   // Initialized to zeros
        tag: [0u8; 16],  // Initialized to zeros
        padding_bits: 0,
        checksum,
        tree: Node { weight: 0, symbol: None, left: None, right: None },
    }
}

impl Default for Headers {
    fn default() -> Self {
        Self::new()
    }
}

impl Headers {
    fn read_bytes<R: Read, const N: usize>(reader: &mut R) -> Result<[u8; N], Box<dyn std::error::Error>> {
        let mut buf = [0u8; N];
        reader.read_exact(&mut buf)?;
        Ok(buf)
    }

    pub fn new() -> Self {
        Headers {
            magic_bytes: MAGIC_BYTES,
            version: VERSION,
            flags: 0,
            original_size: 0,
            original_file_name: String::new(),
            compressed_size: 0,
            payload_actual_size: 0,
            salt: [0u8; 16],
            iv: [0u8; IV_LEN],
            tag: [0u8; 16],
            padding_bits: 0,
            checksum: 0,
            tree: Node { weight: 0, symbol: None, left: None, right: None },
        }
    }

    pub fn to_bytes(self) -> Vec<u8> {
        let mut bytes = vec![];

        bytes.extend_from_slice(&self.magic_bytes);
        bytes.extend_from_slice(&self.version.to_le_bytes());
        bytes.extend_from_slice(&self.flags.to_le_bytes());
        bytes.extend_from_slice(&self.original_size.to_le_bytes());
        bytes.extend_from_slice(&(self.original_file_name.len() as u16).to_le_bytes());
        bytes.extend_from_slice(self.original_file_name.as_bytes());
        bytes.extend_from_slice(&self.compressed_size.to_le_bytes());
        bytes.extend_from_slice(&self.payload_actual_size.to_le_bytes());
        bytes.extend_from_slice(&self.salt);
        bytes.extend_from_slice(&self.iv);
        bytes.extend_from_slice(&self.tag);
        bytes.extend_from_slice(&self.padding_bits.to_le_bytes());
        bytes.extend_from_slice(&self.checksum.to_le_bytes());

        let mut out: Vec<u8> = Vec::new();
        serialize_tree(&self.tree, &mut out);

        let tree_length = out.len() as u16;
        bytes.extend_from_slice(&tree_length.to_le_bytes());
        bytes.extend_from_slice(&out);

        bytes
    }

    pub fn from_reader<R: Read>(reader: &mut R) -> Result<Headers, Box<dyn std::error::Error>> {
        let magic_bytes = Self::read_bytes::<R, 4>(reader)?;

        let version = Self::read_bytes::<R, 1>(reader)?[0];

        let flags = Self::read_bytes::<R, 1>(reader)?[0];

        let original_size = u64::from_le_bytes(Self::read_bytes::<R, 8>(reader)?);

        let file_name_len = u16::from_le_bytes(Self::read_bytes::<R, 2>(reader)?);
        let mut name_bytes = vec![0u8; file_name_len as usize];
        reader.read_exact(&mut name_bytes)?;
        let original_file_name = String::from_utf8(name_bytes)?;

        let compressed_size = u64::from_le_bytes(Self::read_bytes::<R, 8>(reader)?);
        let payload_actual_size = u64::from_le_bytes(Self::read_bytes::<R, 8>(reader)?);

        let salt = Self::read_bytes::<R, 16>(reader)?;
        let iv = Self::read_bytes::<R, IV_LEN>(reader)?;
        let tag = Self::read_bytes::<R, 16>(reader)?;

        let padding_bits = Self::read_bytes::<R, 1>(reader)?[0];

        let checksum = u32::from_le_bytes(Self::read_bytes::<R, 4>(reader)?);

        let tree_len = u16::from_le_bytes(Self::read_bytes::<R, 2>(reader)?);
        let mut tree_bytes = vec![0u8; tree_len as usize];
        reader.read_exact(&mut tree_bytes)?;
        let mut stream = tree_bytes.into_iter(); // Deserialize tree from collected bytes
        let tree = deserialize_tree(&mut stream);

        Ok(Headers {
           magic_bytes,
           version,
           flags,
           original_size,
           original_file_name,
           compressed_size,
           payload_actual_size,
           salt,
           iv,
           tag,
           padding_bits,
           checksum,
           tree
        })
    }
}

