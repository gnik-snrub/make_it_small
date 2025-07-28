use std::{cmp::Ordering, collections::BinaryHeap};

pub fn build_huffman_tree(freq: &[u64; 256]) -> Option<Node> {
    let mut heap: BinaryHeap<Node> = BinaryHeap::new();

    for (i, b) in freq.iter().enumerate() {
        if *b > 0 {
            let node = Node {
                weight: *b,
                symbol: Some(i as u8),
                left: None,
                right: None,
            };
            heap.push(node);
        }
    }

    while heap.len() > 1 {
        debug_assert!(heap.len() >= 2);
        let low_1 = heap.pop().unwrap();
        let low_2 = heap.pop().unwrap();

        let parent = Node {
            weight: low_1.weight + low_2.weight,
            symbol: None,
            left: Some(Box::new(low_1)),
            right: Some(Box::new(low_2)),
        };

        heap.push(parent);
    }

    heap.pop()
}

#[derive(Clone, Debug)]
pub struct Node {
    pub weight: u64,
    pub symbol: Option<u8>,
    pub left: Option<Box<Node>>,
    pub right: Option<Box<Node>>,
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.weight == other.weight
    }
}

impl Eq for Node {
}

impl Ord for Node {
    fn cmp(&self, other: &Node) -> Ordering {
        other.weight.cmp(&self.weight)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub fn serialize_tree(node: &Node, out: &mut Vec<u8>) {
    if node.symbol.is_none() && (node.left.is_none() || node.right.is_none()) {
        // Do nothing, so early return
        return
    }

    if let Some(symbol) = node.symbol {
        // Leaf node
        out.push(1);
        out.push(symbol);
    } else {
        // Internal node
        out.push(0);
        match (&node.left, &node.right) {
            (Some(left), Some(right)) => {
                serialize_tree(left, out);
                serialize_tree(right, out);
            }
            _ => {
                eprintln!("Corrupted node: {:?}", node);
                panic!("Internal node missing a child - invalid Huffman tree")
            },
        }
    }
}

pub fn deserialize_tree<I: Iterator<Item = u8> + ExactSizeIterator>(stream: &mut I) -> Node {
    if stream.len() < 1 {
        return Node {
            weight: 0,
            symbol: None,
            left: None,
            right: None,
        }
    }
    match stream.next() {
        Some(1) => {
            let symbol = stream.next().expect("Expected symbol after leaf marker");
            Node {
                weight: 0,
                symbol: Some(symbol),
                left: None,
                right: None,
            }
        }
        Some(0) => {
            let left = deserialize_tree(stream);
            let right = deserialize_tree(stream);
            Node {
                weight: 0,
                symbol: None,
                left: Some(Box::new(left)),
                right: Some(Box::new(right)),
            }
        }
        _ => {
            panic!("Invalid or corrupted tree data")
        }
    }
}
