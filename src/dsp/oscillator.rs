use std::f32::consts::PI;

pub struct SineOscillator {
    frequency: f32,
    sample_rate: f32,
    phase: f32,
}

impl SineOscillator {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            frequency: 0.0,
            sample_rate,
            phase: 0.0,
        }
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency;
    }

    pub fn next_sample(&mut self) -> f32 {
        let sample = (2.0 * PI * self.phase).sin();
        self.phase = (self.phase + self.frequency / self.sample_rate).fract();
        sample
    }
}

#[cfg(test)]
mod tests {
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
        let sample_rate = 44100.0;
        let frequency = 100.0;
        let mut osc = SineOscillator::new(sample_rate);
        osc.set_frequency(frequency);

        let period_samples = (sample_rate / frequency).round() as usize;
        for _ in 0..period_samples {
            osc.next_sample();
        }

        // Back near the start of the waveform after one full period.
        let sample = osc.next_sample();
        assert!(
            sample.abs() < 0.05,
            "expected near-zero sample, got {sample}"
        );
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
