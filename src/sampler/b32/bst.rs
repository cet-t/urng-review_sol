use crate::rng::Rng;
use crate::sampler::Sampler;

/// Weighted sampler using cumulative sums and binary search (O(log n) sample, O(n) build).
///
/// # Examples
///
/// ```
/// use urng::Sampler;
/// use urng::Bst32;
/// use urng::Mt19937;
///
/// let mut rng = Mt19937::new(1);
/// let mut sampler = Bst32::new(&mut rng, &[1.0f32, 9.0]);
/// assert!(sampler.sample() < 2);
/// ```
#[derive(Debug)]
pub struct Bst32<'a, R: Rng<Word = u32> + 'a> {
    rng: &'a mut R,
    cumulative: Vec<f32>,
}

impl<'a, R: Rng<Word = u32> + 'a> Bst32<'a, R> {
    /// Creates a sampler from finite, nonnegative weights with positive total.
    /// Panics for an empty slice, negative/nonfinite weights, or all-zero weights.
    pub fn new(rng: &'a mut R, weights: &[f32]) -> Self {
        Self {
            rng,
            cumulative: Self::build_cumulative(weights),
        }
    }

    fn build_cumulative(weights: &[f32]) -> Vec<f32> {
        assert!(
            !weights.is_empty() && weights.iter().all(|w| w.is_finite() && *w >= 0.0),
            "invalid weights"
        );
        let maximum = weights.iter().copied().fold(0.0f32, f32::max);
        assert!(maximum > 0.0, "all weights are zero");
        let mut cumulative = Vec::with_capacity(weights.len());
        let mut sum = 0.0f32;
        for &w in weights {
            sum += w / maximum;
            cumulative.push(sum);
        }
        cumulative
    }
}

impl<'a, R: Rng<Word = u32> + 'a> Sampler<'a, R> for Bst32<'a, R> {
    fn weights(&mut self, weights: &[f32]) {
        self.cumulative = Self::build_cumulative(weights);
    }

    fn sample(&mut self) -> usize {
        let total = *self.cumulative.last().unwrap_or(&0.0);
        let r = self.rng.randf(0.0, total);
        // A boundary belongs to the following nonempty interval. In
        // particular, leading zero-weight entries must never be selected.
        self.cumulative.partition_point(|&c| c <= r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prng::b32::Mt19937;
    use crate::sampler::Sampler;

    #[test]
    fn bst32_works() {
        let mut rng = Mt19937::new(1);
        let mut sampler = Bst32::new(&mut rng, &[1.0f32, 9.0f32]);
        let ones = (0..100).filter(|_| sampler.sample() == 1).count();
        assert_eq!(ones, 87);
    }
}
