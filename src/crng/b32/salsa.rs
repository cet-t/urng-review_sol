use crate::{Rng, SplitMix32};
use wrapn::{wrap, wu32, wusize};

/// Standard Salsa block-function RNG. `ROUNDS` must be positive and even.
///
/// `new(u32)` is a deterministic convenience seed with only 32 input bits;
/// `Default` uses a noncryptographic time seed. For secret material, supply an
/// independently generated 256-bit key to `from_key_nonce` instead.
///
/// # Example
/// ```
/// use urng::{Rng, Salsa20};
///
/// let mut rng = Salsa20::new(12345);
/// let _ = rng.nextu();
/// ```
#[repr(C, align(64))]
#[derive(Debug, Clone, Copy)]
pub struct Salsa<const ROUNDS: usize> {
    x: [wu32; 16],
    pub(crate) buf: [wu32; 16],
    pub(crate) pos: wusize,
    exhausted: bool,
}
impl<const ROUNDS: usize> Salsa<ROUNDS> {
    /// Creates a new `Salsa` instance with the given seed.
    pub fn new(seed: u32) -> Self {
        let mut sm = SplitMix32::new(seed);
        let mut key = [0u8; 32];
        for chunk in key.chunks_exact_mut(4) {
            chunk.copy_from_slice(&sm.nextu().to_le_bytes());
        }
        Self::from_key_nonce(key, [0u8; 8], 0)
    }
    /// Constructs the standard key/nonce/counter state. Exhausting the counter
    /// panics before any counter/nonce pair could be reused.
    pub fn from_key_nonce(key: [u8; 32], nonce: [u8; 8], counter: u64) -> Self {
        assert!(
            ROUNDS > 0 && ROUNDS.is_multiple_of(2),
            "round count must be positive and even"
        );
        let mut x = [0u32; 16];
        for (i, constant) in [0x61707865, 0x3320646e, 0x79622d32, 0x6b206574]
            .into_iter()
            .enumerate()
        {
            x[5 * i] = constant;
        }
        for (i, bytes) in key.chunks_exact(4).enumerate() {
            x[if i < 4 { i + 1 } else { i + 7 }] = u32::from_le_bytes(bytes.try_into().unwrap());
        }
        x[6] = u32::from_le_bytes(nonce[..4].try_into().unwrap());
        x[7] = u32::from_le_bytes(nonce[4..].try_into().unwrap());
        x[8] = counter as u32;
        x[9] = (counter >> 32) as u32;
        Self {
            x: x.map(|w| wrap!(w)),
            buf: wrap![0u32;16],
            pos: wrap!(16),
            exhausted: false,
        }
    }
    fn qr(mut a: wu32, mut b: wu32, mut c: wu32, mut d: wu32) -> [wu32; 4] {
        b ^= (a + d).rotate_left(7);
        c ^= (b + a).rotate_left(9);
        d ^= (c + b).rotate_left(13);
        a ^= (d + c).rotate_left(18);
        [a, b, c, d]
    }
    fn next_raw(&mut self) -> [wu32; 16] {
        assert!(!self.exhausted, "Salsa counter exhausted");
        let original = self.x;
        let mut x = original;
        for _ in 0..ROUNDS / 2 {
            [x[0], x[4], x[8], x[12]] = Self::qr(x[0], x[4], x[8], x[12]);
            [x[5], x[9], x[13], x[1]] = Self::qr(x[5], x[9], x[13], x[1]);
            [x[10], x[14], x[2], x[6]] = Self::qr(x[10], x[14], x[2], x[6]);
            [x[15], x[3], x[7], x[11]] = Self::qr(x[15], x[3], x[7], x[11]);
            [x[0], x[1], x[2], x[3]] = Self::qr(x[0], x[1], x[2], x[3]);
            [x[5], x[6], x[7], x[4]] = Self::qr(x[5], x[6], x[7], x[4]);
            [x[10], x[11], x[8], x[9]] = Self::qr(x[10], x[11], x[8], x[9]);
            [x[15], x[12], x[13], x[14]] = Self::qr(x[15], x[12], x[13], x[14]);
        }
        if self.x[8] == u32::MAX && self.x[9] == u32::MAX {
            self.exhausted = true;
        } else {
            self.x[8] += 1;
            if self.x[8] == 0 {
                self.x[9] += 1;
            }
        }
        std::array::from_fn(|i| x[i] + original[i])
    }
}
impl<const ROUNDS: usize> Rng for Salsa<ROUNDS> {
    type Word = u32;
    fn nextu(&mut self) -> u32 {
        if self.pos >= 16 {
            self.buf = self.next_raw();
            self.pos = wrap!(0);
        }
        let v = self.buf[*self.pos];
        self.pos += 1;
        *v
    }
}
impl<const ROUNDS: usize> Default for Salsa<ROUNDS> {
    fn default() -> Self {
        Self::new(crate::_internal::default_seed32())
    }
}
impl<const ROUNDS: usize> crate::Seed for Salsa<ROUNDS> {
    type Seed = u32;
    fn from_seed(seed: u32) -> Self {
        Self::new(seed)
    }
}
pub type Salsa8 = Salsa<8>;
pub type Salsa20 = Salsa<20>;
#[cfg(test)]
mod tests {
    use super::*;
    crate::safe_test! { Salsa8, Salsa20 }
}
