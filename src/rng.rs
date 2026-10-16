//! A 32/64-bit random number generator trait.

use crate::_internal::{i2f_bits, randi_wide, u2f_01};

mod sealed {
    /// Prevents `Word` from being implemented outside this crate, so the
    /// `u32 -> f32/i32` / `u64 -> f64/i64` width relationships stay closed.
    pub trait Sealed {}
    impl Sealed for u32 {}
    impl Sealed for u64 {}
}

/// The unsigned output word of a generator (`u32` or `u64`).
///
/// A `Word` fixes the whole numeric family of an RNG — its float type, its
/// signed-range integer type, and the bit width — and centralizes the
/// uniform-conversion formulas in one place so every [`Rng`] implementor
/// inherits them for free.
pub trait Word: sealed::Sealed + Copy {
    /// The float type produced from this word (`f32` for `u32`, `f64` for `u64`).
    type Float: Copy;
    /// The signed integer type used for ranged draws (`i32` / `i64`).
    type Int: Copy;

    /// Maps the word uniformly onto `[0, 1)`.
    #[must_use]
    fn to_f01(self) -> Self::Float;

    /// Scales one word into `[min, max]`. This fixed-cost mapping is biased
    /// unless the range width divides the word domain. Use [`Rng::randi`] for
    /// rejection-sampled uniform integers. Panics if `min > max`.
    #[must_use]
    fn to_randi(self, min: Self::Int, max: Self::Int) -> Self::Int;

    /// Maps the word uniformly onto the half-open float range `[min, max)`.
    #[must_use]
    fn to_randf(self, min: Self::Float, max: Self::Float) -> Self::Float;

    /// Scales one word into `[0, len)`, with the same fixed-cost bias as
    /// [`Word::to_randi`]. Panics on an empty range. Use [`Rng::index`] for
    /// rejection-sampled uniform indices, including slices longer than 2^32.
    #[must_use]
    fn to_index(self, len: usize) -> usize;

    #[doc(hidden)]
    fn sample_randi<F: FnMut() -> Self>(next: &mut F, min: Self::Int, max: Self::Int) -> Self::Int;

    #[doc(hidden)]
    fn sample_index<F: FnMut() -> Self>(next: &mut F, len: usize) -> usize;
}

macro_rules! impl_word {
    ($bits:tt) => {
        pastey::paste! {
            impl Word for [<u $bits>] {
                type Float = [<f $bits>];
                type Int = [<i $bits>];

                #[inline(always)]
                fn to_f01(self) -> Self::Float {
                    u2f_01!([<f $bits>], $bits, self)
                }

                #[inline(always)]
                fn to_randi(self, min: Self::Int, max: Self::Int) -> Self::Int {
                    assert!(min <= max, "empty integer range");
                    let range = (max as randi_wide!(i $bits) - min as randi_wide!(i $bits) + 1)
                        as randi_wide!(u $bits);
                    (((self as randi_wide!(u $bits) * range) >> $bits) as randi_wide!(i $bits)
                        + min as randi_wide!(i $bits)) as [<i $bits>]
                }

                #[inline(always)]
                fn to_randf(self, min: Self::Float, max: Self::Float) -> Self::Float {
                    assert!(min.is_finite() && max.is_finite() && min < max, "invalid float range");
                    let u = self.to_f01();
                    let width = max - min;
                    let value = if width.is_finite() {
                        u * width + min
                    } else {
                        min * (1.0 - u) + max * u
                    };
                    // Even exact u < 1 can round an affine transform to max.
                    if value >= max { max.next_down() } else { value.max(min) }
                }

                #[inline(always)]
                fn to_index(self, len: usize) -> usize {
                    assert!(len != 0, "empty index range");
                    ((self as u128 * len as u128) >> $bits) as usize
                }

                #[inline]
                fn sample_randi<F: FnMut() -> Self>(next: &mut F, min: Self::Int, max: Self::Int) -> Self::Int {
                    assert!(min <= max, "empty integer range");
                    let width = (max as randi_wide!(i $bits) - min as randi_wide!(i $bits) + 1)
                        as randi_wide!(u $bits);
                    if width == (1 as randi_wide!(u $bits)) << $bits {
                        return (next() as randi_wide!(i $bits) + min as randi_wide!(i $bits)) as Self::Int;
                    }
                    let range = width as Self;
                    let threshold = range.wrapping_neg() % range;
                    loop {
                        let product = next() as randi_wide!(u $bits) * width;
                        if product as Self >= threshold {
                            return ((product >> $bits) as randi_wide!(i $bits)
                                + min as randi_wide!(i $bits)) as Self::Int;
                        }
                    }
                }

                #[inline]
                fn sample_index<F: FnMut() -> Self>(next: &mut F, len: usize) -> usize {
                    assert!(len != 0, "empty index range");
                    // A 32-bit RNG needs two draws to cover a 64-bit length.
                    let range = len as u64;
                    let threshold = range.wrapping_neg() % range;
                    loop {
                        let word = if $bits == 32 && len as u128 > u32::MAX as u128 {
                            ((next() as u64) << 32) | next() as u64
                        } else if $bits == 32 {
                            let r = len as u32;
                            let threshold32 = r.wrapping_neg() % r;
                            loop {
                                let product = next() as u64 * r as u64;
                                if product as u32 >= threshold32 { return (product >> 32) as usize; }
                            }
                        } else { next() as u64 };
                        let product = word as u128 * range as u128;
                        if product as u64 >= threshold { return (product >> 64) as usize; }
                    }
                }
            }
        }
    };
}

impl_word!(32);
impl_word!(64);

/// A random number generator characterized by a single output [`Word`].
///
/// Implementors provide only [`nextu`](Rng::nextu); the ranged and float
/// draws are supplied as defaults derived from the associated [`Word`], so the
/// numeric formulas live in exactly one place.
pub trait Rng {
    /// The unsigned output word this generator produces.
    type Word: Word;

    /// Generates the next raw word in `[0, 2^BITS)`.
    #[must_use]
    fn nextu(&mut self) -> Self::Word;

    /// Generates the next float in `[0, 1)`.
    #[must_use]
    #[inline(always)]
    fn nextf(&mut self) -> <Self::Word as Word>::Float {
        self.nextu().to_f01()
    }

    /// Generates an integer uniformly in the inclusive range `[min, max]`.
    #[must_use]
    #[inline(always)]
    fn randi(
        &mut self,
        min: <Self::Word as Word>::Int,
        max: <Self::Word as Word>::Int,
    ) -> <Self::Word as Word>::Int {
        Self::Word::sample_randi(&mut || self.nextu(), min, max)
    }

    /// Generates a float uniformly in the half-open range `[min, max)`.
    #[must_use]
    #[inline(always)]
    fn randf(
        &mut self,
        min: <Self::Word as Word>::Float,
        max: <Self::Word as Word>::Float,
    ) -> <Self::Word as Word>::Float {
        self.nextu().to_randf(min, max)
    }

    /// Generates a uniform index in `[0, len)`. Panics if `len == 0`.
    #[inline]
    fn index(&mut self, len: usize) -> usize {
        Self::Word::sample_index(&mut || self.nextu(), len)
    }
}

/// Constructs a generator from a raw seed of its native word width.
pub trait Seed {
    /// The seed type (`u32` or `u64`, matching the generator's word width).
    type Seed;

    /// Builds `Self` from a seed.
    #[must_use]
    fn from_seed(seed: Self::Seed) -> Self;
}
