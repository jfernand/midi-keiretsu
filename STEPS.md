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

## [2026-09-11] Computer-Keyboard MIDI Controller

### Goal
Let the computer keyboard emulate a MIDI controller: typing on it
plays this synth directly, and also exposes a real virtual MIDI port
that other software (DAWs, `aconnect`/`amidi`, Audio MIDI Setup) can
see and receive from — not just an internal shortcut.

### Approach
1.  **Encoding:** Added `MidiEvent::to_bytes` (`midi.rs`) to encode
    `NoteOn`/`NoteOff` back into raw channel-0 MIDI bytes, and derived
    `Clone, Copy, PartialEq, Eq` on `MidiEvent` so one event can be
    fanned out to two consumers (this app's engine and the virtual
    port). Took `self` by value per clippy's `wrong_self_convention`
    since `MidiEvent` is `Copy`.
2.  **Keyboard mapping:** Added `keyboard_midi.rs` — a pure
    `key_to_pitch` function (fixed one-octave layout: `Z X C V B N M ,`
    for white keys from C4, `S D G H J` for the black keys above them)
    and `KeyboardState`, which tracks currently-held pitches to turn
    raw `rdev` press/release events into exactly one `NoteOn`/`NoteOff`
    per physical press/release (ignoring OS key-repeat and stray
    duplicate releases). No OS hooks in this module, so it's fully
    unit-tested with synthetic `rdev::Event` values.
3.  **Virtual MIDI bridge:** Added `virtual_midi.rs` — `VirtualMidiOut`
    wraps a `midir` virtual output port ("Keyboard Synth"), and
    `spawn_keyboard_listener` runs `rdev::listen` on its own thread
    (it blocks forever and has no shutdown handle), feeding events
    through `KeyboardState` and sending each resulting `MidiEvent` on
    both the app's `midi_tx` channel and, if present, the virtual port.
4.  **Wiring:** `main.rs` no longer hard-errors when no hardware MIDI
    port exists — it now prompts to enable the keyboard controller
    instead, only erroring if both are unavailable. Enabling it tries
    to create the virtual port, warns and continues without one on
    failure (e.g. Windows, or a Linux/macOS permission issue), then
    spawns the listener and prints the key-layout hint.
5.  **Dependency:** Added `rdev = "0.5.3"` (needed for genuine OS-level
    key-press/key-release events — terminal raw-mode release events
    are unreliable, especially broken on Linux terminals). Added it,
    plus the already-in-use `midir` and `crossbeam-channel`, to
    `AGENTS.md`'s "Pre-approved dependencies" list, which had gone
    stale.

### Key Decisions
*   Octave-shift keys were deliberately deferred — one fixed octave
    ships first to keep the change small; a natural follow-up.
*   Virtual-port creation and keyboard-listener start failures degrade
    gracefully (a printed warning) rather than aborting, so the app
    still runs keyboard-to-local-synth even on Windows or without the
    right OS permissions.
*   Skipped a `KeyboardListenerError` (`thiserror`) type: `rdev::listen`'s
    failure surfaces asynchronously inside its own thread and is never
    propagated as a `Result` to a caller, so wrapping it would just be
    unused ceremony — it's logged directly with `{:?}` instead.
    `VirtualMidiError` *is* a real `thiserror` type, since
    `VirtualMidiOut::new`/`send` do return `Result`s to callers.

### Verification Steps
*   Ran `cargo test`: 46 tests passed (new `midi.rs` encoding/round-trip
    tests and `keyboard_midi.rs` mapping/edge-detection tests).
*   Ran `cargo clippy --all-targets`: no new warnings.
*   Ran `cargo fmt`.
*   Manual verification (not run in this session — requires a real
    keyboard/OS session and, for the virtual-port check, another MIDI
    app):
    1.  Run the app, opt into keyboard mode, confirm the key-layout
        hint prints.
    2.  Press/hold/release mapped keys; confirm correct note on/off
        audio with no repeat-triggering while a key is held.
    3.  Linux: `aconnect -l` / `amidi -l` in another terminal while the
        app runs should show a "Keyboard Synth" port.
    4.  macOS: Audio MIDI Setup / a DAW's MIDI input list should show
        the same port.
    5.  Confirm hardware MIDI input still works unaffected when
        keyboard mode is declined.

## [2026-09-11] Karplus-Strong Oscillator

### Goal
Implement the Karplus-Strong plucked-string algorithm (already on the
`ARCHITECTURE.md` roadmap, paper vendored at
`docs/papers/karplus-strong-1983.pdf`) as a second, selectable
oscillator alongside the existing sine wave.

