use std::f32::consts::PI;

pub struct SineOscillator {
    frequency: f32,
    sample_rate: f32,
    phase: f32,
}

impl SineOscillator {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            frequency: 0.0,
            sample_rate,
            phase: 0.0,
        }
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency;
    }

    pub fn next_sample(&mut self) -> f32 {
        let sample = (2.0 * PI * self.phase).sin();
        self.phase = (self.phase + self.frequency / self.sample_rate).fract();
        sample
    }
}
