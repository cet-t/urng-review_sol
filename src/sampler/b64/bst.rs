use crate::rng::Rng;
use crate::sampler::Sampler;

/// Weighted sampler using cumulative sums and binary search (O(log n) sample, O(n) build).
///
/// # Examples
///
/// ```
/// use urng::Sampler;
/// use urng::Bst64;
/// use urng::Mt1993764;
///
/// let mut rng = Mt1993764::new(1);
/// let mut sampler = Bst64::new(&mut rng, &[1.0f64, 2.0, 4.0, 8.0]);
/// assert!(sampler.sample() < 4);
/// ```
#[derive(Debug)]
pub struct Bst64<'a, R: Rng<Word = u64> + 'a> {
    rng: &'a mut R,
    cumulative: Vec<f64>,
}

impl<'a, R: Rng<Word = u64> + 'a> Bst64<'a, R> {
    /// Creates a sampler from finite, nonnegative weights with positive total.
    /// Panics for an empty slice, negative/nonfinite weights, or all-zero weights.
    pub fn new(rng: &'a mut R, weights: &[f64]) -> Self {
        Self {
            rng,
            cumulative: Self::build_cumulative(weights),
        }
    }

    fn build_cumulative(weights: &[f64]) -> Vec<f64> {
        assert!(
            !weights.is_empty() && weights.iter().all(|w| w.is_finite() && *w >= 0.0),
            "invalid weights"
        );
        let maximum = weights.iter().copied().fold(0.0f64, f64::max);
        assert!(maximum > 0.0, "all weights are zero");
        let mut cumulative = Vec::with_capacity(weights.len());
        let mut sum = 0.0f64;
        for &w in weights {
            sum += w / maximum;
            cumulative.push(sum);
        }
        cumulative
    }
}

impl<'a, R: Rng<Word = u64> + 'a> Sampler<'a, R> for Bst64<'a, R> {
    fn weights(&mut self, weights: &[f64]) {
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
    use crate::prng::b64::Mt1993764;
    use crate::sampler::Sampler;

    #[test]
    fn bst64_works() {
        let mut rng = Mt1993764::new(1);
        let mut sampler = Bst64::new(&mut rng, &[0.1, 1.0, 10.0, 100.0]);
        let ones = (0..100_000).filter(|_| sampler.sample() == 1).count();
        assert_eq!(ones, 911);
    }
}
