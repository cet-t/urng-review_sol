use urng::{ChaCha20, Rng, Salsa20};

#[test]
fn chacha20_rfc8439_entire_block_and_counter_continuation() {
    // RFC 8439 section 2.3.2: independent normative known-answer vector.
    let key = std::array::from_fn(|i| i as u8);
    let nonce = [0, 0, 0, 9, 0, 0, 0, 0x4a, 0, 0, 0, 0];
    let mut r = ChaCha20::from_key_nonce(key, nonce, 1);
    assert_eq!(
        std::array::from_fn::<_, 16, _>(|_| r.nextu()),
        [
            0xe4e7f110, 0x15593bd1, 0x1fdd0f50, 0xc47120a3, 0xc7f4d1c7, 0x0368c033, 0x9aaa2204,
            0x4e6cd4c3, 0x466482d2, 0x09aa9f07, 0x05d7c214, 0xa2028bd9, 0xd19c12b5, 0xb94e16de,
            0xe883d0cb, 0x4e3c50a2
        ]
    );
    let mut next = ChaCha20::from_key_nonce(key, nonce, 2);
    for _ in 0..32 {
        assert_eq!(r.nextu(), next.nextu());
    }
}
#[test]
fn salsa20_zero_key_nonce_known_answer() {
    // Salsa20/20, zero 256-bit key, zero nonce, counter zero, little-endian words.
    let expected = "9a97f65b9b4c721b960a672145fca8d4e32e67f9111ea979ce9c4826806aeee63de9c0da2bd7f91ebcb2639bf989c6251b29bf38d39a9bdce7c55f4b2ac12a39";
    let mut r = Salsa20::from_key_nonce([0; 32], [0; 8], 0);
    let actual: Vec<u8> = (0..16).flat_map(|_| r.nextu().to_le_bytes()).collect();
    let got = actual
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    assert_eq!(got, expected);
}
#[test]
fn salsa_counter_carries_without_changing_nonce() {
    let mut r = Salsa20::from_key_nonce([1; 32], [2; 8], u32::MAX as u64);
    for _ in 0..16 {
        let _ = r.nextu();
    }
    let mut next = Salsa20::from_key_nonce([1; 32], [2; 8], 1u64 << 32);
    for _ in 0..32 {
        assert_eq!(r.nextu(), next.nextu());
    }
}
#[test]
fn final_counter_block_is_available_then_exhaustion_panics() {
    let mut ch = ChaCha20::from_key_nonce([1; 32], [2; 12], u32::MAX);
    let mut sa = Salsa20::from_key_nonce([1; 32], [2; 8], u64::MAX);
    for _ in 0..16 {
        let _ = ch.nextu();
        let _ = sa.nextu();
    }
    assert!(std::panic::catch_unwind(move || ch.nextu()).is_err());
    assert!(std::panic::catch_unwind(move || sa.nextu()).is_err());
}
