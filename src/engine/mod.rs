pub mod voice;

use crate::midi::MidiEvent;
use crate::engine::voice::Voice;

pub struct SynthEngine {
    voices: Vec<Voice>,
    sample_rate: f32,
}

impl SynthEngine {
    pub fn new(sample_rate: f32, num_voices: usize) -> Self {
        let mut voices = Vec::with_capacity(num_voices);
        for _ in 0..num_voices {
            voices.push(Voice::new(sample_rate));
        }
        Self { voices, sample_rate }
    }

    pub fn handle_event(&mut self, event: MidiEvent) {
        match event {
            MidiEvent::NoteOn { pitch, .. } => {
                // Find an inactive voice or reuse oldest
                if let Some(voice) = self.voices.iter_mut().find(|v| !v.is_active) {
                    voice.note_on(pitch);
                } else {
                    // Simple replacement: first voice (could be improved to LRU)
                    if !self.voices.is_empty() {
                        self.voices[0].note_on(pitch);
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
            if voice.is_active {
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
