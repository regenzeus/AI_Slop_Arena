//! Deterministic random numbers.
//!
//! The generator is implemented here (PCG32) rather than taken from a crate so
//! that a seed means the same thing forever, regardless of dependency updates.

use crate::math;

#[derive(Clone, Debug)]
pub struct Rng {
    state: u64,
    inc: u64,
}

impl Rng {
    /// Creates a generator. Different `stream` values give independent
    /// sequences for the same seed, so each system can own its own stream.
    pub fn new(seed: u64, stream: u64) -> Self {
        let mut rng = Rng { state: 0, inc: (stream << 1) | 1 };
        rng.next_u32();
        rng.state = rng.state.wrapping_add(seed);
        rng.next_u32();
        rng
    }

    pub fn next_u32(&mut self) -> u32 {
        let old = self.state;
        self.state = old
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(self.inc);
        let xorshifted = (((old >> 18) ^ old) >> 27) as u32;
        let rot = (old >> 59) as u32;
        xorshifted.rotate_right(rot)
    }

    /// Uniform in [0, 1).
    pub fn f32(&mut self) -> f32 {
        (self.next_u32() >> 8) as f32 * (1.0 / (1u32 << 24) as f32)
    }

    /// Uniform in [lo, hi).
    pub fn range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + (hi - lo) * self.f32()
    }

    /// Uniform integer in [0, n).
    pub fn below(&mut self, n: u32) -> u32 {
        ((self.next_u32() as u64 * n as u64) >> 32) as u32
    }

    pub fn chance(&mut self, p: f32) -> bool {
        self.f32() < p
    }

    /// Standard normal sample (Box-Muller).
    pub fn normal(&mut self) -> f32 {
        let u1 = self.f32().max(1e-7);
        let u2 = self.f32();
        math::sqrt(-2.0 * math::ln(u1)) * math::cos(std::f32::consts::TAU * u2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_sequence() {
        let mut a = Rng::new(42, 1);
        let mut b = Rng::new(42, 1);
        for _ in 0..1000 {
            assert_eq!(a.next_u32(), b.next_u32());
        }
    }

    #[test]
    fn streams_differ() {
        let mut a = Rng::new(42, 1);
        let mut b = Rng::new(42, 2);
        let same = (0..100).filter(|_| a.next_u32() == b.next_u32()).count();
        assert!(same < 3);
    }

    #[test]
    fn normal_has_unit_variance() {
        let mut rng = Rng::new(7, 0);
        let n = 20_000;
        let samples: Vec<f32> = (0..n).map(|_| rng.normal()).collect();
        let mean = samples.iter().sum::<f32>() / n as f32;
        let var = samples.iter().map(|s| (s - mean) * (s - mean)).sum::<f32>() / n as f32;
        assert!(mean.abs() < 0.03, "mean {mean}");
        assert!((var - 1.0).abs() < 0.05, "variance {var}");
    }
}
