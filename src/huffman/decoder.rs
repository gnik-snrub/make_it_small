pub fn rebuild_table(lengths: &[u8; 256]) -> [(u32, u8); 256] {
    let mut active_symbols = vec![];

    for (sym, &len) in lengths.iter().enumerate() {
        if len > 0 {
            active_symbols.push((sym as u8, len));
        }
    }

    active_symbols.sort_by_key(|&(sym, len)| (len, sym));
    
    let mut table = [(0u32, 0u8); 256];

    let mut prev_len = 0;
    let mut code = 0u32;
    for (sym, len) in active_symbols {
        debug_assert!(len <= 32);
        if len > prev_len {
            code <<= (len - prev_len) as u32;
            prev_len = len;
        }
        table[sym as usize] = (code, len);
        code += 1;
    }

    table
}
