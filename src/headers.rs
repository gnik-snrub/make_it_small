use serde::{Deserialize, Serialize};

use crate::constants::{MAGIC_BYTES, VERSION};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Headers {
    magic_bytes: [u8; 4],
    version: u8,
    flags: u8,
    original_size: u64,
    original_file_name: String,
    compressed_size: u64,
    salt_and_iv: [u64; 2],
    padding_bits: u8,
}

pub fn write_header(file: &Vec<u8>, name: &str, padding_bits: u8) -> Headers {
    Headers {
        magic_bytes: MAGIC_BYTES,
        version: VERSION,
        flags: 0b0000_0000, // TODO - Create a bitmask from selected flags using consts
        original_size: file.len() as u64,
        original_file_name: name.to_string(),
        compressed_size: 0, // TODO - Check output file sizes, and put here
        // TODO - Randomize both values with a secure RNG
        salt_and_iv: [1234432112344321, 4321123443211234],
        padding_bits
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

        bytes
    }

    pub fn from_bytes(bytes: &Vec<u8>) -> (Headers, usize) {
        let mut cursor = 0;

        let magic_bytes = <[u8; 4]>::try_from(&bytes[cursor..cursor+4]).unwrap();
        cursor += 4;

        let version = bytes[cursor];
        cursor += 1;

        let flags = bytes[cursor];
        cursor += 1;

        let original_size = u64::from_le_bytes(bytes[cursor..cursor+8].try_into().unwrap());
        cursor += 8;

        let file_name_len = u16::from_le_bytes(bytes[cursor..cursor+2].try_into().unwrap()) as usize;
        cursor += 2;

        let name_bytes = &bytes[cursor..cursor+file_name_len];
        let original_file_name = String::from_utf8(name_bytes.to_vec()).unwrap();
        cursor += file_name_len;

        let compressed_size = u64::from_le_bytes(bytes[cursor..cursor+8].try_into().unwrap());
        cursor += 8;

        let salt = u64::from_le_bytes(bytes[cursor..cursor+8].try_into().unwrap());
        cursor += 8;
        let iv = u64::from_le_bytes(bytes[cursor..cursor+8].try_into().unwrap());
        cursor += 8;

        let padding_bits = bytes[cursor];
        cursor += 1;

        (Headers {
           magic_bytes,
           version,
           flags,
           original_size,
           original_file_name,
           compressed_size,
           salt_and_iv: [salt, iv],
           padding_bits,
        }, cursor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn headers_round_trip() {
        let original = Headers {
            magic_bytes: [0xDE, 0xAD, 0xBE, 0xEF],
            version: 1,
            flags: 0b1010_1010,
            original_size: 12345,
            compressed_size: 6789,
            original_file_name: "testfile.txt".to_string(),
            salt_and_iv: [987654321, 123456789],
            padding_bits: 3,
        };

        let bytes = original.clone().to_bytes();
        let reconstructed = Headers::from_bytes(&bytes).0;

        assert_eq!(original.magic_bytes, reconstructed.magic_bytes);
        assert_eq!(original.version, reconstructed.version);
        assert_eq!(original.flags, reconstructed.flags);
        assert_eq!(original.original_size, reconstructed.original_size);
        assert_eq!(original.compressed_size, reconstructed.compressed_size);
        assert_eq!(original.original_file_name, reconstructed.original_file_name);
        assert_eq!(original.salt_and_iv, reconstructed.salt_and_iv);
        assert_eq!(original.padding_bits, reconstructed.padding_bits);
    }
}
