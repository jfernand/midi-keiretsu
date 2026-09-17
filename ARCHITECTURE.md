### Proposed Architecture for MIDI-Driven Synthesizer

Your initial list covers the core pillars of a synthesizer. To build a robust and idiomatic Rust implementation using `rodio`, here is a refined breakdown of the modules and some additional components you'll likely need:

### Workspace Layout

This is a Cargo workspace with two crates:

*   **`crates/midi-core`** — a `no_std` library holding everything below that has no I/O dependency: `MidiEvent`, `Instrument`, and everything under `engine/` and `dsp/`. `SynthEngine::{new, handle_event, next_sample}` is its whole public surface for driving the synth: feed it `MidiEvent`s, pull mono `f32` samples in `[-1.0, 1.0]` one at a time. Being `no_std` means it can be embedded in non-desktop targets (a game, an embedded/wasm target, ...) with no dependency on an OS or heap-backed I/O beyond `alloc` (used only for `Vec` in the voice pool and the Karplus-Strong delay line). See `AGENTS.md`'s "no_std dependencies" section for the ground rules when touching this crate.
*   **`crates/midi`** — the `std`-based desktop app: hardware MIDI input (`midir`), the computer-keyboard-as-MIDI-controller feature (`keyboard_midi.rs`/`virtual_midi.rs`, `rdev`/virtual `midir` ports), and audio playback (`rodio`). Depends on `midi-core` via a path dependency and is the only crate that knows those library names exist.

#### 1. `midi_input` (The Dispatcher)
*   **Role:** Reads MIDI events from a hardware port or a file.
*   **Functionality:** Translates raw MIDI bytes into high-level domain events (e.g., `NoteOn(pitch, velocity)`, `NoteOff(pitch)`, `ControlChange`).
*   **Implementation Tip:** Use a crate like `midir` for real-time input. It should probably run in its own thread or use a callback mechanism to avoid blocking the audio thread.

#### 2. `synth_engine` (The Core Logic)
*   **Role:** Manages the "Voices" and state.
*   **Components:**
    *   **Voice Management:** A polyphonic synth needs to manage multiple "Voices" (instances of your oscillators/envelopes). It needs logic to assign a new `NoteOn` to an available voice and release it on `NoteOff`.
    *   **Velocity Sensitivity:** `Voice::note_on` derives a linear `velocity_gain` (`velocity / 127`) from the `NoteOn`'s velocity and applies it on top of the oscillator/envelope output, so harder-hit notes play louder. The envelope's own attack/decay/release timing is unaffected by velocity — only output amplitude scales.
    *   **Modulation Matrix:** (Optional but recommended) Connects LFOs (Low Frequency Oscillators) or Envelopes to parameters like pitch or filter cutoff.
*   **Data Flow:** Receives high-level events from `midi_input` and updates the state of active voices.

#### 3. `dsp` (Digital Signal Processing - The "Sound Maker")
*   **Role:** The math-heavy part that generates the actual samples.
*   **Oscillators:** Implemented in `dsp/oscillator.rs` — Sine, Square, Sawtooth, and Triangle, all sharing a `PhaseAccumulator` for frequency/phase bookkeeping and differing only in how they turn a `0.0..1.0` phase into a sample. Implementing harmonics often involves "Additive Synthesis" (stacking sines) or "Subtractive Synthesis" (filtering rich waveforms) — the latter is now implemented (see Filters below).
*   **Karplus-Strong:** Implemented in `dsp/karplus_strong.rs` — a physical-modeling algorithm for plucked-string/percussive timbres. Each "pluck" (`set_frequency`) seeds a delay line with a short noise burst; each sample is read back and replaced with the average of itself and its neighbor, a lossy filter that gives the characteristic decaying pluck. See Kevin Karplus and Alex Strong, "Digital Synthesis of Plucked-String and Drum Timbres," *Computer Music Journal* 7(2), 1983 (paper: [`docs/papers/karplus-strong-1983.pdf`](docs/papers/karplus-strong-1983.pdf)).
*   **FM (Phase Modulation):** Implemented in `dsp/fm.rs` — a sine carrier whose phase is modulated by a sine modulator running at some ratio of the carrier frequency, scaled by a modulation index. At index 0 it's a plain sine; higher indices (and non-integer ratios) add sidebands for brighter or bell-like, inharmonic tones, all from simple sine building blocks. See John M. Chowning, "The Synthesis of Complex Audio Spectra by Means of Frequency Modulation," *Journal of the Audio Engineering Society* 21(7), 1973 (paper: [`docs/papers/chowning-fm-1973.pdf`](docs/papers/chowning-fm-1973.pdf)).
*   These six techniques (Sine, Square, Sawtooth, Triangle, Karplus-Strong, FM) are unified behind `dsp::OscillatorKind`/`Oscillator`, but a user never picks a technique directly — see `instrument.rs` below.
*   **Envelopes (ADSR):** Attack, Decay, Sustain, and Release. This is crucial for making the sound feel like an instrument rather than just a constant beep.
*   **Filters:** Implemented in `dsp/filter.rs` — `LowPassFilter`, a one-pole (RC) low-pass: each sample moves partway from the previous output towards the current input, where that step size (`alpha`) is derived from a cutoff frequency and the sample rate. Applied per-`Voice` to the raw oscillator output, before the envelope and velocity gain.

