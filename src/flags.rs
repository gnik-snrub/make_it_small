use crate::constants::{ENCRYPTED, HAS_CHECKSUM, IS_ARCHIVE, STORED_RAW};

pub fn flip_encrypted(flags: &mut u8) {
    *flags |= ENCRYPTED;
}

pub fn is_encrypted(flags: u8) -> bool {
    flags & ENCRYPTED != 0
}

pub fn flip_has_checksum(flags: &mut u8) {
    *flags |= HAS_CHECKSUM;
}

pub fn has_checksum(flags: u8) -> bool {
    flags & HAS_CHECKSUM != 0
}

pub fn flip_stored_raw(flags: &mut u8) {
    *flags |= STORED_RAW;
}

pub fn is_stored_raw(flags: u8) -> bool {
    flags & STORED_RAW != 0
}

pub fn flip_is_archive(flags: &mut u8) {
    *flags |= IS_ARCHIVE;
}

pub fn is_archive(flags: u8) -> bool {
    flags & IS_ARCHIVE != 0
}
