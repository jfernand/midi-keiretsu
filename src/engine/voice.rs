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
