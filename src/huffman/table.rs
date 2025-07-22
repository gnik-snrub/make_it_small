use super::tree::Node;

pub fn generate_code_table(root: &Node) -> [(u64, u8); 256] {
    let mut table = [(0u64, 0u8); 256];

    walk(&mut table, root, 0u64, 0u8);

    table
}

fn walk(table: &mut [(u64, u8); 256], node: &Node, curr_bits: u64, curr_len: u8) {
    let left = &node.left;
    let right = &node.right;

    if is_leaf(node) {
        let sym = node.symbol;
        let effective_len = if curr_len == 0 { 1 } else { curr_len };
        let packed = curr_bits << (64 - effective_len);
        table[sym.unwrap() as usize] = (packed, effective_len);
        return;
    } else {
        if let Some(left_node) = left {
            walk(table, &left_node, curr_bits << 1, curr_len + 1);
        }
        if let Some(right_node) = right {
            walk(table, &right_node, (curr_bits << 1) | 1, curr_len + 1);
        }
    }
}

fn is_leaf(n: &Node) -> bool {
    n.left.is_none() && n.right.is_none()
}
