#[cfg(test)]
use crate::huffman::compute_frequencies;
use crate::huffman::{build_huffman_tree, Node};

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
