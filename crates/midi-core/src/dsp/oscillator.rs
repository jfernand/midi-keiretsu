use core::f32::consts::PI;

/// Equivalent to `f32::fract`, which isn't available in `core`. Only
/// correct for non-negative `x` less than 2.0 -- true for every phase
/// value here, since phase is always advanced by less than 1.0 per
/// sample and wrapped back into `0.0..1.0` immediately.
pub(crate) fn fract(x: f32) -> f32 {
    x - libm::truncf(x)
}

/// Shared phase bookkeeping for the simple periodic oscillators below:
/// each one only needs to turn a `0.0..1.0` phase into a sample.
struct PhaseAccumulator {
    frequency: f32,
    sample_rate: f32,
    phase: f32,
}

impl PhaseAccumulator {
    fn new(sample_rate: f32) -> Self {
        Self {
            frequency: 0.0,
            sample_rate,
            phase: 0.0,
        }
    }

    fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency;
    }

    /// Returns the current phase and advances it for the next sample.
    fn advance(&mut self) -> f32 {
        let phase = self.phase;
        self.phase = fract(self.phase + self.frequency / self.sample_rate);
        phase
    }
}

pub struct SineOscillator(PhaseAccumulator);

impl SineOscillator {
    pub fn new(sample_rate: f32) -> Self {
        Self(PhaseAccumulator::new(sample_rate))
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        self.0.set_frequency(frequency);
    }

    pub fn next_sample(&mut self) -> f32 {
        let phase = self.0.advance();
        libm::sinf(2.0 * PI * phase)
    }
}

/// A bipolar square wave, high for the first half of each period and low
/// for the second.
pub struct SquareOscillator(PhaseAccumulator);

impl SquareOscillator {
    pub fn new(sample_rate: f32) -> Self {
        Self(PhaseAccumulator::new(sample_rate))
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        self.0.set_frequency(frequency);
    }

    pub fn next_sample(&mut self) -> f32 {
        let phase = self.0.advance();
        if phase < 0.5 { 1.0 } else { -1.0 }
    }
}

/// A rising sawtooth wave: ramps linearly from -1 to 1 across each period,
/// then jumps back down.
pub struct SawtoothOscillator(PhaseAccumulator);

impl SawtoothOscillator {
    pub fn new(sample_rate: f32) -> Self {
        Self(PhaseAccumulator::new(sample_rate))
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        self.0.set_frequency(frequency);
    }

    pub fn next_sample(&mut self) -> f32 {
        let phase = self.0.advance();
        2.0 * phase - 1.0
    }
}

/// A triangle wave: rises from 0 to 1 over the first quarter-period, falls
/// to -1 by the three-quarter mark, then rises back to 0.
pub struct TriangleOscillator(PhaseAccumulator);

impl TriangleOscillator {
    pub fn new(sample_rate: f32) -> Self {
        Self(PhaseAccumulator::new(sample_rate))
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        self.0.set_frequency(frequency);
    }

