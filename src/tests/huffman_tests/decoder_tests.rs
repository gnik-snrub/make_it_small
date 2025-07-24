use crate::huffman::decoder::rebuild_table;

#[cfg(test)]

fn blank_lengths() -> [u8; 256] { [0u8; 256] }

#[test]
fn all_zeroes() {
    let table = rebuild_table(&blank_lengths());
    assert!(table.iter().all(|&(code, len)| code == 0 && len == 0));
}

#[test]
fn single_symbol_len1() {
    let mut lengths = blank_lengths();
    lengths[0x2A] = 1; // Arbitrary
    let table = rebuild_table(&lengths);

    assert_eq!(table[0x2A as usize], (0, 1));
    assert!(table.iter().enumerate().all(|(i, &entry)|
        if i == 0x2A { entry == (0, 1) } else { entry == (0, 0) }
    ));
}

#[test]
fn two_symbols_len1() {
    let mut lengths = blank_lengths();
    lengths[0x41] = 1; // 'A'
    lengths[0x42] = 1; // 'B'
    let table = rebuild_table(&lengths);

    assert_eq!(table[0x41], (0, 1));
    assert_eq!(table[0x42], (1, 1));
}

#[test]
fn staircase_lengths() {
    let mut lengths = blank_lengths();
    lengths[0x00] = 1; // 'A'
    lengths[0x01] = 2; // 'B'
    lengths[0x02] = 3; // 'B'
    let table = rebuild_table(&lengths);

    assert_eq!(table[0x00], (0b0, 1));
    assert_eq!(table[0x01], (0b10, 2));
    assert_eq!(table[0x02], (0b110, 3));
}
