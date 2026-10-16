//! Independent C-generated sequences and published known-answer vectors.
use crate::*;
use wrapn::wrap;

fn compare(name: &str, mut next: impl FnMut() -> u64) {
    let line = include_str!("../tests/reference/author_vectors.txt")
        .lines()
        .find(|line| line.split_whitespace().next() == Some(name))
        .unwrap();
    assert_eq!(
        line.split_whitespace().count(),
        17,
        "{name}: fixture must contain 16 outputs"
    );
    for expected in line.split_whitespace().skip(1) {
        assert_eq!(next(), u64::from_str_radix(expected, 16).unwrap(), "{name}");
    }
}

#[test]
fn scalar_algorithms_match_independently_compiled_unsigned_c() {
    let mut r = Xoshiro128Pp {
        s: wrap![1, 2, 3, 4],
    };
    compare("xoshiro128pp", || r.nextu() as u64);
    let mut r = Xoshiro128Ss {
        s: wrap![1, 2, 3, 4],
    };
    compare("xoshiro128ss", || r.nextu() as u64);
    let mut r = Xoshiro256Pp {
        s: wrap![1, 2, 3, 4],
    };
    compare("xoshiro256pp", || r.nextu());
    let mut r = Xoshiro256Ss {
        s: wrap![1, 2, 3, 4],
    };
    compare("xoshiro256ss", || r.nextu());
    let mut r = Xoroshiro64Ss { s: wrap![1, 2] };
    compare("xoroshiro64ss", || r.nextu() as u64);
    let mut r = Xoroshiro128Pp { s: wrap![1, 2] };
    compare("xoroshiro128pp", || r.nextu());
    let mut r = Xoroshiro128Ss { s: wrap![1, 2] };
    compare("xoroshiro128ss", || r.nextu());
    let mut r = Sfc32 {
        a: wrap!(1),
        b: wrap!(2),
        c: wrap!(3),
        counter: wrap!(4),
    };
    compare("sfc32", || r.nextu() as u64);
    let mut r = Sfc64 {
        a: wrap!(1),
        b: wrap!(2),
        c: wrap!(3),
        counter: wrap!(4),
    };
    compare("sfc64", || r.nextu());
    let mut r = Jsf32 {
        a: wrap!(1),
        b: wrap!(2),
        c: wrap!(3),
        d: wrap!(4),
    };
    compare("jsf32", || r.nextu() as u64);
    let mut r = Xorshift32 { a: wrap!(1) };
    compare("xorshift32", || r.nextu() as u64);
    let mut r = Xorshift128 {
        x: wrap![1, 2, 3, 4],
    };
    compare("xorshift128", || r.nextu() as u64);
    let mut r = Xorwow {
        x: wrap![1, 2, 3, 4, 5],
        c: wrap!(6),
    };
    compare("xorwow", || r.nextu() as u64);
    let mut r = SplitMix32 { state: wrap!(1) };
    compare("splitmix32", || r.nextu() as u64);
    let mut r = SplitMix64 { s: wrap!(1) };
    compare("splitmix64", || r.nextu());
    let mut r = Biski64 {
        fast_loop: wrap!(1),
        mix: wrap!(2),
        loop_mix: wrap!(3),
    };
    compare("biski64", || r.nextu());
    let mut r = Cet64 { s: wrap!(1) };
    compare("cet64", || r.nextu());
    let mut r = Cet256 {
        s: wrap![1, 2, 3, 4],
    };
    compare("cet256", || r.nextu());
    let mut r = Cet256 {
        s: wrap![u64::MAX;4],
    };
    compare("cet256_carry", || r.nextu());
    let mut r = Pcg32 {
        state: wrap!(0),
        inc: wrap!(109),
    };
    let _ = r.nextu();
    r.state += 42;
    let _ = r.nextu();
    let mut published = r;
    // PCG's published seed=(42,54) example. Also pins the independent C oracle.
    assert_eq!(
        std::array::from_fn::<_, 6, _>(|_| published.nextu()),
        [
            0xa15c02b7, 0x7b47f409, 0xba1d3330, 0x83d2f293, 0xbfa4784b, 0xcbed606e
        ]
    );
    compare("pcg32", || r.nextu() as u64);
    let mut r = Squares32 {
        c: wrap!(0),
        k: wrap!(0xc8e4fd154ce32f6d),
    };
    compare("squares32", || r.nextu() as u64);
}

#[test]
fn xoroshiro_jumps_match_the_authors_c_reference() {
    let mut r = Xoroshiro128Pp { s: wrap![1, 2] };
    r.jump();
    compare("xoroshiro128pp_jump", || r.nextu());
    let mut r = Xoroshiro128Pp { s: wrap![1, 2] };
    r.long_jump();
    compare("xoroshiro128pp_long_jump", || r.nextu());
    let mut r = Xoroshiro128Ss { s: wrap![1, 2] };
    r.jump();
    compare("xoroshiro128ss_jump", || r.nextu());
    let mut r = Xoroshiro128Ss { s: wrap![1, 2] };
    r.long_jump();
    compare("xoroshiro128ss_long_jump", || r.nextu());
}

