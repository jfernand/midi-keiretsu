pub mod envelope;
pub mod karplus_strong;
pub mod oscillator;

use karplus_strong::KarplusStrongOscillator;
use oscillator::SineOscillator;

/// Which waveform-generation algorithm a `Voice` should use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OscillatorKind {
    Sine,
    KarplusStrong,
}

impl OscillatorKind {
    pub fn build(self, sample_rate: f32) -> Oscillator {
        match self {
            OscillatorKind::Sine => Oscillator::Sine(SineOscillator::new(sample_rate)),
            OscillatorKind::KarplusStrong => {
                Oscillator::KarplusStrong(KarplusStrongOscillator::new(sample_rate))
            }
        }
    }
}

/// A waveform generator, dispatching to whichever concrete oscillator
/// a voice was built with.
pub enum Oscillator {
    Sine(SineOscillator),
    KarplusStrong(KarplusStrongOscillator),
}

impl Oscillator {
    pub fn set_frequency(&mut self, frequency: f32) {
        match self {
            Oscillator::Sine(osc) => osc.set_frequency(frequency),
            Oscillator::KarplusStrong(osc) => osc.set_frequency(frequency),
        }
    }

    pub fn next_sample(&mut self) -> f32 {
        match self {
            Oscillator::Sine(osc) => osc.next_sample(),
            Oscillator::KarplusStrong(osc) => osc.next_sample(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sine_kind_builds_a_sine_oscillator() {
        let mut osc = OscillatorKind::Sine.build(44100.0);
        // A sine oscillator starts silent until a frequency is set.
        assert_eq!(osc.next_sample(), 0.0);
    }

    #[test]
    fn karplus_strong_kind_builds_a_plucked_string_oscillator() {
        let mut osc = OscillatorKind::KarplusStrong.build(44100.0);
        // Silent until "plucked" by setting a frequency.
        assert_eq!(osc.next_sample(), 0.0);
        osc.set_frequency(220.0);
        let sample = osc.next_sample();
        assert!((-1.0..=1.0).contains(&sample));
    }
}
