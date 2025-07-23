#[cfg(test)]
use crate::huffman::{freq::compute_frequencies, tree::{build_huffman_tree, Node}};

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