#[test]
fn random123_published_counter_key_known_answers() {
    // https://github.com/DEShawResearch/random123/blob/main/tests/kat_vectors
    use crate::cbrng::{b32::*, b64::*};
    let got = Philox32::compute(wrap![0;4], wrap![0;2]);
    assert_eq!(
        got.map(|w| *w),
        [0x6627e8d5, 0xe169c58d, 0xbc57ac4c, 0x9b00dbd8]
    );
    let got = Philox32::compute(wrap![u32::MAX;4], wrap![u32::MAX;2]);
    assert_eq!(
        got.map(|w| *w),
        [0x408f276d, 0x41c83b0e, 0xa20bc7c6, 0x6d5451fd]
    );
    let got = Philox64::compute(wrap![0;2], wrap![0;2]);
    assert_eq!(got.map(|w| *w), [0xca00a0459843d731, 0x66c24222c9a845b5]);
    let got = Philox4x64::compute(wrap![0;4], wrap![0;2]);
    assert_eq!(
        got.map(|w| *w),
        [
            0x16554d9eca36314c,
            0xdb20fe9d672d0fdc,
            0xd7e772cee186176b,
            0x7e68b68aec7ba23b
        ]
    );
    let k = wrap![0, 0, 0x1bd11bda];
    assert_eq!(
        Threefry32x2::compute(wrap![0;2], &k).map(|w| *w),
        [0x6b200159, 0x99ba4efe]
    );
    let k = wrap![0, 0, 0, 0, 0x1bd11bda];
    assert_eq!(
        Threefry32x4::compute(wrap![0;4], &k, &wrap![0;3]).map(|w| *w),
        [0x9c6ca96a, 0xe17eae66, 0xfc10ecd4, 0x5256a7d8]
    );
    let c = wrap![0x243f6a88, 0x85a308d3, 0x13198a2e, 0x03707344];
    let words = [0xa4093822, 0x299f31d0, 0x082efa98, 0xec4e6c89];
    let k = wrap![
        words[0],
        words[1],
        words[2],
        words[3],
        words.into_iter().fold(0x1bd11bda, |a, b| a ^ b)
    ];
    assert_eq!(
        Threefry32x4::compute(c, &k, &wrap![0;3]).map(|w| *w),
        [0x59cd1dbb, 0xb8879579, 0x86b5d00c, 0xac8b6d84]
    );
}

#[test]
fn pcg_increment_is_odd_for_each_seed() {
    for seed in 0..1024 {
        assert_eq!(*Pcg32::new(seed).inc & 1, 1);
    }
}

#[test]
fn fresh_counter_rngs_refill_instead_of_returning_initial_zeroes() {
    let mut scalar = Threefish256::new(123);
    let mut raw = scalar;
    let expected = raw.next_raw().map(|w| *w);
    assert_eq!(std::array::from_fn::<_, 4, _>(|_| scalar.nextu()), expected);
    let mut scalar = Philox4x64::new(123);
    let mut raw = Philox4x64::new(123);
    assert_eq!(
        std::array::from_fn::<_, 4, _>(|_| scalar.nextu()),
        raw.next_raw().map(|w| *w)
    );
}

#[test]
fn threefish256_skein_13_zero_known_answer() {
    // Skein 1.3 / Threefish-256: zero key, tweak, and plaintext.
    // https://www.schneier.com/wp-content/uploads/2015/01/skein.pdf
    let mut r = Threefish256::new(1);
    r.c = wrap![0;4];
    r.k = wrap![0, 0, 0, 0, 0x1bd11bdaa9fc1a22];
    r.tw = wrap![0;3];
    assert_eq!(
        r.next_raw().map(|w| *w),
        [
            0x94eeea8b1f2ada84,
            0xadf103313eae6670,
            0x952419a1f4b16d53,
            0xd83f13e63c9f6b11
        ]
    );
    // Nonzero plaintext/key/tweak vector from Skein 1.3's NIST CD, also
    // published by Bouncy Castle. This detects accidental UBI feed-forward
    // (XOR with the counter), which the all-zero plaintext cannot detect.
    // https://github.com/bcgit/bc-csharp/blob/master/crypto/test/src/crypto/test/Threefish256Test.cs
    let words: [u64; 4] = std::array::from_fn(|i| {
        u64::from_le_bytes(std::array::from_fn(|j| (0x10 + i * 8 + j) as u8))
    });
    r.k = std::array::from_fn(|i| {
        wrap!(if i < 4 {
            words[i]
        } else {
            words.into_iter().fold(0x1bd11bdaa9fc1a22, |a, b| a ^ b)
        })
    });
    r.tw = wrap![0x0706050403020100, 0x0f0e0d0c0b0a0908, 0x0808080808080808];
    r.c = std::array::from_fn(|i| {
        wrap!(u64::from_le_bytes(std::array::from_fn(
            |j| 0xff - (i * 8 + j) as u8
        )))
    });
    assert_eq!(
        r.next_raw().map(|w| *w),
        [
            0xdf8fea0eff91d0e0,
            0xd50ad82ee69281c9,
            0x76f48d58085d869d,
            0xdf975e95b5567065,
        ]
    );
}
