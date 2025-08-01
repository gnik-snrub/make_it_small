use crate::flags::is_stored_raw;
use crate::headers::Headers;
use crate::constants::{MAGIC_BYTES, VERSION};
use crate::io::BitReader;

pub fn decode(src: Vec<u8>) -> (Vec<u8>, String) {
    let (header, end_of_header) = match Headers::from_bytes(&src) {
        Err(_) => {
            eprintln!("Error parsing file header");
            return (vec![], String::new())
        }
        Ok((header, cursor)) => {
            (header, cursor)
        }
    };

    if header.magic_bytes != MAGIC_BYTES {
        eprintln!("Error: Not a valid .small file");
        return (vec![], String::new())
    }

    if header.version != VERSION {
        eprintln!("Error: Incorrect version");
        return (vec![], String::new())
    }

    if is_stored_raw(header.flags) {
        return (Vec::from(&src[end_of_header..]), header.original_file_name)
    }
    
    let mut reader = BitReader::new(header.padding_bits as usize, &src[end_of_header..]);
    let mut out = Vec::with_capacity(header.original_size as usize);
    let mut current = &header.tree;

    while out.len() < header.original_size as usize {
        if let Some(byte) = current.symbol {
            out.push(byte);
            current = &header.tree;
            continue;
        }

        match reader.read_bit() {
            Some(0) => current = current.left.as_ref().expect("Missing left"),
            Some(1) => current = current.right.as_ref().expect("Missing left"),
            None => {
                eprintln!("Corrupted: Unexpected end of file");
                return (out, header.original_file_name)
            }
            _ => {

            }
        }
    }

    (out, header.original_file_name)
}