### Approach
1.  **Algorithm:** Added `dsp/karplus_strong.rs`. `set_frequency`
    "plucks" the string: it reseeds a delay line (length =
    `sample_rate / frequency`) with noise from a small in-house
    xorshift32 PRNG. `next_sample` reads the current sample, writes
    back the average of it and its neighbor (a lossy averaging
    filter), and advances — this is what makes the tone naturally
    decay to silence.
2.  **Selection:** Added `OscillatorKind` (`Sine` | `KarplusStrong`)
    and an `Oscillator` enum in `dsp/mod.rs` that dispatches
    `set_frequency`/`next_sample` to whichever concrete oscillator a
    voice was built with, so `Voice` no longer depends on
    `SineOscillator` directly.
3.  **Wiring:** Threaded `OscillatorKind` through `Voice::new`,
    `SynthEngine::new`, and `SynthSource::new`, and added a prompt in
    `main()` ("Choose a sound: 1: Sine wave (default) / 2:
    Karplus-Strong plucked string") alongside the existing
    hardware/keyboard MIDI prompts.

### Key Decisions
*   Wrote a minimal xorshift32 PRNG instead of adding the `rand`
    crate — one call site (the noise burst) didn't justify a new
    dependency.
*   Used an enum (`Oscillator`) rather than a trait object for
    dispatch, consistent with the project's existing style (no `dyn`
    usage elsewhere) and avoiding heap allocation per voice.
*   Oscillator choice is a single global prompt at startup, not
    per-voice/per-note — matches the current scope; per-note timbre
    switching would need a bigger redesign (e.g. per-`NoteOn`
    instrument selection) that isn't needed yet.

### Verification Steps
*   Ran `cargo test`: 54 tests passed (new `karplus_strong.rs` and
    `dsp/mod.rs` oscillator-selection tests, plus a `Voice` test
    confirming bounded output with `OscillatorKind::KarplusStrong`).
*   Ran `cargo clippy --all-targets`: no new warnings.
*   Ran `cargo fmt --check`: clean.
*   Manual verification (not run in this session — requires a real
    audio/keyboard session): run the app (`cargo run`), select the
    Karplus-Strong option, and confirm it plays audible
    plucked-string-like notes that decay naturally, distinct from the
    sine wave.

## [2026-09-11] Square, Sawtooth, and Triangle Oscillators

### Goal
Fill out the remaining oscillators from `ARCHITECTURE.md`'s roadmap
("Not just Sine, but also Square, Saw, and Triangle") and make all
five waveforms selectable.

### Approach
1.  **Shared bookkeeping:** Factored `SineOscillator`'s
    frequency/sample-rate/phase fields and phase-advance logic out
    into a private `PhaseAccumulator` (`dsp/oscillator.rs`), since all
    four simple periodic oscillators only differ in how they turn a
    `0.0..1.0` phase into a sample. `SineOscillator`'s public API is
    unchanged.
2.  **New oscillators:** Added `SquareOscillator` (bipolar, 50% duty:
    `phase < 0.5`), `SawtoothOscillator` (linear ramp `2*phase - 1`),
    and `TriangleOscillator` (`0 -> 1` over the first quarter-period,
    down to `-1` by three-quarters, back to `0`).
3.  **Selection:** Added `Square`, `Sawtooth`, and `Triangle`
    variants to `OscillatorKind`/`Oscillator` (`dsp/mod.rs`)
    alongside the existing `Sine`/`KarplusStrong`, and extended
    `main()`'s sound-selection prompt to list all five.

### Key Decisions
*   Generalized the existing "completes one period" oscillator test
    into a shared `assert_returns_to_start_after_one_period` helper
    (100Hz at 44.1kHz gives an exact 441-sample period), reused by
    all four phase-accumulator oscillators. Needed a small tolerance
    (`1e-3`) rather than exact equality for the continuous waveforms
    (sine/sawtooth/triangle), since `fract()` accumulates tiny
    floating-point drift over 441 calls — square is insensitive to
    this since it only compares `phase < 0.5`.
*   Kept the waveforms unfiltered/aliased (no band-limiting), matching
    the project's existing "basic" scope; a low-pass filter to tame
    the raw square/sawtooth harmonics is the next natural `dsp` item.

### Verification Steps
*   Ran `cargo test`: 67 tests passed.
*   Ran `cargo clippy --all-targets`: no new warnings.
*   Ran `cargo fmt`.
*   Manual verification (not run in this session): `cargo run`,
    select each of the square/sawtooth/triangle options in turn, and
    confirm each sounds distinctly different (buzzier/brighter than
    the sine wave) with correct pitch.
