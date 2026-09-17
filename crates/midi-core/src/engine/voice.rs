use crate::dsp::Oscillator;
use crate::dsp::envelope::Envelope;
use crate::dsp::filter::LowPassFilter;
use crate::instrument::Instrument;

pub struct Voice {
    pub pitch: u8,
    oscillator: Oscillator,
    filter: LowPassFilter,
    envelope: Envelope,
    velocity_gain: f32,
}

impl Voice {
    pub fn new(sample_rate: f32, instrument: Instrument) -> Self {
        Self {
            pitch: 0,
            oscillator: instrument.build_oscillator(sample_rate),
            filter: instrument.build_filter(sample_rate),
            envelope: instrument.build_envelope(sample_rate),
            velocity_gain: 1.0,
        }
    }

    pub fn note_on(&mut self, pitch: u8, velocity: u8) {
        self.pitch = pitch;
        self.oscillator.set_frequency(midi_pitch_to_freq(pitch));
        self.velocity_gain = velocity as f32 / 127.0;
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
            let filtered = self.filter.process(self.oscillator.next_sample());
            filtered * self.envelope.next_sample() * self.velocity_gain
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
        let mut voice = Voice::new(44100.0, Instrument::SinePad);
        assert!(!voice.is_active());
        assert_eq!(voice.next_sample(), 0.0);
    }

    #[test]
    fn note_on_activates_voice_with_pitch() {
        let mut voice = Voice::new(44100.0, Instrument::SinePad);
        voice.note_on(60, 100);
        assert!(voice.is_active());
        assert_eq!(voice.pitch, 60);
    }

    #[test]
    fn note_off_keeps_voice_active_during_release() {
        let mut voice = Voice::new(44100.0, Instrument::SinePad);
        voice.note_on(60, 100);
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
        let mut voice = Voice::new(44100.0, Instrument::SinePad);
        voice.note_on(60, 100);
        for _ in 0..100 {
            voice.next_sample();
        }
        voice.note_off();
        // Run well past any instrument's release time.
        for _ in 0..200_000 {
            voice.next_sample();
        }
        assert!(!voice.is_active());
        assert_eq!(voice.next_sample(), 0.0);
    }

    #[test]
    fn karplus_strong_voice_produces_bounded_sound_after_note_on() {
        let mut voice = Voice::new(44100.0, Instrument::PluckedString);
        voice.note_on(60, 100);
        for _ in 0..1000 {
            let sample = voice.next_sample();
            assert!((-1.0..=1.0).contains(&sample));
        }
    }

    #[test]
    fn square_voice_produces_bounded_sound_after_note_on() {
        let mut voice = Voice::new(44100.0, Instrument::SquareLead);
        voice.note_on(60, 100);
        for _ in 0..1000 {
            let sample = voice.next_sample();
            assert!((-1.0..=1.0).contains(&sample));
        }
    }

    #[test]
    fn velocity_scales_output_proportionally() {
        // Two identically-timed voices differing only in velocity should
        // produce samples in exactly the ratio of their velocity gains.
        let mut full = Voice::new(44100.0, Instrument::SquareLead);
        full.note_on(60, 127);
        let mut half = Voice::new(44100.0, Instrument::SquareLead);
        half.note_on(60, 64);

        let full_sample = full.next_sample();
        let half_sample = half.next_sample();
        assert!((half_sample / full_sample - 64.0 / 127.0).abs() < 1e-4);
    }

    #[test]
    fn lower_velocity_produces_quieter_output() {
        let mut loud = Voice::new(44100.0, Instrument::SquareLead);
        loud.note_on(60, 127);
        let mut quiet = Voice::new(44100.0, Instrument::SquareLead);
        quiet.note_on(60, 32);

        // Skip the attack so both voices are at full envelope level.
        for _ in 0..100 {
            loud.next_sample();
            quiet.next_sample();
        }
        assert!(quiet.next_sample().abs() < loud.next_sample().abs());
    }

    #[test]
    fn minimum_velocity_is_much_quieter_than_maximum() {
        let mut voice = Voice::new(44100.0, Instrument::SquareLead);
        voice.note_on(60, 1);
        for _ in 0..100 {
            voice.next_sample();
        }
        assert!(voice.next_sample().abs() < 0.05);
    }
}
