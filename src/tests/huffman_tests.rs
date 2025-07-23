use crate::huffman::freq::compute_frequencies;
use crate::huffman::{table::generate_code_table, tree::{build_huffman_tree, Node}};

#[cfg(test)]
// compute_frequencies tests
#[test]
fn freq_empty_slice() {
    let buf: &[u8] = b"";
    let table = compute_frequencies(buf);

    assert!(table.iter().all(|&c| c == 0));
}

#[test]
fn freq_single_byte() {
    let buf: &[u8] = &[42];
    let table = compute_frequencies(buf);

    assert_eq!(table[42], 1);
    assert_eq!(table.iter().sum::<u64>(), 1);
}

#[test]
fn freq_known_mix() {
    // 0x3, 1x2, 2x1
    let buf: &[u8] = &[0, 0, 0, 1, 1, 2];
    let table = compute_frequencies(buf);

    assert_eq!(table[0], 3);
    assert_eq!(table[1], 2);
    assert_eq!(table[2], 1);
    assert_eq!(table.iter().sum::<u64>(), buf.len() as u64);
}

#[test]
fn freq_hello_world() {
    let buf: &[u8] = b"hello world";
    let table = compute_frequencies(buf);

    assert_eq!(table[b'h' as usize], 1);
    assert_eq!(table[b'e' as usize], 1);
    assert_eq!(table[b'l' as usize], 3);
    assert_eq!(table[b'o' as usize], 2);
    assert_eq!(table[b' ' as usize], 1);
    assert_eq!(table[b'w' as usize], 1);
    assert_eq!(table[b'r' as usize], 1);
    assert_eq!(table[b'd' as usize], 1);
    assert_eq!(table.iter().sum::<u64>(), buf.len() as u64);
}

// build_huffman_tree tests
#[test]
fn tree_empty_returns_none() {
    let freq = [0u64; 256];
    assert!(build_huffman_tree(&freq).is_none())
}

#[test]
fn tree_single_symbol() {
    let mut freq = [0u64; 256];
    freq[42] = 10;
    let root = build_huffman_tree(&freq).unwrap();

    assert_eq!(root.weight, 10);
    assert_eq!(root.symbol, Some(42));
    assert!(root.left.is_none() && root.right.is_none());
}

#[test]
fn tree_hellow_world_total_weight() {
    let buf = b"hello world";
    let freq = compute_frequencies(buf);
    let root = build_huffman_tree(&freq).unwrap();

    assert_eq!(root.weight, buf.len() as u64);
}

#[test]
fn tree_leaf_weights_sum_to_root() {
    let mut freq = [0u64; 256];
    freq[0] = 3;
    freq[1] = 2;
    freq[2] = 1;

    let root = build_huffman_tree(&freq).unwrap();

    fn recurse(n: &Node) -> u64 {
        if let Some(_) = n.symbol {
            return n.weight;
        }
        recurse(n.left.as_ref().unwrap()) + recurse(n.right.as_ref().unwrap())
    }

    let leaf_sum = recurse(&root);
    assert_eq!(leaf_sum, root.weight);
}

// Code table tests
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
