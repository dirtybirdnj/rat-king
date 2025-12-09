//! Shared random number generation utilities.
//!
//! Provides deterministic pseudo-random number generators for patterns
//! that need reproducible randomness.

/// A fast, deterministic pseudo-random number generator.
///
/// Uses a Linear Congruential Generator (LCG) with parameters from
/// Numerical Recipes for good statistical properties while being
/// extremely fast.
///
/// # Example
/// ```
/// use rat_king::rng::Rng;
///
/// let mut rng = Rng::new(12345);
/// let value = rng.next_f64(); // Returns value in [0, 1)
/// ```
#[derive(Clone)]
pub struct Rng {
    state: u64,
}

impl Rng {
    /// Create a new RNG with the given seed.
    ///
    /// The same seed will always produce the same sequence of numbers.
    #[inline]
    pub fn new(seed: u64) -> Self {
        Self { state: seed.wrapping_add(1) }
    }

    /// Get the next raw u64 value.
    #[inline]
    pub fn next_u64(&mut self) -> u64 {
        // LCG parameters from Numerical Recipes
        self.state = self.state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.state
    }

    /// Get a random f64 in the range [0, 1).
    #[inline]
    pub fn next_f64(&mut self) -> f64 {
        // Use high bits for better distribution
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Get a random f64 in the range [-1, 1).
    #[inline]
    pub fn next_signed(&mut self) -> f64 {
        self.next_f64() * 2.0 - 1.0
    }

    /// Get a random f64 in the range [min, max).
    #[inline]
    pub fn next_range(&mut self, min: f64, max: f64) -> f64 {
        min + self.next_f64() * (max - min)
    }

    /// Get a random boolean with the given probability of being true.
    #[inline]
    pub fn next_bool(&mut self, probability: f64) -> bool {
        self.next_f64() < probability
    }

    /// Get a random index in the range [0, len).
    #[inline]
    pub fn next_index(&mut self, len: usize) -> usize {
        (self.next_f64() * len as f64) as usize
    }
}

impl Default for Rng {
    fn default() -> Self {
        Self::new(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic() {
        let mut rng1 = Rng::new(42);
        let mut rng2 = Rng::new(42);

        for _ in 0..100 {
            assert_eq!(rng1.next_u64(), rng2.next_u64());
        }
    }

    #[test]
    fn different_seeds_differ() {
        let mut rng1 = Rng::new(1);
        let mut rng2 = Rng::new(2);

        let vals1: Vec<_> = (0..10).map(|_| rng1.next_u64()).collect();
        let vals2: Vec<_> = (0..10).map(|_| rng2.next_u64()).collect();

        assert_ne!(vals1, vals2);
    }

    #[test]
    fn f64_in_range() {
        let mut rng = Rng::new(12345);
        for _ in 0..1000 {
            let v = rng.next_f64();
            assert!(v >= 0.0 && v < 1.0);
        }
    }

    #[test]
    fn signed_in_range() {
        let mut rng = Rng::new(12345);
        for _ in 0..1000 {
            let v = rng.next_signed();
            assert!(v >= -1.0 && v < 1.0);
        }
    }

    #[test]
    fn range_works() {
        let mut rng = Rng::new(12345);
        for _ in 0..1000 {
            let v = rng.next_range(10.0, 20.0);
            assert!(v >= 10.0 && v < 20.0);
        }
    }

    #[test]
    fn index_in_bounds() {
        let mut rng = Rng::new(12345);
        for _ in 0..1000 {
            let idx = rng.next_index(10);
            assert!(idx < 10);
        }
    }
}
