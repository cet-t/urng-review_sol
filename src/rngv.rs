//! Safe lane conversions: array operations need no AVX target features.
use crate::Word;
use std::arch::x86_64::*;
mod sealed {
    use std::arch::x86_64::*;
    pub trait Sealed {}
    impl Sealed for __m128i {}
    impl Sealed for __m256i {}
    impl Sealed for __m512i {}
}
pub trait WordV: sealed::Sealed + Copy + Sized {
    type Int;
    type Float;
    fn to_f01(self) -> Self::Float;
    /// One-word multiply/shift scaling per lane (biased for non-power-of-two
    /// ranges). A zero scale encodes the full 2^32-word range.
    fn to_randi(self, scale: Self::Int, min: Self::Int) -> Self::Int;
    /// Affine lane transform; scale is a width, not an upper bound.
    fn to_randf(self, scale: Self::Float, min: Self::Float) -> Self::Float;
    #[doc(hidden)]
    fn sample_randi(next: &mut impl FnMut() -> Self, scale: Self::Int, min: Self::Int)
    -> Self::Int;
}
macro_rules! lanes {
    ($vector:ty, $float:ty, $n:expr) => {
        impl WordV for $vector {
            type Int = Self;
            type Float = $float;
            fn to_f01(self) -> Self::Float {
                let words: [u32; $n] = unsafe { std::mem::transmute(self) };
                unsafe { std::mem::transmute(words.map(Word::to_f01)) }
            }
            fn to_randi(self, scale: Self, min: Self) -> Self {
                let x: [u32; $n] = unsafe { std::mem::transmute(self) };
                let scale: [u32; $n] = unsafe { std::mem::transmute(scale) };
                let min: [i32; $n] = unsafe { std::mem::transmute(min) };
                let out: [i32; $n] = std::array::from_fn(|i| {
                    let offset = if scale[i] == 0 {
                        x[i]
                    } else {
                        ((x[i] as u64 * scale[i] as u64) >> 32) as u32
                    };
                    (offset as i32).wrapping_add(min[i])
                });
                unsafe { std::mem::transmute(out) }
            }
            fn to_randf(self, scale: Self::Float, min: Self::Float) -> Self::Float {
                let x: [u32; $n] = unsafe { std::mem::transmute(self) };
                let scale: [f32; $n] = unsafe { std::mem::transmute(scale) };
                let min: [f32; $n] = unsafe { std::mem::transmute(min) };
                let out: [f32; $n] =
                    std::array::from_fn(|i| x[i].to_randf(min[i], min[i] + scale[i]));
                unsafe { std::mem::transmute(out) }
            }
            fn sample_randi(next: &mut impl FnMut() -> Self, scale: Self, min: Self) -> Self {
                let first: [u32; $n] = unsafe { std::mem::transmute(next()) };
                let widths: [u32; $n] = unsafe { std::mem::transmute(scale) };
                let minima: [i32; $n] = unsafe { std::mem::transmute(min) };
                let out: [i32; $n] = std::array::from_fn(|lane| {
                    let width = widths[lane];
                    let mut x = first[lane];
                    let offset = if width == 0 {
                        x
                    } else {
                        let threshold = width.wrapping_neg() % width;
                        loop {
                            let product = x as u64 * width as u64;
                            if product as u32 >= threshold {
                                break (product >> 32) as u32;
                            }
                            let block: [u32; $n] = unsafe { std::mem::transmute(next()) };
                            x = block[lane];
                        }
                    };
                    (offset as i32).wrapping_add(minima[lane])
                });
                unsafe { std::mem::transmute(out) }
            }
        }
    };
}
lanes!(__m128i, __m128, 4);
lanes!(__m256i, __m256, 8);
lanes!(__m512i, __m512, 16);

pub trait RngV {
    type Word: WordV;

    fn nextuv(&mut self) -> Self::Word;

    fn nextfv(&mut self) -> <Self::Word as WordV>::Float {
        self.nextuv().to_f01()
    }

    /// Uniform bounded integer draws per lane. `scale` is an unsigned width;
    /// zero encodes 2^32. Results are added to `min` with wrapping arithmetic.
    /// Rejections consume additional blocks, discarding their other lanes.
    fn randiv(
        &mut self,
        scale: <Self::Word as WordV>::Int,
        min: <Self::Word as WordV>::Int,
    ) -> <Self::Word as WordV>::Int {
        Self::Word::sample_randi(&mut || self.nextuv(), scale, min)
    }

    fn randfv(
        &mut self,
        scale: <Self::Word as WordV>::Float,
        min: <Self::Word as WordV>::Float,
    ) -> <Self::Word as WordV>::Float {
        self.nextuv().to_randf(scale, min)
    }
}
