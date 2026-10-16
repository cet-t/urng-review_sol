use wrapn::{wrap, wu32};

use crate::prng::b32::SplitMix32;
use crate::rng::Rng;

/// A xoroshiro64** random number generator.
///
/// A fast, high-quality 32-bit generator with a 64-bit state.
///
/// # Example
/// ```
/// use urng::{Rng, Xoroshiro64Ss};
///
/// let mut rng = Xoroshiro64Ss::new(12345);
/// let _ = rng.nextu();
/// ```
#[repr(C, align(64))]
#[derive(Debug, Clone, Copy)]
pub struct Xoroshiro64Ss {
    pub(crate) s: [wu32; 2],
}

impl Xoroshiro64Ss {
    /// Creates a new `Xoroshiro64Ss` instance with the given seed.
    pub const fn new(seed: u32) -> Self {
        let mut seedgen = SplitMix32::new(seed);

        Self {
            s: wrap![seedgen.nextu_const(), seedgen.nextu_const()],
        }
    }
}

impl Rng for Xoroshiro64Ss {
    type Word = u32;

    #[inline(always)]
    fn nextu(&mut self) -> Self::Word {
        let s0 = self.s[0];
        let mut s1 = self.s[1];
        let result = (s0 * 0x9E3779BB).rotate_left(5) * 5;

        s1 ^= s0;
        self.s[0] = s0.rotate_left(26) ^ s1 ^ (s1 << 9);
        self.s[1] = s1.rotate_left(13);

        *result
    }
}

#[cfg(all(feature = "simd", target_arch = "x86_64"))]
pub use simd::*;

#[cfg(all(feature = "simd", target_arch = "x86_64"))]
pub mod simd {
    use std::arch::x86_64::*;

    use crate::prng::b32::SplitMix32;
    use crate::rngv::RngV;

    pub(crate) const XOROSHIRO64SSX8: usize = 8;

    /// 8-way SIMD implementation of xoroshiro64** 32-bit RNG.
    /// This implementation uses AVX2 instructions to generate 8 random numbers in parallel.
    ///
    /// # Example
    /// ```
    /// use urng::{RngV, Xoroshiro64Ssx8};
    ///
    /// let mut rng = unsafe { Xoroshiro64Ssx8::new(12345) };
    /// let _ = rng.nextuv();
    /// ```
    #[cfg(target_arch = "x86_64")]
    #[repr(C, align(64))]
    pub struct Xoroshiro64Ssx8 {
        pub(crate) s0: __m256i,
        pub(crate) s1: __m256i,
    }

    #[allow(dead_code)]
    impl Xoroshiro64Ssx8 {
        /// # Safety
        /// This function requires AVX2 support. Ensure that the CPU supports it and that the code is compiled with the appropriate target features.
        #[target_feature(enable = "avx2")]
        pub fn new(seed: u32) -> Self {
            assert!(
                std::arch::is_x86_feature_detected!("avx2"),
                "Xoroshiro64Ssx8 requires avx2"
            );
            let mut seedgen = SplitMix32::new(seed);

            let mut s0 = [0u32; XOROSHIRO64SSX8];
            let mut s1 = [0u32; XOROSHIRO64SSX8];

            for i in 0..XOROSHIRO64SSX8 {
                s0[i] = seedgen.nextu_const();
                s1[i] = seedgen.nextu_const();
            }

            unsafe {
                Self {
                    s0: _mm256_loadu_si256(s0.as_ptr() as *const __m256i),
                    s1: _mm256_loadu_si256(s1.as_ptr() as *const __m256i),
                }
            }
        }
    }

    impl RngV for Xoroshiro64Ssx8 {
        type Word = __m256i;

