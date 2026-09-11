use crate::dsp::envelope::Envelope;
use crate::dsp::oscillator::SineOscillator;

const ATTACK_SECONDS: f32 = 0.01;
const DECAY_SECONDS: f32 = 0.1;
const SUSTAIN_LEVEL: f32 = 0.7;
const RELEASE_SECONDS: f32 = 0.2;

pub struct Voice {
    pub pitch: u8,
    oscillator: SineOscillator,
    envelope: Envelope,
}

impl Voice {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            pitch: 0,
            oscillator: SineOscillator::new(sample_rate),
            envelope: Envelope::new(
                sample_rate,
                ATTACK_SECONDS,
                DECAY_SECONDS,
                SUSTAIN_LEVEL,
                RELEASE_SECONDS,
            ),
        }
    }

    pub fn note_on(&mut self, pitch: u8) {
        self.pitch = pitch;
        self.oscillator.set_frequency(midi_pitch_to_freq(pitch));
        self.envelope.note_on();
    }

    /// Begins the release phase; the voice stays active until the
    /// envelope finishes decaying to silence.
    pub fn note_off(&mut self) {
        self.envelope.note_off();
    }

    /// True while the envelope is anywhere in attack/decay/sustain/release,
    /// i.e. while this voice still has audible output to produce.
    pub fn is_active(&self) -> bool {
        self.envelope.is_active()
    }

    pub fn next_sample(&mut self) -> f32 {
        if self.envelope.is_active() {
            self.oscillator.next_sample() * self.envelope.next_sample()
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
        assert!(!voice.is_active());
        assert_eq!(voice.next_sample(), 0.0);
    }

    #[test]
    fn note_on_activates_voice_with_pitch() {
        let mut voice = Voice::new(44100.0);
        voice.note_on(60);
        assert!(voice.is_active());
        assert_eq!(voice.pitch, 60);
    }

    #[test]
    fn note_off_keeps_voice_active_during_release() {
        let mut voice = Voice::new(44100.0);
        voice.note_on(60);
        for _ in 0..100 {
            voice.next_sample();
        }
        voice.note_off();
        // Right after note-off the release phase has just begun.
        voice.next_sample();
        assert!(voice.is_active());
    }

    #[test]
    fn voice_goes_silent_and_inactive_once_release_completes() {
        let mut voice = Voice::new(44100.0);
        voice.note_on(60);
        for _ in 0..100 {
            voice.next_sample();
        }
        voice.note_off();
        // Release is 0.2s at 44.1kHz; run well past that.
        for _ in 0..20_000 {
            voice.next_sample();
        }
        assert!(!voice.is_active());
        assert_eq!(voice.next_sample(), 0.0);
    }
}
