pub fn compute_frequencies(buf: &[u8]) -> [u64; 256] {
    let mut freq = [0u64; 256];

    for byte in buf {
        freq[*byte as usize] += 1;
    }

    freq
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
