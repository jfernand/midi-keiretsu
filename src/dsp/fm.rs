use std::f32::consts::PI;

/// A two-operator FM (phase modulation) oscillator: a sine carrier whose
/// phase is modulated by a sine modulator running at `modulator_ratio`
/// times the carrier frequency. `modulation_index` controls how much
/// modulation is applied -- higher values add more (and more inharmonic,
/// for a non-integer ratio) sidebands, giving brighter or bell-like
/// tones from the same simple sine building blocks.
pub struct FmOscillator {
    sample_rate: f32,
    carrier_frequency: f32,
    carrier_phase: f32,
    modulator_phase: f32,
    modulator_ratio: f32,
    modulation_index: f32,
}

impl FmOscillator {
    pub fn new(sample_rate: f32, modulator_ratio: f32, modulation_index: f32) -> Self {
        Self {
            sample_rate,
            carrier_frequency: 0.0,
            carrier_phase: 0.0,
            modulator_phase: 0.0,
            modulator_ratio,
            modulation_index,
        }
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        self.carrier_frequency = frequency;
    }

    pub fn next_sample(&mut self) -> f32 {
        let modulator = (2.0 * PI * self.modulator_phase).sin();
        let sample = (2.0 * PI * self.carrier_phase + self.modulation_index * modulator).sin();

        self.carrier_phase =
            (self.carrier_phase + self.carrier_frequency / self.sample_rate).fract();
        let modulator_frequency = self.carrier_frequency * self.modulator_ratio;
        self.modulator_phase =
            (self.modulator_phase + modulator_frequency / self.sample_rate).fract();

        sample
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn silent_at_zero_frequency() {
        let mut osc = FmOscillator::new(44100.0, 1.0, 2.0);
        for _ in 0..10 {
            assert_eq!(osc.next_sample(), 0.0);
        }
    }

    #[test]
    fn first_sample_starts_at_zero_phase() {
        let mut osc = FmOscillator::new(44100.0, 3.5, 8.0);
        osc.set_frequency(440.0);
        assert_eq!(osc.next_sample(), 0.0);
    }

    #[test]
    fn stays_within_unit_amplitude() {
        let mut osc = FmOscillator::new(44100.0, 3.5, 8.0);
        osc.set_frequency(440.0);
        for _ in 0..1000 {
            let sample = osc.next_sample();
            assert!((-1.0..=1.0).contains(&sample));
        }
    }

    #[test]
    fn zero_modulation_index_matches_a_plain_sine() {
        let mut fm = FmOscillator::new(44100.0, 3.5, 0.0);
        fm.set_frequency(220.0);

        let mut sine = crate::dsp::oscillator::SineOscillator::new(44100.0);
        sine.set_frequency(220.0);

        for _ in 0..1000 {
            assert!((fm.next_sample() - sine.next_sample()).abs() < 1e-5);
        }
    }

    #[test]
    fn nonzero_modulation_index_differs_from_a_plain_sine() {
        let mut fm = FmOscillator::new(44100.0, 3.5, 8.0);
        fm.set_frequency(220.0);

        let mut sine = crate::dsp::oscillator::SineOscillator::new(44100.0);
        sine.set_frequency(220.0);

        let differs = (0..1000).any(|_| (fm.next_sample() - sine.next_sample()).abs() > 1e-3);
        assert!(
            differs,
            "expected FM with a nonzero index to diverge from a plain sine"
        );
    }
}
