use std::f32::consts::PI;

/// A one-pole (RC) low-pass filter: each output sample moves partway
/// from the previous output towards the current input, where `alpha`
/// (derived from the cutoff frequency and sample rate) sets how far.
/// A low cutoff smooths fast-changing input harder; a very high
/// cutoff (near the Nyquist frequency) leaves the signal almost
/// unchanged.
pub struct LowPassFilter {
    alpha: f32,
    previous_output: f32,
}

impl LowPassFilter {
    pub fn new(sample_rate: f32, cutoff_hz: f32) -> Self {
        Self {
            alpha: Self::alpha_for(sample_rate, cutoff_hz),
            previous_output: 0.0,
        }
    }

    fn alpha_for(sample_rate: f32, cutoff_hz: f32) -> f32 {
        let dt = 1.0 / sample_rate;
        let rc = 1.0 / (2.0 * PI * cutoff_hz);
        dt / (rc + dt)
    }

    pub fn process(&mut self, input: f32) -> f32 {
        self.previous_output += self.alpha * (input - self.previous_output);
        self.previous_output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converges_to_a_constant_input_over_time() {
        let mut filter = LowPassFilter::new(44100.0, 500.0);
        let mut output = 0.0;
        for _ in 0..10_000 {
            output = filter.process(1.0);
        }
        assert!((output - 1.0).abs() < 1e-3);
    }

    #[test]
    fn output_never_overshoots_a_constant_input() {
        let mut filter = LowPassFilter::new(44100.0, 500.0);
        for _ in 0..1000 {
            let sample = filter.process(1.0);
            assert!((0.0..=1.0).contains(&sample));
        }
    }

    #[test]
    fn lower_cutoff_smooths_an_alternating_signal_more_than_a_higher_one() {
        let sample_rate = 44100.0;
        let mut smoothed = LowPassFilter::new(sample_rate, 200.0);
        let mut mostly_passthrough = LowPassFilter::new(sample_rate, 15_000.0);

        let mut smoothed_range = 0.0f32;
        let mut passthrough_range = 0.0f32;
        for i in 0..200 {
            let input = if i % 2 == 0 { 1.0 } else { -1.0 };
            smoothed_range = smoothed_range.max(smoothed.process(input).abs());
            passthrough_range = passthrough_range.max(mostly_passthrough.process(input).abs());
        }

        assert!(
            smoothed_range < passthrough_range,
            "expected the low-cutoff filter to track a fast-alternating signal less closely: \
             smoothed={smoothed_range}, passthrough={passthrough_range}"
        );
    }
}
