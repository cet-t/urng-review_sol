use super::*;
#[cfg(all(feature = "simd", target_arch = "x86_64"))]
#[test]
fn simd_splitmix16_is_the_scalar_stream() {
    if !std::arch::is_x86_feature_detected!("avx512f") {
        return;
    }
    for seed in [0, 1, 2, u32::MAX] {
        let mut vector = unsafe { SplitMix32x16::new(seed) };
        let mut scalar = SplitMix32::new(seed);
        for _ in 0..128 {
            let actual: [u32; 16] = unsafe { std::mem::transmute(vector.nextuv()) };
            assert_eq!(actual, std::array::from_fn(|_| scalar.nextu()));
        }
    }
}

#[cfg(all(feature = "simd", target_arch = "x86_64"))]
#[test]
fn simd_philox4_blocks_match_scalar_and_carry_counters() {
    if !std::arch::is_x86_feature_detected!("avx512f") {
        return;
    }
    let counters: [u32; 16] =
        std::array::from_fn(|i| if i % 4 < 3 { u32::MAX } else { (i / 4) as u32 });
    let keys: [u32; 16] = std::array::from_fn(|i| if i % 2 == 0 { w32(i, 0) } else { 0 });
    let mut vector = Philox32x4x4 {
        c: unsafe { std::mem::transmute(counters) },
        k: unsafe { std::mem::transmute(keys) },
    };
    let mut scalar: [Philox32; 4] = std::array::from_fn(|i| Philox32 {
        c: counters[i * 4..i * 4 + 4]
            .try_into()
            .map(|a: [u32; 4]| a.map(|x| wrap!(x)))
            .unwrap(),
        k: wrap![keys[i * 4], keys[i * 4 + 2]],
        buf: wrap![0;4],
        pos: wrap!(4),
    });
    for _ in 0..256 {
        let actual = unsafe { vector.nextu() };
        let expected: [u32; 16] = scalar
            .iter_mut()
            .flat_map(|r| r.next_raw().map(|x| *x))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        assert_eq!(actual, expected);
    }
}

#[cfg(all(feature = "simd", target_arch = "x86_64"))]
#[test]
fn simd_cet256_two_lanes_match_scalar_through_carries() {
    if !std::arch::is_x86_feature_detected!("avx512dq") {
        return;
    }
    let states = [u64::MAX, u64::MAX, u64::MAX, u64::MAX, 0, 1, 2, 3];
    let mut vector = Cet256x2 {
        s: unsafe { std::mem::transmute(states) },
    };
    let mut scalar = [
        Cet256 {
            s: states[..4]
                .try_into()
                .map(|a: [u64; 4]| a.map(|x| wrap!(x)))
                .unwrap(),
        },
        Cet256 {
            s: states[4..]
                .try_into()
                .map(|a: [u64; 4]| a.map(|x| wrap!(x)))
                .unwrap(),
        },
    ];
    for _ in 0..256 {
        assert_eq!(
            unsafe { vector.nextu() },
            std::array::from_fn(|i| scalar[i].nextu())
        );
    }
}
