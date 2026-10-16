use urng::{Choice, Rng, Shuffle, Word};

struct Words32 {
    words: Vec<u32>,
    pos: usize,
}
impl Rng for Words32 {
    type Word = u32;
    fn nextu(&mut self) -> u32 {
        let x = self.words[self.pos];
        self.pos += 1;
        x
    }
}
#[test]
fn integer_full_width_and_wide_signed_offsets_do_not_overflow() {
    assert_eq!(u32::MAX.to_randi(i32::MIN, i32::MAX), i32::MAX);
    assert_eq!(u64::MAX.to_randi(i64::MIN, i64::MAX), i64::MAX);
    assert_eq!(0u32.to_randi(i32::MIN, i32::MAX), i32::MIN);
    for word in [0, 1, 0x80000000, u32::MAX] {
        let mut r = Words32 {
            words: vec![word],
            pos: 0,
        };
        assert_eq!(
            r.randi(i32::MIN, i32::MAX),
            word.to_randi(i32::MIN, i32::MAX)
        );
    }
}
#[test]
fn rejection_sampling_rejects_low_product_instead_of_biasing_zero() {
    let mut r = Words32 {
        words: vec![0, u32::MAX],
        pos: 0,
    };
    assert_eq!(r.randi(0, 2), 2);
    assert_eq!(r.pos, 2);
    let mut r = Words32 {
        words: vec![0, u32::MAX],
        pos: 0,
    };
    assert_eq!(r.index(3), 2);
    assert_eq!(r.pos, 2);
}
#[test]
fn float_ranges_exclude_upper_and_handle_finite_extremes() {
    let min = 1.0f32;
    let max = min.next_up();
    for x in [0, 1, 0x80000000, u32::MAX] {
        assert_eq!(x.to_randf(min, max), min);
    }
    for x in [0, 1, 1 << 63, u64::MAX] {
        let v = x.to_randf(-f64::MAX, f64::MAX);
        assert!(v.is_finite() && (-f64::MAX..f64::MAX).contains(&v));
        assert_eq!(x.to_randf(1.0f64, 1.0f64.next_up()), 1.0);
    }
}
#[test]
fn fisher_yates_reaches_each_three_element_permutation_once() {
    let mut outcomes = std::collections::BTreeSet::new();
    for a in [1, 0x80000000, u32::MAX] {
        for b in [0, u32::MAX] {
            let mut r = Words32 {
                words: vec![a, b],
                pos: 0,
            };
            let mut data = [0, 1, 2];
            r.shuffle(&mut data).unwrap();
            assert!(outcomes.insert(data));
            let mut r = Words32 {
                words: vec![a, b],
                pos: 0,
            };
            assert_eq!(
                r.shuffled(&[0, 1, 2]).unwrap().copied().collect::<Vec<_>>(),
                data
            );
        }
    }
    assert_eq!(outcomes.len(), 6);
}
#[test]
fn twisted_gfsr_populates_high_bits_and_selects_all_choices() {
    let mut r = urng::TwistedGFSR::new(123);
    assert!((0..32).any(|_| r.nextu() >> 32 != 0));
    let mut found = [false; 4];
    for _ in 0..128 {
        found[*r.choice(&[0, 1, 2, 3])] = true;
    }
    assert_eq!(found, [true; 4]);
    let mut a = urng::TwistedGFSR::new(123);
    let mut b = urng::TwistedGFSR::new(123 + (1 << 32));
    assert_ne!(
        std::array::from_fn::<_, 16, _>(|_| a.nextu()),
        std::array::from_fn::<_, 16, _>(|_| b.nextu())
    );
}
#[test]
fn xorshift_seed_whose_mixer_result_is_zero_remains_live() {
    let mut r = urng::Xorshift32::new(0x61c88647);
    assert!((0..8).all(|_| r.nextu() != 0));
    let mut r = urng::Xorshift64::new(0x61c8864680b583eb);
    assert!((0..8).all(|_| r.nextu() != 0));
}

#[cfg(feature = "wide")]
#[test]
fn wide_xorshift_zero_seed_lanes_are_repaired() {
    use urng::wide::RngW;
    macro_rules! check {
        ($ty:ident) => {{
            let mut r = urng::wide::$ty::new(0x61c88647);
            assert!((0..8).all(|_| r.nextu().iter().all(|&x| x != 0)));
        }};
    };
    check!(Xorshift32x4);
    check!(Xorshift32x8);
    check!(Xorshift32x16);
}

#[cfg(all(feature = "simd", target_arch = "x86_64"))]
#[test]
fn intrinsic_xorshift_zero_seed_lane_is_repaired() {
    use urng::RngV;
    if !std::arch::is_x86_feature_detected!("avx2") {
        return;
    }
    let mut r = urng::Xorshift32x8::new(0x61c88647);
    for _ in 0..8 {
        let block: [u32; 8] = unsafe { std::mem::transmute(r.nextuv()) };
        assert!(block.iter().all(|&x| x != 0));
    }
}

