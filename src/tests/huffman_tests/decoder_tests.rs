use crate::huffman::{decoder::decode, encoder::encode};

use rand::{RngCore, SeedableRng, rngs::StdRng};

#[cfg(test)]
#[test]
fn empty_round_trip() {
    let src = Vec::new();
    let file = encode(&src, "empty");
    let (out, _name) = decode(file);
    assert!(out.is_empty());
}

#[test]
fn single_byte_round_trip() {
    let src = b"a".to_vec();
    let file = encode(&src, "one");
    let (out, _name) = decode(file);
    assert_eq!(out, src);
}

#[test]
fn alternating_ab_round_trip() {
    let src = b"abababab".to_vec();
    let file = encode(&src, "alternating");
    let (out, _name) = decode(file);
    assert_eq!(out, src);
}

#[test]
fn full_ramp_round_trip() {
    let src: Vec<u8> = (0..=255).collect();
    let file = encode(&src, "ramp");
    let (out, _name) = decode(file);
    assert_eq!(out, src);
}

#[test]
fn random_1k_round_trip() {
    let mut rng = StdRng::seed_from_u64(42);
    let mut src = vec![0u8; 1024];
    rng.fill_bytes(&mut src);

    let file = encode(&src, "ramp");
    let (out, _name) = decode(file);
    assert_eq!(out, src);
}
