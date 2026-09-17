pub mod voice;

use crate::engine::voice::Voice;
use crate::instrument::Instrument;
use crate::midi::MidiEvent;
use alloc::vec::Vec;

pub struct SynthEngine {
    voices: Vec<Voice>,
    sample_rate: f32,
}

impl SynthEngine {
    pub fn new(sample_rate: f32, num_voices: usize, instrument: Instrument) -> Self {
        let mut voices = Vec::with_capacity(num_voices);
        for _ in 0..num_voices {
            voices.push(Voice::new(sample_rate, instrument));
        }
        Self {
            voices,
            sample_rate,
        }
    }

    pub fn handle_event(&mut self, event: MidiEvent) {
        match event {
            MidiEvent::NoteOn { pitch, velocity } => {
                // Find an inactive voice or reuse oldest
                if let Some(voice) = self.voices.iter_mut().find(|v| !v.is_active()) {
                    voice.note_on(pitch, velocity);
                } else {
                    // Simple replacement: first voice (could be improved to LRU)
                    if !self.voices.is_empty() {
                        self.voices[0].note_on(pitch, velocity);
                    }
                }
            }
            MidiEvent::NoteOff { pitch } => {
                for voice in self.voices.iter_mut().filter(|v| v.pitch == pitch) {
                    voice.note_off();
                }
            }
            _ => {}
        }
    }

    pub fn next_sample(&mut self) -> f32 {
        let mut mixed = 0.0;
        let mut active_count = 0;
        for voice in self.voices.iter_mut() {
            if voice.is_active() {
                mixed += voice.next_sample();
                active_count += 1;
            }
        }
        if active_count > 0 {
            mixed / active_count as f32
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn note_on_activates_a_voice() {
        let mut engine = SynthEngine::new(44100.0, 4, Instrument::SinePad);
        engine.handle_event(MidiEvent::NoteOn {
            pitch: 60,
            velocity: 100,
        });
        assert_eq!(engine.voices.iter().filter(|v| v.is_active()).count(), 1);
    }

    #[test]
    fn note_off_starts_release_but_keeps_voice_active() {
        let mut engine = SynthEngine::new(44100.0, 4, Instrument::SinePad);
        engine.handle_event(MidiEvent::NoteOn {
            pitch: 60,
            velocity: 100,
        });
        engine.handle_event(MidiEvent::NoteOn {
            pitch: 64,
            velocity: 100,
        });
        engine.handle_event(MidiEvent::NoteOff { pitch: 60 });

        // Both voices are still audible: 60 is releasing, 64 is sustaining.
        assert_eq!(engine.voices.iter().filter(|v| v.is_active()).count(), 2);
    }

    #[test]
    fn voice_becomes_inactive_once_release_completes() {
        let mut engine = SynthEngine::new(44100.0, 4, Instrument::SinePad);
        engine.handle_event(MidiEvent::NoteOn {
            pitch: 60,
            velocity: 100,
        });
        engine.handle_event(MidiEvent::NoteOn {
            pitch: 64,
            velocity: 100,
        });
        engine.handle_event(MidiEvent::NoteOff { pitch: 60 });

        // Run well past any instrument's release time.
        for _ in 0..200_000 {
            engine.next_sample();
        }

        assert!(!engine.voices.iter().any(|v| v.pitch == 60 && v.is_active()));
        assert!(engine.voices.iter().any(|v| v.pitch == 64 && v.is_active()));
    }

    #[test]
    fn each_note_on_takes_a_separate_voice_up_to_capacity() {
        let mut engine = SynthEngine::new(44100.0, 2, Instrument::SinePad);
        engine.handle_event(MidiEvent::NoteOn {
            pitch: 60,
            velocity: 100,
        });
        engine.handle_event(MidiEvent::NoteOn {
            pitch: 64,
            velocity: 100,
        });

        assert_eq!(engine.voices.iter().filter(|v| v.is_active()).count(), 2);
    }

    #[test]
    fn steals_a_voice_when_all_are_busy() {
        let mut engine = SynthEngine::new(44100.0, 1, Instrument::SinePad);
        engine.handle_event(MidiEvent::NoteOn {
            pitch: 60,
            velocity: 100,
        });
        engine.handle_event(MidiEvent::NoteOn {
            pitch: 64,
            velocity: 100,
        });

        // Only one voice exists, so it must have been stolen for the new note.
        assert_eq!(engine.voices.len(), 1);
        assert!(engine.voices[0].is_active());
        assert_eq!(engine.voices[0].pitch, 64);
    }

    #[test]
    fn next_sample_is_silent_with_no_active_voices() {
        let mut engine = SynthEngine::new(44100.0, 4, Instrument::SinePad);
        assert_eq!(engine.next_sample(), 0.0);
    }

    #[test]
    fn note_on_velocity_scales_engine_output() {
        let mut loud = SynthEngine::new(44100.0, 1, Instrument::SquareLead);
        loud.handle_event(MidiEvent::NoteOn {
            pitch: 60,
            velocity: 127,
        });

        let mut quiet = SynthEngine::new(44100.0, 1, Instrument::SquareLead);
        quiet.handle_event(MidiEvent::NoteOn {
            pitch: 60,
            velocity: 32,
        });

        assert!(quiet.next_sample().abs() < loud.next_sample().abs());
    }

    #[test]
    fn next_sample_averages_active_voices() {
        let mut engine = SynthEngine::new(44100.0, 2, Instrument::SinePad);
        engine.handle_event(MidiEvent::NoteOn {
            pitch: 69,
            velocity: 100,
        });
        engine.handle_event(MidiEvent::NoteOn {
            pitch: 69,
            velocity: 100,
        });

        // Both voices share the same pitch/phase, so the mix equals either one alone.
        let mixed = engine.next_sample();
        assert_eq!(mixed, 0.0); // first sample of a sine oscillator is always 0
    }
}
