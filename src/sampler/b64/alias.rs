use crate::rng::Rng;
use crate::sampler::Sampler;

/// Weighted sampler using Walker's Alias Method (O(1) sample, O(n) build).
///
/// Preferred over [`Bst64`](super::Bst64) when sampling repeatedly from the same weight distribution.
///
/// # Examples
///
/// ```
/// use urng::Sampler;
/// use urng::Alias64;
/// use urng::Mt1993764;
///
/// let mut rng = Mt1993764::new(1);
/// let mut sampler = Alias64::new(&mut rng, &[1.0f64, 2.0, 4.0, 8.0]);
/// assert!(sampler.sample() < 4);
///
/// // Update weights
/// sampler.weights(&[1.0, 1.0, 1.0, 1.0]);
/// assert!(sampler.sample() < 4);
/// ```
#[derive(Debug)]
pub struct Alias64<'a, R: Rng<Word = u64> + 'a> {
    rng: &'a mut R,
    prob: Vec<f64>,
    alias: Vec<usize>,
}

impl<'a, R: Rng<Word = u64> + 'a> Alias64<'a, R> {
    /// Creates a sampler from finite, nonnegative weights with positive total.
    /// Panics for an empty slice, negative/nonfinite weights, or all-zero weights.
    pub fn new(rng: &'a mut R, weights: &[f64]) -> Self {
        let (prob, alias) = Self::build(weights);
        Self { rng, prob, alias }
    }

    fn build(weights: &[f64]) -> (Vec<f64>, Vec<usize>) {
        let n = weights.len();
        assert!(
            n != 0 && weights.iter().all(|w| w.is_finite() && *w >= 0.0),
            "invalid weights"
        );
        let maximum = weights.iter().copied().fold(0.0f64, f64::max);
        assert!(maximum > 0.0, "all weights are zero");
        // Normalize before summing. Dividing each weight by the maximum avoids
        // overflow for [MAX, MAX] and reciprocal overflow for tiny weights.
        let total: f64 = weights.iter().map(|w| *w / maximum).sum();
        let scale = n as f64 / total;

        let mut prob = vec![0.0f64; n];
        let mut alias = vec![0usize; n];
        let mut small = Vec::with_capacity(n);
        let mut large = Vec::with_capacity(n);
        let mut p: Vec<f64> = weights.iter().map(|&w| (w / maximum) * scale).collect();

        for (i, &pi) in p.iter().enumerate() {
            if pi < 1.0 {
                small.push(i);
            } else {
                large.push(i);
            }
        }

        while !small.is_empty() && !large.is_empty() {
            let s = small.pop().unwrap();
            let l = *large.last().unwrap();
            prob[s] = p[s];
            alias[s] = l;
            p[l] -= 1.0 - p[s];
            if p[l] < 1.0 {
                large.pop();
                small.push(l);
            }
        }
        for i in large {
            prob[i] = 1.0;
        }
        for i in small {
            prob[i] = 1.0;
        }

        (prob, alias)
    }
}

impl<'a, R: Rng<Word = u64> + 'a> Sampler<'a, R> for Alias64<'a, R> {
    fn weights(&mut self, weights: &[f64]) {
        let (prob, alias) = Self::build(weights);
        self.prob = prob;
        self.alias = alias;
    }

    fn sample(&mut self) -> usize {
        let n = self.prob.len();
        let i = self.rng.index(n);
        let u = self.rng.randf(0.0, 1.0);
        if u < self.prob[i] { i } else { self.alias[i] }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prng::b64::Mt1993764;
    use crate::sampler::Sampler;

    #[test]
    fn alias64_works() {
        let mut rng = Mt1993764::new(1);
        let mut sampler = Alias64::new(&mut rng, &[0.1, 1.0, 10.0, 100.0]);
        let ones = (0..100_000).filter(|_| sampler.sample() == 1).count();
        assert_eq!(ones, 867);
    }
}