    pub fn next_sample(&mut self) -> f32 {
        let phase = self.0.advance();
        if phase < 0.25 {
            4.0 * phase
        } else if phase < 0.75 {
            2.0 - 4.0 * phase
        } else {
            4.0 * phase - 4.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Runs `period_samples` more calls after `first`, then asserts the
    /// waveform is back where it started -- valid for any of the periodic
    /// oscillators below, since 100Hz at 44.1kHz is an exact 441-sample
    /// period with no rounding error.
    fn assert_returns_to_start_after_one_period(mut next_sample: impl FnMut() -> f32) {
        let period_samples = 441; // 44100.0 / 100.0
        let first = next_sample();
        for _ in 0..period_samples - 1 {
            next_sample();
        }
        let after_one_period = next_sample();
        assert!(
            (first - after_one_period).abs() < 1e-3,
            "expected the waveform to repeat after one period: first={first}, after_one_period={after_one_period}"
        );
    }

    mod sine {
        use super::*;

        #[test]
        fn silent_at_zero_frequency() {
            let mut osc = SineOscillator::new(44100.0);
            for _ in 0..10 {
                assert_eq!(osc.next_sample(), 0.0);
            }
        }

        #[test]
        fn first_sample_starts_at_zero_phase() {
            let mut osc = SineOscillator::new(44100.0);
            osc.set_frequency(440.0);
            assert_eq!(osc.next_sample(), 0.0);
        }

        #[test]
        fn completes_one_period_in_sample_rate_over_frequency_samples() {
            let mut osc = SineOscillator::new(44100.0);
            osc.set_frequency(100.0);
            assert_returns_to_start_after_one_period(|| osc.next_sample());
        }

        #[test]
        fn stays_within_unit_amplitude() {
            let mut osc = SineOscillator::new(44100.0);
            osc.set_frequency(440.0);
            for _ in 0..1000 {
                let sample = osc.next_sample();
                assert!((-1.0..=1.0).contains(&sample));
            }
        }
    }

    mod square {
        use super::*;

        #[test]
        fn high_for_first_half_of_period_low_for_second() {
            let mut osc = SquareOscillator::new(44100.0);
            osc.set_frequency(100.0);
            assert_eq!(osc.next_sample(), 1.0);
            for _ in 0..220 {
                osc.next_sample();
            }
            // The 222nd sample is the first whose phase has crossed 0.5.
            assert_eq!(osc.next_sample(), -1.0);
        }

        #[test]
        fn completes_one_period_in_sample_rate_over_frequency_samples() {
            let mut osc = SquareOscillator::new(44100.0);
            osc.set_frequency(100.0);
            assert_returns_to_start_after_one_period(|| osc.next_sample());
        }

        #[test]
        fn only_takes_values_plus_or_minus_one() {
            let mut osc = SquareOscillator::new(44100.0);
            osc.set_frequency(440.0);
            for _ in 0..1000 {
                let sample = osc.next_sample();
                assert!(sample == 1.0 || sample == -1.0);
            }
        }
    }

    mod sawtooth {
        use super::*;

        #[test]
        fn ramps_up_from_minus_one() {
            let mut osc = SawtoothOscillator::new(44100.0);
            osc.set_frequency(100.0);
            let first = osc.next_sample();
            let second = osc.next_sample();
            assert_eq!(first, -1.0);
            assert!(second > first);
        }

        #[test]
        fn completes_one_period_in_sample_rate_over_frequency_samples() {
            let mut osc = SawtoothOscillator::new(44100.0);
            osc.set_frequency(100.0);
            assert_returns_to_start_after_one_period(|| osc.next_sample());
        }

        #[test]
        fn stays_within_unit_amplitude() {
            let mut osc = SawtoothOscillator::new(44100.0);
            osc.set_frequency(440.0);
            for _ in 0..1000 {
                let sample = osc.next_sample();
                assert!((-1.0..=1.0).contains(&sample));
            }
        }
    }

    mod triangle {
        use super::*;

        #[test]
        fn starts_at_zero_and_rises() {
            let mut osc = TriangleOscillator::new(44100.0);
            osc.set_frequency(100.0);
            let first = osc.next_sample();
            let second = osc.next_sample();
            assert_eq!(first, 0.0);
            assert!(second > first);
        }

        #[test]
        fn completes_one_period_in_sample_rate_over_frequency_samples() {
            let mut osc = TriangleOscillator::new(44100.0);
            osc.set_frequency(100.0);
            assert_returns_to_start_after_one_period(|| osc.next_sample());
        }

        #[test]
        fn stays_within_unit_amplitude() {
            let mut osc = TriangleOscillator::new(44100.0);
            osc.set_frequency(440.0);
            for _ in 0..1000 {
                let sample = osc.next_sample();
                assert!((-1.0..=1.0).contains(&sample));
            }
        }
    }
}
