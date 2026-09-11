use crate::engine::SynthEngine;
use crate::midi::MidiEvent;
use crossbeam_channel::Receiver;
use rodio::Source;
use std::time::Duration;

pub struct SynthSource {
    engine: SynthEngine,
    midi_rx: Receiver<MidiEvent>,
}

impl SynthSource {
    pub fn new(sample_rate: u32, num_voices: usize, midi_rx: Receiver<MidiEvent>) -> Self {
        Self {
            engine: SynthEngine::new(sample_rate as f32, num_voices),
            midi_rx,
        }
    }
}

impl Iterator for SynthSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        // Drain any incoming midi events
        while let Ok(event) = self.midi_rx.try_recv() {
            self.engine.handle_event(event);
        }

        Some(self.engine.next_sample())
    }
}

impl Source for SynthSource {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        44100
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}
