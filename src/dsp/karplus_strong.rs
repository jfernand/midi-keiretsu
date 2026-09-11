/// A Karplus-Strong plucked-string oscillator.
///
/// Physical-modeling synthesis: a short burst of noise is fed into a
/// delay line whose length sets the pitch; each sample is read back and
/// replaced by the average of itself and its neighbor, which acts as a
/// lossy low-pass filter and gives the characteristic decaying pluck.
///
/// See Kevin Karplus and Alex Strong, "Digital Synthesis of
/// Plucked-String and Drum Timbres," Computer Music Journal 7(2), 1983
/// (`docs/papers/karplus-strong-1983.pdf`).
pub struct KarplusStrongOscillator {
    sample_rate: f32,
    buffer: Vec<f32>,
    position: usize,
    rng_state: u32,
}

impl KarplusStrongOscillator {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            buffer: Vec::new(),
            position: 0,
            rng_state: 0x9E37_79B9,
        }
    }

    /// "Plucks" the string at the given frequency: reseeds the delay
    /// line with noise, sized so the loop takes `sample_rate / frequency`
    /// samples to repeat.
    pub fn set_frequency(&mut self, frequency: f32) {
        let length = (self.sample_rate / frequency).round().max(2.0) as usize;
        self.buffer = (0..length).map(|_| self.next_noise_sample()).collect();
        self.position = 0;
    }

    /// xorshift32, seeded per-oscillator so each voice's pluck is a
    /// distinct noise burst rather than sample-for-sample identical.
    fn next_noise_sample(&mut self) -> f32 {
        self.rng_state ^= self.rng_state << 13;
        self.rng_state ^= self.rng_state >> 17;
        self.rng_state ^= self.rng_state << 5;
        (self.rng_state as f32 / u32::MAX as f32) * 2.0 - 1.0
    }

    pub fn next_sample(&mut self) -> f32 {
        if self.buffer.is_empty() {
            return 0.0;
        }
        let len = self.buffer.len();
        let current = self.buffer[self.position];
        let next = self.buffer[(self.position + 1) % len];
        self.buffer[self.position] = 0.5 * (current + next);
        self.position = (self.position + 1) % len;
        current
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn silent_before_a_pluck() {
        let mut osc = KarplusStrongOscillator::new(44100.0);
        for _ in 0..10 {
            assert_eq!(osc.next_sample(), 0.0);
        }
    }

    #[test]
    fn delay_line_length_matches_frequency() {
        let mut osc = KarplusStrongOscillator::new(44100.0);
        osc.set_frequency(441.0);
        assert_eq!(osc.buffer.len(), 100);
    }

    #[test]
    fn stays_within_unit_amplitude_after_a_pluck() {
        let mut osc = KarplusStrongOscillator::new(44100.0);
        osc.set_frequency(220.0);
        for _ in 0..5000 {
            let sample = osc.next_sample();
            assert!((-1.0..=1.0).contains(&sample));
        }
    }

    #[test]
    fn pluck_decays_towards_silence_over_time() {
        let mut osc = KarplusStrongOscillator::new(44100.0);
        osc.set_frequency(220.0);

        let energy = |osc: &mut KarplusStrongOscillator, n: usize| -> f32 {
            (0..n).map(|_| osc.next_sample().abs()).sum()
        };

        let early_energy = energy(&mut osc, 2000);
        let late_energy = energy(&mut osc, 2000);
        assert!(
            late_energy < early_energy,
            "expected the pluck to decay: early={early_energy}, late={late_energy}"
        );
    }

    #[test]
    fn replucking_resets_the_delay_line() {
        let mut osc = KarplusStrongOscillator::new(44100.0);
        osc.set_frequency(220.0);
        for _ in 0..10_000 {
            osc.next_sample();
        }
        let decayed_energy: f32 = (0..2000).map(|_| osc.next_sample().abs()).sum();

        osc.set_frequency(220.0);
        let fresh_energy: f32 = (0..2000).map(|_| osc.next_sample().abs()).sum();

        assert!(
            fresh_energy > decayed_energy,
            "expected a fresh pluck to be louder than the decayed tail: \
             fresh={fresh_energy}, decayed={decayed_energy}"
        );
    }
}
