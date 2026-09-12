use crate::dsp::envelope::Envelope;
use crate::dsp::{Oscillator, OscillatorKind};

/// Attack/Decay/Sustain/Release timings for one `Instrument` preset.
#[derive(Debug, Clone, Copy, PartialEq)]
struct EnvelopeParams {
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
}

/// A named instrument a user picks, rather than reasoning about
/// oscillators/envelopes directly. Each variant fixes both a
/// waveform-generation technique (`OscillatorKind`) and an amplitude
/// envelope, tuned to sound like the named instrument.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Instrument {
    SinePad,
    SquareLead,
    SawBass,
    Flute,
    PluckedString,
    ElectricPiano,
    Bell,
}

impl Instrument {
    pub const ALL: [Instrument; 7] = [
        Instrument::SinePad,
        Instrument::SquareLead,
        Instrument::SawBass,
        Instrument::Flute,
        Instrument::PluckedString,
        Instrument::ElectricPiano,
        Instrument::Bell,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Instrument::SinePad => "Sine Pad",
            Instrument::SquareLead => "Square Lead",
            Instrument::SawBass => "Saw Bass",
            Instrument::Flute => "Flute",
            Instrument::PluckedString => "Plucked String",
            Instrument::ElectricPiano => "Electric Piano",
            Instrument::Bell => "Bell",
        }
    }

    fn oscillator_kind(self) -> OscillatorKind {
        match self {
            Instrument::SinePad => OscillatorKind::Sine,
            Instrument::SquareLead => OscillatorKind::Square,
            Instrument::SawBass => OscillatorKind::Sawtooth,
            Instrument::Flute => OscillatorKind::Triangle,
            Instrument::PluckedString => OscillatorKind::KarplusStrong,
            Instrument::ElectricPiano => OscillatorKind::Fm {
                modulator_ratio: 1.0,
                modulation_index: 2.0,
            },
            Instrument::Bell => OscillatorKind::Fm {
                modulator_ratio: 3.5,
                modulation_index: 8.0,
            },
        }
    }

    fn envelope_params(self) -> EnvelopeParams {
        match self {
            Instrument::SinePad => EnvelopeParams {
                attack: 0.05,
                decay: 0.1,
                sustain: 0.8,
                release: 0.4,
            },
            Instrument::SquareLead => EnvelopeParams {
                attack: 0.005,
                decay: 0.05,
                sustain: 0.6,
                release: 0.1,
            },
            Instrument::SawBass => EnvelopeParams {
                attack: 0.005,
                decay: 0.1,
                sustain: 0.5,
                release: 0.05,
            },
            Instrument::Flute => EnvelopeParams {
                attack: 0.08,
                decay: 0.05,
                sustain: 0.9,
                release: 0.2,
            },
            Instrument::PluckedString => EnvelopeParams {
                // The Karplus-Strong delay line already decays the tone
                // itself; the envelope here mostly just shapes the
                // attack and lets a held note keep ringing.
                attack: 0.001,
                decay: 0.05,
                sustain: 1.0,
                release: 0.3,
            },
            Instrument::ElectricPiano => EnvelopeParams {
                attack: 0.005,
                decay: 0.3,
                sustain: 0.4,
                release: 0.3,
            },
            Instrument::Bell => EnvelopeParams {
                attack: 0.001,
                decay: 0.5,
                sustain: 0.1,
                release: 1.5,
            },
        }
    }

    pub fn build_oscillator(self, sample_rate: f32) -> Oscillator {
        self.oscillator_kind().build(sample_rate)
    }

    pub fn build_envelope(self, sample_rate: f32) -> Envelope {
        let params = self.envelope_params();
        Envelope::new(
            sample_rate,
            params.attack,
            params.decay,
            params.sustain,
            params.release,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn every_instrument_has_a_unique_name() {
        let names: HashSet<&str> = Instrument::ALL.iter().map(|i| i.name()).collect();
        assert_eq!(names.len(), Instrument::ALL.len());
    }

    #[test]
    fn every_instrument_builds_a_bounded_oscillator() {
        for instrument in Instrument::ALL {
            let mut osc = instrument.build_oscillator(44100.0);
            osc.set_frequency(220.0);
            for _ in 0..1000 {
                let sample = osc.next_sample();
                assert!(
                    (-1.0..=1.0).contains(&sample),
                    "{} produced an out-of-range sample: {sample}",
                    instrument.name()
                );
            }
        }
    }

    #[test]
    fn every_instrument_builds_an_envelope_that_reaches_and_leaves_silence() {
        for instrument in Instrument::ALL {
            let mut envelope = instrument.build_envelope(44100.0);
            assert!(
                !envelope.is_active(),
                "{} should start idle",
                instrument.name()
            );

            envelope.note_on();
            assert!(
                envelope.is_active(),
                "{} should activate on note_on",
                instrument.name()
            );

            envelope.note_off();
            // Run well past any instrument's release time.
            for _ in 0..200_000 {
                envelope.next_sample();
            }
            assert!(
                !envelope.is_active(),
                "{} should return to idle after release",
                instrument.name()
            );
        }
    }

    #[test]
    fn bell_uses_a_more_inharmonic_fm_ratio_than_electric_piano() {
        assert_eq!(
            Instrument::ElectricPiano.oscillator_kind(),
            OscillatorKind::Fm {
                modulator_ratio: 1.0,
                modulation_index: 2.0
            }
        );
        assert_eq!(
            Instrument::Bell.oscillator_kind(),
            OscillatorKind::Fm {
                modulator_ratio: 3.5,
                modulation_index: 8.0
            }
        );
    }

    #[test]
    fn plucked_string_uses_karplus_strong() {
        assert_eq!(
            Instrument::PluckedString.oscillator_kind(),
            OscillatorKind::KarplusStrong
        );
    }
}