#[cfg(all(feature = "simd", target_arch = "x86_64"))]
#[test]
#[allow(unused_unsafe)] // Constructors differ in whether they require unsafe.
fn intrinsic_array_ranged_draws_handle_full_width_and_half_open_extremes() {
    macro_rules! check {($ty:ident,$bits:tt)=>{pastey::paste! {{
        let mut raw=unsafe{urng::$ty::new(123)};
        let mut bounded=unsafe{urng::$ty::new(123)};
        for _ in 0..32 {
            let expected=unsafe{raw.nextu()}.map(|x|x.to_randi([<i $bits>]::MIN,[<i $bits>]::MAX));
            assert_eq!(unsafe{bounded.randi([<i $bits>]::MIN,[<i $bits>]::MAX)},expected);
        }
        let min=1.0 as [<f $bits>];let max=min.next_up();
        for _ in 0..32 {assert!(unsafe{bounded.randf(min,max)}.iter().all(|&x|x==min));}
        for _ in 0..32 {assert!(unsafe{bounded.randf(-[<f $bits>]::MAX,[<f $bits>]::MAX)}.iter().all(|x|x.is_finite()&&*x<[<f $bits>]::MAX));}
    }}}};
    if std::arch::is_x86_feature_detected!("avx512f") {
        check!(Philox32x4x4, 32);
        check!(Sfc64x8, 64);
        check!(Biski64x8, 64);
        if std::arch::is_x86_feature_detected!("avx512dq") {
            check!(Cet64x8, 64);
            check!(Cet256x2, 64);
        }
    }
}

#[cfg(all(feature = "simd", target_arch = "x86_64"))]
#[test]
fn rngv_rejects_low_products_in_every_lane_without_isa_requirements() {
    use std::arch::x86_64::*;
    use urng::RngV;
    macro_rules! check {
        ($v:ty,$n:expr) => {{
            struct Source {
                draws: usize,
            }
            impl RngV for Source {
                type Word = $v;
                fn nextuv(&mut self) -> $v {
                    let block = if self.draws == 0 {
                        [0u32; $n]
                    } else {
                        [u32::MAX; $n]
                    };
                    self.draws += 1;
                    unsafe { std::mem::transmute(block) }
                }
            }
            let mut r = Source { draws: 0 };
            let widths: $v = unsafe { std::mem::transmute([3u32; $n]) };
            let minima: $v = unsafe { std::mem::transmute([0i32; $n]) };
            let got: [i32; $n] = unsafe { std::mem::transmute(r.randiv(widths, minima)) };
            assert_eq!(got, [2; $n]);
            assert_eq!(r.draws, 1 + $n);
            let mut r = Source { draws: 0 };
            let full: $v = unsafe { std::mem::transmute([0u32; $n]) };
            let min: $v = unsafe { std::mem::transmute([i32::MIN; $n]) };
            let got: [i32; $n] = unsafe { std::mem::transmute(r.randiv(full, min)) };
            assert_eq!(got, [i32::MIN; $n]);
            assert_eq!(r.draws, 1);
        }};
    };
    check!(__m128i, 4);
    check!(__m256i, 8);
    check!(__m512i, 16);
}

#[cfg(target_pointer_width = "64")]
#[test]
fn u32_rng_can_address_slices_longer_than_its_word_domain() {
    let mut r = Words32 {
        words: vec![u32::MAX, u32::MAX],
        pos: 0,
    };
    assert!(r.index((1usize << 33) + 1) >= 1usize << 32);
    assert_eq!(r.pos, 2);
    assert!(u32::MAX.to_index(usize::MAX) < usize::MAX);
}

#[cfg(all(feature = "simd", target_arch = "x86_64"))]
#[test]
fn vector_conversions_match_each_scalar_lane_including_odd_and_full_ranges() {
    use urng::WordV;
    macro_rules! check {
        ($vector:ty,$float:ty,$n:expr) => {{
            let words: [u32; $n] =
                std::array::from_fn(|i| [1, u32::MAX, 0x12345678, 0x80000000][i % 4]);
            let scale: [u32; $n] = std::array::from_fn(|i| [3, 7, 0, 100][i % 4]);
            let min: [i32; $n] =
                std::array::from_fn(|i| if scale[i] == 0 { i32::MIN } else { -100 });
            let v: $vector = unsafe { std::mem::transmute(words) };
            let s: $vector = unsafe { std::mem::transmute(scale) };
            let m: $vector = unsafe { std::mem::transmute(min) };
            let out: [i32; $n] = unsafe { std::mem::transmute(v.to_randi(s, m)) };
            for i in 0..$n {
                let max = if scale[i] == 0 {
                    i32::MAX
                } else {
                    min[i] + scale[i] as i32 - 1
                };
                assert_eq!(out[i], words[i].to_randi(min[i], max));
            }
            let floats: [f32; $n] = unsafe { std::mem::transmute::<$float, [f32; $n]>(v.to_f01()) };
            assert_eq!(floats, words.map(Word::to_f01));
        }};
    };
    use std::arch::x86_64::*;
    check!(__m128i, __m128, 4);
    check!(__m256i, __m256, 8);
    check!(__m512i, __m512, 16);
}

#[cfg(feature = "wide")]
#[test]
fn wide_splitmix_is_the_scalar_stream_without_overlapping_blocks() {
    use urng::wide::RngW;
    macro_rules! check {
        ($ty:ident) => {{
            for seed in [0, 1, 2, u32::MAX] {
                let mut wide = urng::wide::$ty::new(seed);
                let mut scalar = urng::SplitMix32::new(seed);
                for _ in 0..128 {
                    for actual in wide.nextu() {
                        assert_eq!(actual, scalar.nextu());
                    }
                }
            }
        }};
    };
    check!(SplitMix32x4);
    check!(SplitMix32x8);
    check!(SplitMix32x16);
}
