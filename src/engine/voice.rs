use crate::dsp::oscillator::SineOscillator;

pub struct Voice {
    pub pitch: u8,
    oscillator: SineOscillator,
    pub is_active: bool,
}

impl Voice {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            pitch: 0,
            oscillator: SineOscillator::new(sample_rate),
            is_active: false,
        }
    }

    pub fn note_on(&mut self, pitch: u8) {
        self.pitch = pitch;
        self.oscillator.set_frequency(midi_pitch_to_freq(pitch));
        self.is_active = true;
    }

    pub fn note_off(&mut self) {
        self.is_active = false;
    }

    pub fn next_sample(&mut self) -> f32 {
        if self.is_active {
            self.oscillator.next_sample()
        } else {
            0.0
        }
    }
}

fn midi_pitch_to_freq(pitch: u8) -> f32 {
    440.0 * 2.0_f32.powf((pitch as f32 - 69.0) / 12.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a4_is_440hz() {
        assert!((midi_pitch_to_freq(69) - 440.0).abs() < 1e-3);
    }

    #[test]
    fn one_octave_up_doubles_frequency() {
        let a4 = midi_pitch_to_freq(69);
        let a5 = midi_pitch_to_freq(81);
        assert!((a5 - 2.0 * a4).abs() < 1e-3);
    }

    #[test]
    fn new_voice_is_inactive_and_silent() {
        let mut voice = Voice::new(44100.0);
        assert!(!voice.is_active);
        assert_eq!(voice.next_sample(), 0.0);
    }

    #[test]
    fn note_on_activates_voice_with_pitch() {
        let mut voice = Voice::new(44100.0);
        voice.note_on(60);
        assert!(voice.is_active);
        assert_eq!(voice.pitch, 60);
    }

    #[test]
    fn note_off_deactivates_and_silences_voice() {
        let mut voice = Voice::new(44100.0);
        voice.note_on(60);
        voice.note_off();
        assert!(!voice.is_active);
        assert_eq!(voice.next_sample(), 0.0);
    }
}
