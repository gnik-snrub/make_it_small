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
