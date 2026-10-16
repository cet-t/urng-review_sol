use wrapn::{wrap, wu64};

use crate::prng::b64::SplitMix64;
use crate::rng::Rng;

/// Xoshiro256++ 64-bit RNG implementation.
///
/// # Example
/// ```
/// use urng::{Rng, Xoshiro256Pp};
///
/// let mut rng = Xoshiro256Pp::new(1);
/// let _ = rng.nextu();
/// ```
#[repr(C, align(64))]
#[derive(Debug, Clone, Copy)]
pub struct Xoshiro256Pp {
    pub(crate) s: [wu64; 4],
}

impl Xoshiro256Pp {
    /// Creates a new `Xoshiro256Pp` instance with the given seed.
    pub const fn new(seed: u64) -> Self {
        let mut seedgen = SplitMix64::new(seed);
        Self {
            s: wrap![
                seedgen.nextu_const(),
                seedgen.nextu_const(),
                seedgen.nextu_const(),
                seedgen.nextu_const(),
            ],
        }
    }
}

impl Rng for Xoshiro256Pp {
    type Word = u64;

    #[inline]
    fn nextu(&mut self) -> Self::Word {
        let s = &mut self.s;
        let res = s[0] + (s[0] + s[3]).rotate_left(23);
        let t = s[1] << 17;

        s[2] ^= s[0];
        s[3] ^= s[1];
        s[1] ^= s[2];
        s[0] ^= s[3];
        s[2] ^= t;
        s[3] = s[3].rotate_left(45);

        *res
    }
}

/// Xoshiro256** 64-bit RNG implementation.
///
/// # Example
/// ```
/// use urng::{Rng, Xoshiro256Ss};
///
/// let mut rng = Xoshiro256Ss::new(1);
/// let _ = rng.nextu();
/// ```
#[repr(C, align(64))]
#[derive(Debug, Clone, Copy)]
pub struct Xoshiro256Ss {
    pub(crate) s: [wu64; 4],
}

impl Xoshiro256Ss {
    /// Creates a new `Xoshiro256Ss` instance with the given seed.
    pub const fn new(seed: u64) -> Self {
        let mut seedgen = SplitMix64::new(seed);
        Self {
            s: wrap![
                seedgen.nextu_const(),
                seedgen.nextu_const(),
                seedgen.nextu_const(),
                seedgen.nextu_const(),
            ],
        }
    }
}

impl Rng for Xoshiro256Ss {
    type Word = u64;

    #[inline]
    fn nextu(&mut self) -> Self::Word {
        let s = &mut self.s;
        let res = (s[1] * 5).rotate_left(7) * 9;
        let t = s[1] << 17;

        s[2] ^= s[0];
        s[3] ^= s[1];
        s[1] ^= s[2];
        s[0] ^= s[3];
        s[2] ^= t;
        s[3] = s[3].rotate_left(45);

        *res
    }
}

#[cfg(all(feature = "simd", target_arch = "x86_64"))]
pub use simd::*;

#[cfg(all(feature = "simd", target_arch = "x86_64"))]
pub mod simd {
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;

    use crate::prng::b64::SplitMix64;

    /// Two xoshiro256** streams in packed SIMD storage.
    ///
    /// Uses a portable scalar step for this packed layout. No AVX-512 CPU
    /// support is needed for these safe methods.
    #[cfg(target_arch = "x86_64")]
    #[repr(C, align(64))]
    pub struct Xoshiro256Ssx2 {
        pub(crate) s: __m512i,
    }

    impl Xoshiro256Ssx2 {
        /// Creates a new `Xoshiro256Ssx2` instance.
        #[cfg(target_arch = "x86_64")]
        pub fn new(seed: u64) -> Self {
            let mut seedgen = SplitMix64::new(seed);
            let s = [0u64; 8].map(|_| seedgen.nextu_const());

            Self {
                s: unsafe { std::mem::transmute::<[u64; 8], __m512i>(s) },
            }
        }

        /// Generates the next 2 random `u64` values.
        #[cfg(target_arch = "x86_64")]
        #[inline(always)]
        pub fn nextu(&mut self) -> [u64; 2] {
            // Array conversion copies bits and imposes no SIMD target features.
            let mut state: [u64; 8] = unsafe { std::mem::transmute(self.s) };
            let out = std::array::from_fn(|lane| {
                let s = &mut state[lane * 4..lane * 4 + 4];
                let out = s[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);
                let t = s[1] << 17;
                s[2] ^= s[0];
                s[3] ^= s[1];
                s[1] ^= s[2];
                s[0] ^= s[3];
                s[2] ^= t;
                s[3] = s[3].rotate_left(45);
                out
            });
            self.s = unsafe { std::mem::transmute::<[u64; 8], __m512i>(state) };
            out
        }
    }

    #[cfg(test)]
    mod regression {
        use super::*;
        use crate::Rng;

        #[test]
        fn packed_ss_has_two_correct_lanes_without_wide_stores() {
            // Independent author's-state known answer: state [1,2,3,4] => 11520.
            // https://prng.di.unimi.it/xoshiro256starstar.c
            let mut packed = Xoshiro256Ssx2 {
                s: unsafe { std::mem::transmute([1u64, 2, 3, 4, 1, 2, 3, 4]) },
            };
            assert_eq!(packed.nextu(), [11520, 11520]);
            for seed in [0, 1, 2, 123, u64::MAX] {
                let mut sm = SplitMix64::new(seed);
                let words = std::array::from_fn::<_, 8, _>(|_| super::super::wrap!(sm.nextu()));
                let mut expected = [
                    super::super::Xoshiro256Ss {
                        s: words[..4].try_into().unwrap(),
                    },
                    super::super::Xoshiro256Ss {
                        s: words[4..].try_into().unwrap(),
                    },
                ];
                let mut packed = Xoshiro256Ssx2::new(seed);
                for _ in 0..256 {
                    assert_eq!(packed.nextu(), std::array::from_fn(|i| expected[i].nextu()));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    crate::safe_test! {
        Xoshiro256Pp,
        Xoshiro256Ss
    }
}