#### 3a. `instrument.rs` (Named Instrument Presets)
*   **Role:** The user-facing layer above `dsp` — lets someone pick "Electric Piano" or "Bell" instead of reasoning about oscillators, filters, and envelopes directly.
*   **Implementation:** `Instrument` is an enum of named presets (Sine Pad, Square Lead, Saw Bass, Flute, Plucked String, Electric Piano, Bell), each mapping to a fixed `dsp::OscillatorKind`, a tuned ADSR (`Envelope`), and a `LowPassFilter` cutoff. Square/sawtooth-based instruments (Square Lead, Saw Bass, Electric Piano) get a cutoff low enough to noticeably tame their brightness — classic subtractive synthesis; the rest get a cutoff high enough to stay essentially unfiltered. Electric Piano and Bell are both FM under the hood, distinguished only by modulator ratio/index; Plucked String wraps Karplus-Strong. `Voice`/`SynthEngine`/`SynthSource` are built with an `Instrument`, not raw oscillator/filter/envelope parameters.

#### 4. `audio_output` (The Rodio Bridge)
*   **Role:** Bridges your DSP code with the hardware via `rodio`.
*   **Implementation:** In `rodio`, you'll likely implement the `Source` trait for your `SynthEngine`. This trait requires a `next()` method that returns the next audio sample (`f32` or `i16`).
*   **Sync:** This is where the "pull" happens—the audio hardware asks for samples, and your synth must provide them in real-time.

#### 5. `keyboard_midi` / `virtual_midi` (Computer Keyboard as MIDI Controller)
*   **Role:** Lets the computer keyboard stand in for a hardware MIDI controller, without requiring one.
*   **`keyboard_midi`:** Pure mapping/edge-detection logic — a fixed one-octave key layout (`key_to_pitch`) and `KeyboardState`, which turns raw OS key-press/release events into `MidiEvent::NoteOn`/`NoteOff`, guarding against key-repeat and duplicate releases. No OS hooks here, so it's fully unit-testable.
*   **`virtual_midi`:** I/O glue — uses `rdev::listen` (on its own thread, since it blocks forever) to capture real key press/release events, and `midir`'s virtual-port support (`os::unix::VirtualOutput`) to expose a real MIDI output port ("Keyboard Synth") that other software (DAWs, `aconnect`/`amidi`, Audio MIDI Setup) can see and receive from. **Linux and macOS only** — `midir` doesn't support virtual ports on Windows; on Windows (or if virtual-port creation otherwise fails), the app degrades to keyboard-driving-the-local-synth-only, with a warning.

---

### Other modules/considerations?

1.  **Clock/Timing Module:** Even if you aren't building a sequencer, you might want to sync LFOs or effects to a specific BPM.
2.  **Parameters & State:** A way to store and change settings (like "Attack Time" or "Filter Cutoff") that is thread-safe, so the GUI (if you add one) can talk to the audio thread.
3.  **Effects Chain:** Even a simple Reverb or Delay can transform a flat sine wave into something professional.
4.  **Wavetable/Sample Support:** If you want to go beyond basic waveforms later, a module to load and playback small samples or wavetables.
5.  **Data flow:** Midi events, generated samples,and output sound should flow through the system via channels. 

### Suggested Module Hierarchy
```text
crates/
├── midi-core/           # no_std: engine, dsp, and MIDI event types
│   └── src/
│       ├── lib.rs       # #![cfg_attr(not(test), no_std)]; extern crate alloc
│       ├── midi.rs      # MidiEvent type and byte parsing/encoding
│       ├── instrument.rs # Named Instrument presets (technique + envelope + filter)
│       ├── engine/      # Voice management and polyphony logic
│       │   ├── mod.rs
│       │   └── voice.rs
│       └── dsp/         # Math and signal generation
│           ├── mod.rs   # OscillatorKind/Oscillator selector
│           ├── oscillator.rs
│           ├── karplus_strong.rs
│           ├── fm.rs
│           ├── envelope.rs
│           └── filter.rs
└── midi/                # std desktop app: hardware/keyboard MIDI input, audio output
    └── src/
        ├── main.rs          # Entry point, wire everything together
        ├── keyboard_midi.rs # Computer-keyboard -> MidiEvent mapping and edge-detection
        ├── virtual_midi.rs  # rdev listener + midir virtual output port bridge
        └── output.rs        # Rodio Source implementation, wraps midi_core::engine::SynthEngine
```