        #[inline]
        fn nextuv(&mut self) -> Self::Word {
            assert!(
                std::arch::is_x86_feature_detected!("avx2"),
                "Xoroshiro64Ssx8 requires avx2"
            );
            let s0 = self.s0;
            let mut s1 = self.s1;

            unsafe {
                let mult = _mm256_set1_epi32(0x9E3779BBu32 as i32);
                let mixed = _mm256_mullo_epi32(s0, mult);
                let rotated =
                    _mm256_or_si256(_mm256_slli_epi32(mixed, 5), _mm256_srli_epi32(mixed, 27));
                let result = _mm256_add_epi32(rotated, _mm256_slli_epi32(rotated, 2));

                s1 = _mm256_xor_si256(s1, s0);
                self.s0 = _mm256_xor_si256(
                    _mm256_xor_si256(
                        _mm256_or_si256(_mm256_slli_epi32(s0, 26), _mm256_srli_epi32(s0, 6)),
                        s1,
                    ),
                    _mm256_slli_epi32(s1, 9),
                );
                self.s1 = _mm256_or_si256(_mm256_slli_epi32(s1, 13), _mm256_srli_epi32(s1, 19));

                result
            }
        }
    }

    pub(crate) const XOROSHIRO64SSX16: usize = 16;

    /// 16-way SIMD implementation of xoroshiro64** 32-bit RNG.
    /// This implementation uses AVX-512F instructions to generate 16 random numbers in parallel.
    ///
    /// # Example
    /// ```no_run
    /// use urng::{RngV, Xoroshiro64Ssx16};
    ///
    /// let mut rng = unsafe { Xoroshiro64Ssx16::new(12345) };
    /// let _ = rng.nextuv();
    /// ```
    #[cfg(target_arch = "x86_64")]
    #[repr(C, align(64))]
    pub struct Xoroshiro64Ssx16 {
        pub(crate) s0: __m512i,
        pub(crate) s1: __m512i,
    }

    #[allow(dead_code)]
    impl Xoroshiro64Ssx16 {
        /// # Safety
        /// This function requires AVX-512F support. Ensure that the CPU supports it and that the code is compiled with the appropriate target features.
        #[target_feature(enable = "avx512f")]
        pub fn new(seed: u32) -> Self {
            assert!(
                std::arch::is_x86_feature_detected!("avx512f"),
                "Xoroshiro64Ssx16 requires avx512f"
            );
            let mut seedgen = SplitMix32::new(seed);

            let mut s0 = [0u32; XOROSHIRO64SSX16];
            let mut s1 = [0u32; XOROSHIRO64SSX16];

            for i in 0..XOROSHIRO64SSX16 {
                s0[i] = seedgen.nextu_const();
                s1[i] = seedgen.nextu_const();
            }

            unsafe {
                Self {
                    s0: _mm512_loadu_si512(s0.as_ptr() as _),
                    s1: _mm512_loadu_si512(s1.as_ptr() as _),
                }
            }
        }
    }

    impl RngV for Xoroshiro64Ssx16 {
        type Word = __m512i;

        #[inline]
        fn nextuv(&mut self) -> Self::Word {
            assert!(
                std::arch::is_x86_feature_detected!("avx512f"),
                "Xoroshiro64Ssx16 requires avx512f"
            );
            let s0 = self.s0;
            let mut s1 = self.s1;

            unsafe {
                let mult = _mm512_set1_epi32(0x9E3779BBu32 as i32);
                let mixed = _mm512_mullo_epi32(s0, mult);
                let rotated =
                    _mm512_or_si512(_mm512_slli_epi32(mixed, 5), _mm512_srli_epi32(mixed, 27));
                let result = _mm512_add_epi32(rotated, _mm512_slli_epi32(rotated, 2));

                s1 = _mm512_xor_si512(s1, s0);
                self.s0 = _mm512_xor_si512(
                    _mm512_xor_si512(_mm512_rol_epi32(s0, 26), s1),
                    _mm512_slli_epi32(s1, 9),
                );
                self.s1 = _mm512_rol_epi32(s1, 13);

                result
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    crate::safe_test! { Xoroshiro64Ss }
    #[cfg(all(feature = "simd", target_feature = "avx512f"))]
    crate::unsafe_test! { Xoroshiro64Ssx16 }
}
