# Development Steps

## [2026-04-01] Initial Synthesizer Implementation

### Goal
Implement a basic MIDI-driven synthesizer with a sine wave engine and rodio output.

### Approach
1.  **MIDI Input:** Created a `midi` module to parse raw MIDI bytes into high-level `NoteOn` and `NoteOff` events using `midir`.
2.  **DSP Engine:**
    *   Implemented a `SineOscillator` for waveform generation.
    *   Created a `Voice` struct to manage individual note state and frequency calculation.
    *   Implemented a `SynthEngine` to manage polyphony (voice assignment and mixing).
3.  **Audio Output:** Implemented the `rodio::Source` trait for `SynthSource` to bridge the `SynthEngine` with the hardware.
4.  **Communication:** Used `crossbeam-channel` for thread-safe communication between the MIDI callback thread and the audio playback thread.

### Key Decisions
*   **Decoupling:** Separated MIDI parsing, DSP logic, and audio output into distinct modules (`midi`, `dsp`, `engine`, `output`).
*   **Polyphony:** Used a simple polyphonic voice management system (up to 8 voices) to allow multiple notes to be played simultaneously.
*   **Sample Rate:** Fixed the sample rate to 44.1kHz for consistent audio generation.

### Verification Steps
*   Ran `cargo check` to ensure code compiles and follows Rust safety rules.
*   The system is ready for manual testing with a MIDI controller.

## [2026-09-11] Unit Tests and Karplus-Strong Roadmap Addition

### Goal
Add unit test coverage for the existing modules (per the TDD guideline in
`AGENTS.md`) and extend the oscillator roadmap with the Karplus-Strong
plucked-string algorithm.

### Approach
1.  **Tests:** Added `#[cfg(test)]` modules in-file for `midi.rs`
    (message parsing), `dsp/oscillator.rs` (`SineOscillator` behavior),
    `engine/voice.rs` (pitch-to-frequency conversion and voice
    lifecycle), and `engine/mod.rs` (`SynthEngine` voice assignment,
    stealing, and mixing).
2.  **Roadmap:** Added Karplus-Strong to the oscillator list in
    `ARCHITECTURE.md` and vendored the original 1983 Karplus & Strong
    paper ("Digital Synthesis of Plucked-String and Drum Timbres") under
    `docs/papers/` for reference during implementation.

### Key Decisions
*   Tests live alongside the code they cover, as directed by `AGENTS.md`,
    rather than in a separate `tests/` integration suite.
*   The paper is stored in the repo rather than only linked, since it's
    the primary reference for the not-yet-implemented Karplus-Strong
    oscillator.

### Verification Steps
*   Ran `cargo test`: 23 tests passed.

## [2026-09-11] ADSR Envelope

### Goal
Give each voice an Attack/Decay/Sustain/Release amplitude envelope so
notes fade in/out instead of clicking on/off, per the `dsp` module
described in `ARCHITECTURE.md`.

### Approach
1.  **Envelope:** Added `dsp/envelope.rs` with a linear-segment ADSR
    state machine (`Idle -> Attack -> Decay -> Sustain -> Release ->
    Idle`). `note_off` computes a release rate from whatever level the
    envelope was at, so releasing early (e.g. mid-decay) still ramps to
    zero smoothly instead of jumping.
2.  **Voice integration:** `Voice` now owns an `Envelope` alongside its
    oscillator and multiplies each sample by the envelope's output.
    `Voice::is_active` became a method delegating to
    `Envelope::is_active`, since a voice must stay "active" through its
    release tail even after `note_off` — it no longer cuts off
    instantly.
3.  **Engine:** `SynthEngine` updated to call `voice.is_active()`
    instead of reading a field; voice assignment/stealing logic is
    otherwise unchanged.

### Key Decisions
*   Fixed default envelope times (10ms attack, 100ms decay, 70%
    sustain, 200ms release) live as constants in `voice.rs` rather than
    being configurable yet — no UI/parameter system exists to expose
    them.
*   Linear segments (not exponential) for simplicity; matches the
    "basic" scope of the current implementation.

### Verification Steps
*   Ran `cargo test`: 31 tests passed.
*   Ran `cargo clippy --all-targets`: no new warnings (only the
    pre-existing unused `sample_rate`/`velocity` fields).
