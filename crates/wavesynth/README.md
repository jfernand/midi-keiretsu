# wavesynth

A `no_std` MIDI-driven synthesizer engine: oscillators, ADSR envelopes, a
low-pass filter, and named instrument presets — no operating system,
audio device, or MIDI hardware required. Feed it `MidiEvent`s, pull mono
`f32` samples in `[-1.0, 1.0]` one at a time.

```rust
use wavesynth::engine::SynthEngine;
use wavesynth::instrument::Instrument;
use wavesynth::midi::MidiEvent;

let mut engine = SynthEngine::new(44_100.0, 8, Instrument::ElectricPiano);

engine.handle_event(MidiEvent::NoteOn { pitch: 60, velocity: 100 });
let samples: Vec<f32> = (0..44_100).map(|_| engine.next_sample()).collect();
```

## Instruments

Rather than picking a raw waveform-generation technique, you pick a named
instrument. Each one bundles an oscillator, an ADSR envelope, and a
low-pass filter cutoff tuned to sound like the name suggests:

- **Sine Pad**, **Square Lead**, **Saw Bass**, **Flute** — the classic
  waveforms (sine, square, sawtooth, triangle), each shaped with its own
  envelope and (for the harmonically rich ones) a subtractive-synthesis
  low-pass filter.
- **Plucked String** — [Karplus-Strong](https://en.wikipedia.org/wiki/Karplus%E2%80%93Strong_string_synthesis)
  physical modeling: a noise burst decaying through a delay line.
- **Electric Piano**, **Bell** — two-operator FM (phase modulation)
  synthesis, distinguished only by modulator ratio and index.

## `no_std`

This crate has no dependency on `std` outside test code (`cargo test`
always links `std` for the test harness, so existing tests use it freely
via `#[cfg(test)]`). It depends only on [`libm`](https://crates.io/crates/libm)
for floating-point math (`sin`, `pow`, `round`, ...) not available on
`core::f32`, and `alloc` for a couple of `Vec`s (the voice pool and the
Karplus-Strong delay line).

## License

Licensed under either of [Apache License, Version 2.0](../../LICENSE-APACHE)
or [MIT license](../../LICENSE-MIT) at your option.
