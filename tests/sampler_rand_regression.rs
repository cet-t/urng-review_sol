#[cfg(any(feature = "sampler", feature = "rand"))]
use urng::Rng;

#[cfg(feature = "sampler")]
#[test]
fn samplers_handle_extreme_finite_weights_and_never_select_zero_weights() {
    use urng::Sampler;
    macro_rules! check {
        ($rng:ident,$ty:ident,$float:ty) => {{
            for weights in [
                [<$float>::MAX, <$float>::MAX],
                [<$float>::from_bits(1), <$float>::from_bits(1)],
            ] {
                let mut r = urng::$rng::new(1);
                let mut s = urng::$ty::new(&mut r, &weights);
                let first = (0..4096).filter(|_| s.sample() == 0).count();
                assert!(
                    (1800..2300).contains(&first),
                    "{}: {first}",
                    stringify!($ty)
                );
                s.weights(&[0.0, 1.0, 0.0]);
                for _ in 0..1024 {
                    assert_eq!(s.sample(), 1);
                }
            }
        }};
    };
    check!(Mt19937, Alias32, f32);
    check!(Mt19937, Bst32, f32);
    check!(Mt1993764, Alias64, f64);
    check!(Mt1993764, Bst64, f64);
    struct Zero32;
    impl Rng for Zero32 {
        type Word = u32;
        fn nextu(&mut self) -> u32 {
            0
        }
    }
    struct Zero64;
    impl Rng for Zero64 {
        type Word = u64;
        fn nextu(&mut self) -> u64 {
            0
        }
    }
    assert_eq!(urng::Bst32::new(&mut Zero32, &[0.0, 1.0]).sample(), 1);
    assert_eq!(urng::Bst64::new(&mut Zero64, &[0.0, 1.0]).sample(), 1);
}

#[cfg(feature = "sampler")]
#[test]
fn invalid_weights_are_rejected_at_construction() {
    macro_rules! check {
        ($ty:ident,$rng:ident,$float:ty) => {{
            for weights in [
                vec![],
                vec![0.0],
                vec![-1.0, 2.0],
                vec![<$float>::NAN],
                vec![<$float>::INFINITY],
            ] {
                assert!(
                    std::panic::catch_unwind(|| {
                        let mut r = urng::$rng::new(1);
                        let _ = urng::$ty::new(&mut r, &weights);
                    })
                    .is_err()
                );
            }
        }};
    };
    check!(Alias32, Mt19937, f32);
    check!(Bst32, Mt19937, f32);
    check!(Alias64, Mt1993764, f64);
    check!(Bst64, Mt1993764, f64);
}

#[cfg(feature = "rand")]
#[test]
fn rand_32_bit_adapters_handle_full_words_byte_tails_and_little_endian_seeds() {
    use rand::{Rng as RandRng, SeedableRng};
    macro_rules! check {($($ty:ident),+)=>{$({
        let mut adapter=urng::$ty::from_seed([1,2,3,4]);
        let mut native=urng::$ty::new(0x04030201);
        assert_eq!(adapter.next_u32(),urng::Rng::nextu(&mut native));
        let high=urng::Rng::nextu(&mut native) as u64;
        let low=urng::Rng::nextu(&mut native) as u64;
        assert_eq!(adapter.next_u64(),high<<32|low);
        for n in [0,1,3,4,5,7,9,17,31] {
            let mut dst=vec![0xa5;n+2];let mut expected=Vec::new();
            for _ in 0..n.div_ceil(4) {expected.extend_from_slice(&urng::Rng::nextu(&mut native).to_le_bytes());}
            adapter.fill_bytes(&mut dst[1..n+1]);
            assert_eq!(&dst[1..n+1],&expected[..n]);assert_eq!(dst[0],0xa5);assert_eq!(dst[n+1],0xa5);
        }
    })+};}
    check!(
        Jsf32,
        Mt19937,
        Sfmt607,
        Sfmt1279,
        Sfmt2281,
        Sfmt4253,
        Sfmt11213,
        Sfmt19937,
        Sfmt44497,
        Sfmt86243,
        Sfmt132049,
        Sfmt216091,
        Pcg32,
        Philox32,
        Sfc32,
        SplitMix32,
        Squares32,
        Threefry32x2,
        Threefry32x4,
        Xoroshiro64Ss,
        Xorshift32,
        Xorshift128,
        Xorwow,
        Xoshiro128Pp,
        Xoshiro128Ss
    );
}

#[cfg(feature = "rand")]
#[test]
fn rand_64_bit_adapters_draw_full_words_and_initialize_byte_tails() {
    use rand::{Rng as RandRng, SeedableRng};
    macro_rules! check {($($ty:ident),+)=>{$({
        let mut adapter=urng::$ty::seed_from_u64(123);
        let mut native=urng::$ty::seed_from_u64(123);
        assert_eq!(adapter.next_u64(),urng::Rng::nextu(&mut native));
        for n in [0,1,7,8,9,17,31] {
            let mut dst=vec![0xa5;n+2];let mut expected=Vec::new();
            for _ in 0..n.div_ceil(8) {expected.extend_from_slice(&urng::Rng::nextu(&mut native).to_le_bytes());}
            adapter.fill_bytes(&mut dst[1..n+1]);
            assert_eq!(&dst[1..n+1],&expected[..n]);assert_eq!(dst[0],0xa5);assert_eq!(dst[n+1],0xa5);
        }
    })+};}
    check!(
        Biski64,
        Cet64,
        Cet256,
        Mt1993764,
        Sfmt1993764,
        Philox64,
        Sfc64,
        SplitMix64,
        Threefish256,
        TwistedGFSR,
        Xoroshiro128Pp,
        Xoroshiro128Ss,
        Xorshift64,
        Xoshiro256Pp,
        Xoshiro256Ss,
        WyHash64
    );
    let mut a = <urng::Sfc64 as SeedableRng>::from_seed([1, 2, 3, 4]);
    let mut b = urng::Sfc64::new(0x04030201);
    assert_eq!(a.next_u64(), urng::Rng::nextu(&mut b));
}
