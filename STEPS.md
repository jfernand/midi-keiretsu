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

## [2026-09-11] FM Synthesis and Named Instrument Presets

### Goal
Add FM (phase modulation) as a sixth synthesis technique, and — per
explicit request — stop exposing raw waveform techniques to the user
at all: let them pick a named *instrument* ("Electric Piano", "Bell",
"Plucked String", ...), with the underlying technique and envelope
as an implementation detail.

### Approach
1.  **FM oscillator:** Added `dsp/fm.rs`'s `FmOscillator` — a sine
    carrier whose phase is modulated by a sine modulator running at
    `modulator_ratio` times the carrier frequency, scaled by
    `modulation_index`. At index 0 it's mathematically identical to
    `SineOscillator`; nonzero index adds sidebands (non-integer
    ratios make them inharmonic), giving brighter or bell-like tones.
2.  **Wired into technique selection:** Added
    `OscillatorKind::Fm { modulator_ratio, modulation_index }` /
    `Oscillator::Fm` (`dsp/mod.rs`). Dropped the `Eq` derive on
    `OscillatorKind` since it now carries `f32` fields (`PartialEq`
    only, matching `Envelope`'s existing precedent).
3.  **Instrument presets:** Added `instrument.rs`'s `Instrument` enum
    — Sine Pad, Square Lead, Saw Bass, Flute, Plucked String, Electric
    Piano, Bell — each mapping to a fixed `OscillatorKind` *and* a
    tuned ADSR. Electric Piano (`ratio: 1.0, index: 2.0`) and Bell
    (`ratio: 3.5, index: 8.0`) are both FM under the hood, distinguished
    only by those two numbers; Plucked String wraps Karplus-Strong,
    with a near-instant attack and full sustain since the delay line
    itself already provides the natural decay.
4.  **Rewired the whole chain:** `Voice::new`, `SynthEngine::new`, and
    `SynthSource::new` now take an `Instrument` instead of an
    `OscillatorKind` (`Voice` no longer has its own hardcoded ADSR
    constants at all -- the instrument supplies both oscillator and
    envelope). `main()`'s prompt now lists instrument names built
    from `Instrument::ALL`, not raw waveform techniques.

### Key Decisions
*   `OscillatorKind`/`Oscillator` (the technique layer) stayed exactly
    where they were architecturally -- `Instrument` sits *above* them
    as a mapping layer, rather than replacing or duplicating them.
    Direct technique selection is still fully testable and usable
    internally; it's just no longer what the user sees.
*   Tests that looped a fixed sample count past "the release time"
    (`engine/mod.rs`, `engine/voice.rs`) were loosened from counts
    tied to one specific release constant (since different
    instruments now have different, sometimes much longer, releases
    -- Bell's is 1.5s) to a generously large, instrument-agnostic
    sample count (200,000) with an updated comment.

### Verification Steps
*   Ran `cargo test`: 78 tests passed (new `fm.rs`,
    `dsp::tests::fm_kind_builds_...`, and `instrument.rs` tests,
    including one iterating `Instrument::ALL` to check every
    instrument's oscillator stays bounded and every envelope reaches
    and leaves silence).
*   Ran `cargo clippy --all-targets`: no new warnings.
*   Ran `cargo fmt`.
*   Manual verification (not run in this session): `cargo run`,
    select each instrument in turn, and confirm Electric Piano and
    Bell sound like FM (brighter/more complex than a plain sine, Bell
    audibly inharmonic and long-ringing), and that all seven names
    print correctly in the selection prompt.
*   Verified end-to-end with a real MIDI note piped through ALSA
    (`aplaymidi` -> the app's "Midi Through" input): selected Electric
    Piano and Bell in separate runs, both processed the NoteOn/NoteOff
    and exited cleanly with no panics or errors. Could not confirm
    actual audible output from this sandboxed session (no working
    audio device here); the user manually confirmed both sound
    correct.

## [2026-09-11] Velocity Sensitivity

### Goal
Make `NoteOn` velocity (already parsed but previously discarded)
actually affect loudness, so playing harder produces louder notes.

### Approach
1.  **`Voice`:** Added a `velocity_gain: f32` field. `note_on` now
    takes `(pitch, velocity)` and sets `velocity_gain = velocity as
    f32 / 127.0`; `next_sample` multiplies the oscillator/envelope
    product by it.
2.  **`SynthEngine`:** `handle_event`'s `NoteOn` arm now destructures
    and forwards `velocity` to `voice.note_on(pitch, velocity)`
    instead of discarding it (`NoteOn { pitch, .. }` -> `NoteOn {
    pitch, velocity }`).

### Key Decisions
*   Linear velocity-to-gain mapping (`velocity / 127`), not a
    perceptual/dB curve -- matches the project's existing "basic"
    scope; a non-linear curve is a cheap follow-up if the linear one
    feels off.
*   Velocity scales output amplitude only, not envelope timing
    (e.g. a harder hit doesn't get a faster attack). Keeps the
    change small and orthogonal to the existing ADSR logic; some
    real synths do vary attack time with velocity, but that's a
    separate, larger change.
*   The computer-keyboard controller still sends a fixed velocity
    (100) per key press, since a typing keyboard has no way to sense
    how hard a key was struck -- velocity sensitivity is really only
    meaningful for hardware MIDI controllers with velocity-sensitive
    keys.

### Verification Steps
*   Ran `cargo test`: 82 tests passed, including new tests confirming
    velocity scales output proportionally
    (`velocity_scales_output_proportionally`), that lower velocity is
    quieter, that velocity 1 is much quieter than 127, and that
    `SynthEngine` forwards velocity from `NoteOn` through to output.
*   Ran `cargo clippy --all-targets`: no new warnings.
*   Ran `cargo fmt --check`: clean.
*   Manual verification (not run in this session): `cargo run` with a
    velocity-sensitive MIDI controller (or a DAW sending varying
    velocities), and confirm soft vs. hard playing is audibly
    quieter/louder.

## [2026-09-11] Low-Pass Filter

### Goal
Implement the last unaddressed `dsp` component from `ARCHITECTURE.md`
("A Low-Pass Filter (LPF) is standard for shaping the harmonics"),
and use it to give the harmonically-rich instruments (Square Lead,
Saw Bass) an actual subtractive-synthesis character instead of raw,
unfiltered waveforms.

### Approach
1.  **Filter:** Added `dsp/filter.rs`'s `LowPassFilter` — a one-pole
    (RC) low-pass. `alpha` (how far each sample moves from the
    previous output towards the current input) is derived from a
    cutoff frequency and the sample rate; a low cutoff smooths
    fast-changing input harder, a cutoff near Nyquist leaves the
    signal almost unchanged.
2.  **Per-instrument cutoff:** Added `Instrument::filter_cutoff_hz`
    and `build_filter`. Square Lead (6kHz), Saw Bass (1.2kHz), and
    Electric Piano (5kHz) get cutoffs low enough to noticeably tame
    their brightness; the rest (Sine Pad, Flute, Plucked String,
    Bell) get cutoffs high enough (8-18kHz) to stay essentially
    unfiltered, since there's little harshness to remove from those
    tones in the first place.
3.  **Voice:** Added a `filter: LowPassFilter` field. `next_sample`
    now filters the raw oscillator output before multiplying by the
    envelope and velocity gain: `oscillator -> filter -> envelope ->
    velocity_gain`, matching the conventional subtractive-synthesis
    signal chain.

### Key Decisions
*   A one-pole filter, not a resonant multi-pole design (e.g.
    Moog-style ladder filter with resonance) -- keeps the
    implementation and its behavior simple to reason about and test,
    consistent with the project's existing "basic" scope. A resonant
    filter with a cutoff/resonance envelope is a natural, larger
    follow-up.
*   Filter state is not reset on `note_on` -- it persists across
    notes like the oscillators already do (aside from Karplus-Strong,
    which deliberately reseeds). A one-pole filter converges within a
    few time constants, so any carryover from a previous note is
    inaudible in practice; this also matches how filters behave in
    real analog/hardware synths, which don't reset per note either.
*   Filter is applied unconditionally in the signal chain (every
    `Voice` has one) rather than being optional, since a
    near-Nyquist cutoff is already indistinguishable from no filter
    at all -- avoids an `Option<LowPassFilter>` for no real benefit.

### Verification Steps
*   Ran `cargo test`: 87 tests passed (new `filter.rs` tests --
    convergence to a constant input, no overshoot, and a low cutoff
    smoothing an alternating signal more than a high one -- plus
    `instrument.rs` tests confirming Saw Bass is filtered darker than
    Sine Pad and that every instrument's filter stays bounded).
*   Ran `cargo clippy --all-targets`: no new warnings.
*   Ran `cargo fmt --check`: clean.
*   Manual verification (not run in this session): `cargo run`,
    select Saw Bass and Square Lead, and confirm they sound warmer/
    less harsh than before this change (compare against Sine Pad or
    Bell, which should sound unchanged).

## [2026-09-17] Split into a midi-core / midi Workspace, Made midi-core no_std

### Goal
Pull the synth engine (MIDI event types, `Instrument`, everything
under `engine/` and `dsp/`) into its own `no_std` library crate, so
it can be reused outside this desktop app (e.g. embedded in another
project's build) without dragging in `rodio`/`midir`/`rdev`/std.

### Approach
Done in two commits, each independently buildable/testable:

1.  **Workspace split (no behavior change):** Moved `midi.rs`,
    `instrument.rs`, `engine/`, and `dsp/` into a new
    `crates/midi-core` library crate; left `main.rs`, `output.rs`,
    `keyboard_midi.rs`, and `virtual_midi.rs` in `crates/midi`, which
    now depends on `midi-core` via a path dependency. The root
    `Cargo.toml` became a workspace manifest. Only import paths
    changed in the app crate (`crate::midi::MidiEvent` ->
    `midi_core::midi::MidiEvent`, etc.) -- internal `crate::...`
    references inside the moved modules stayed valid unchanged, since
    they're all still in the same crate, just renamed. All 87 tests
    still passed, now split 77 (`midi-core`) / 10 (`midi`,
    `keyboard_midi.rs`'s tests).
2.  **no_std port:** Added `#![cfg_attr(not(test), no_std)]` plus
    `extern crate alloc` to `midi-core`'s `lib.rs`. This attribute
    form (rather than a bare `#![no_std]`) means `cargo test` still
    links `std` for the test harness regardless -- every existing
    `#[cfg(test)]` block, including `instrument.rs`'s
    `std::collections::HashSet`, needed zero changes. Only 6
    non-test call sites actually touched `std`:
    *   `oscillator.rs`/`fm.rs`: `std::f32::consts::PI` -> `core`;
        `.sin()` -> `libm::sinf`; `.fract()` -> a small `fract()`
        helper built on `libm::truncf` (exact for the non-negative,
        `<2.0` phase values these oscillators produce), shared
        between the two files.
    *   `karplus_strong.rs`: `Vec<f32>` -> `alloc::vec::Vec<f32>`;
        `.round()` -> `libm::roundf` (`.max()` stayed --
        `f32::max` is core-safe).
    *   `engine/mod.rs`: `Vec<Voice>` -> `alloc::vec::Vec<Voice>`.
    *   `engine/voice.rs`: `.powf()` -> `libm::powf`.
    *   `filter.rs`: PI import only, no transcendental calls.
    `envelope.rs`, `instrument.rs`, and `midi.rs` needed **no
    changes** -- they only use core-safe f32 methods
    (`.max()`/`.clamp()`) or pure byte-slice logic.

### Key Decisions
*   `#![cfg_attr(not(test), no_std)]` over a bare `#![no_std]` --
    the whole point was to make `no_std` cheap to adopt without
    rewriting the extensive existing test suite; this is the standard
    idiom for that.
*   A tiny in-crate `fract()` helper (subtraction against
    `libm::truncf`) instead of pulling in a bigger math-helpers
    dependency for one missing method.
*   No `std` feature flag on `midi-core` to let it opt back into
    `std` on host builds -- it's unconditionally `no_std`; a `std`
    binary (the `midi` app) can depend on a `no_std` library just
    fine, so there was no need for one.

### Verification Steps
*   Ran `cargo test --workspace`: all 87 tests passed unchanged
    (77 in `midi-core`, 10 in `midi`).
*   Ran `cargo clippy --workspace --all-targets`: no new warnings.
*   Ran `cargo fmt --check`: clean.
*   **Proved actual `no_std`-ness** (which `cargo test` alone cannot
    do, since the test harness always links `std`):
    `cargo check -p midi-core --target thumbv6m-none-eabi` (bare-metal
    ARM, no OS at all) and `--target wasm32-unknown-unknown` both
    succeeded.
*   Ran the full desktop app (`cargo run -p midi`) end-to-end after
    the split and confirmed it behaves identically to before (hardware
    port selection, keyboard-controller prompt, instrument selection,
    playback all unchanged).

## [2026-09-17] Renamed midi-core to wavesynth, Made It Publish-Ready

### Goal
Pick a crate name actually available on crates.io (checked: `midi`,
`synth`, and `synth-core` are all taken; `wavesynth`/`wavesynth-core`
were free), and finish the metadata/licensing needed to publish it.

### Approach
1.  **Rename:** `crates/midi-core` -> `crates/wavesynth`, package name
    `midi-core` -> `wavesynth`, and every `midi_core::` import in the
    `midi` app crate -> `wavesynth::`. No other changes.
2.  **Publish-readiness:** Added `description`, `license = "MIT OR
    Apache-2.0"`, `repository`, `readme`, `keywords`, and `categories`
    to `crates/wavesynth/Cargo.toml`; added a `README.md` for the
    crate; added `LICENSE-MIT` and `LICENSE-APACHE` at the repo root
    (the Rust ecosystem's de facto standard dual license).

### Key Decisions
*   Dual MIT/Apache-2.0 over a single license, matching near-universal
    Rust-ecosystem convention -- maximizes compatibility with
    downstream users' own licensing constraints.
*   License files at the repo root (not inside `crates/wavesynth/`),
    using the `license` field's SPDX identifier rather than
    `license-file` -- this is the standard layout for Rust workspaces
    where only some member crates are published, and doesn't require
    Cargo to pull files from outside the crate directory into the
    published package.
*   Left the `midi` app crate's package name alone even though `midi`
    is also taken on crates.io -- it isn't meant to be published (no
    metadata was added to it either), so the name collision is
    irrelevant; only path-dependency workspace members need locally
    unique names, not globally unique ones.

### Verification Steps
*   Ran `cargo test --workspace`: all 87 tests still passed.
*   Ran `cargo clippy --workspace --all-targets` and `cargo fmt
    --check`: clean.
*   Re-verified `no_std`-ness after the rename:
    `cargo check -p wavesynth --target thumbv6m-none-eabi` succeeded.
*   Ran `cargo package -p wavesynth --list` to confirm the packaged
    file set looks right (all `src/`, `Cargo.toml`, `README.md`, no
    stray files).
*   Ran `cargo publish -p wavesynth --dry-run`: packaging, the
    verification build, and everything short of the actual upload
    succeeded (the dry run correctly aborts before uploading). Did
    **not** actually publish -- that's a separate, irreversible step
    for the user to take when ready.
