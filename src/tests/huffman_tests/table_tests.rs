use crate::huffman::{freq::compute_frequencies, table::generate_code_table, tree::build_huffman_tree};

#[cfg(test)]
// Helper function
fn table_for(buf: &[u8]) -> [(u32, u8); 256] {
    let freq = compute_frequencies(buf);
    let root = build_huffman_tree(&freq).unwrap();
    generate_code_table(&root)
}

#[test]
fn single_symbol_len_is_one() {
    let buf = b"AAAAAAAA";
    let table = table_for(buf);

    assert_eq!(table[b'A' as usize].1, 1);
    assert_eq!(table[b'A' as usize].0 & 1, 0);
}

#[test]
fn completeness_for_input_bytes() {
    let buf = b"hellow world";
    let table = table_for(buf);

    for &b in buf {
        assert!(table[b as usize].1 > 0, "byte {b} is missing in table");
    }
}

#[test]
fn codes_are_prefix_free() {
    let buf = b"test string";
    let table = table_for(buf);

    let codes: Vec<(u32, u8)> = table
        .iter()
        .filter(|&&(_, len)| len > 0)
        .cloned()
        .collect();

    println!("Table: {:?}", codes);

    for (i, &(bits_a, len_a)) in codes.iter().enumerate() {
        for &(bits_b, len_b) in &codes[i + 1..] {
            let min_len = len_a.min(len_b);
            let mask = if min_len == 32 { u32::MAX } else { u32::MAX << (32 - min_len) };
            assert!((bits_a & mask) != (bits_b & mask), "prefix collision detected between {:b} (len {}) and {:b} (len {})", bits_a, len_a, bits_b, len_b);
        }
    }
}

#[test]
fn encoded_bit_count_matches_formula() {
    let buf = b"some reasonably long sample text to vary frequencies";
    let freq = compute_frequencies(buf);
    let root = build_huffman_tree(&freq).unwrap();
    let table = generate_code_table(&root);

    let expected_bits: u32 = freq
        .iter()
        .enumerate()
        .map(|(sym, &count)| {
            let (_, len) = table[sym];
            count as u32 * len as u32
        })
        .sum();

    let max_possible = buf.len() as u32 * 64;

    assert!(expected_bits > 0 && expected_bits <= max_possible)
}

#[test]
fn table_is_stable_across_runs() {
    let buf = b"hello world";
    let first = table_for(buf);
    let second = table_for(buf);
    assert_eq!(first, second);
}
