### Proposed Architecture for MIDI-Driven Synthesizer

Your initial list covers the core pillars of a synthesizer. To build a robust and idiomatic Rust implementation using `rodio`, here is a refined breakdown of the modules and some additional components you'll likely need:

#### 1. `midi_input` (The Dispatcher)
*   **Role:** Reads MIDI events from a hardware port or a file.
*   **Functionality:** Translates raw MIDI bytes into high-level domain events (e.g., `NoteOn(pitch, velocity)`, `NoteOff(pitch)`, `ControlChange`).
*   **Implementation Tip:** Use a crate like `midir` for real-time input. It should probably run in its own thread or use a callback mechanism to avoid blocking the audio thread.

#### 2. `synth_engine` (The Core Logic)
*   **Role:** Manages the "Voices" and state.
*   **Components:**
    *   **Voice Management:** A polyphonic synth needs to manage multiple "Voices" (instances of your oscillators/envelopes). It needs logic to assign a new `NoteOn` to an available voice and release it on `NoteOff`.
    *   **Modulation Matrix:** (Optional but recommended) Connects LFOs (Low Frequency Oscillators) or Envelopes to parameters like pitch or filter cutoff.
*   **Data Flow:** Receives high-level events from `midi_input` and updates the state of active voices.

#### 3. `dsp` (Digital Signal Processing - The "Sound Maker")
*   **Role:** The math-heavy part that generates the actual samples.
*   **Oscillators:** Not just Sine, but also Square, Saw, and Triangle. Implementing harmonics often involves "Additive Synthesis" (stacking sines) or "Subtractive Synthesis" (filtering rich waveforms).
*   **Karplus-Strong:** A physical-modeling algorithm for plucked-string/percussive timbres — feed a short burst of noise into a delay line with feedback and a low-pass/averaging filter, and the resonant loop reads back as a decaying plucked string. See Kevin Karplus and Alex Strong, "Digital Synthesis of Plucked-String and Drum Timbres," *Computer Music Journal* 7(2), 1983 (paper: [`docs/papers/karplus-strong-1983.pdf`](docs/papers/karplus-strong-1983.pdf)).
*   **Envelopes (ADSR):** Attack, Decay, Sustain, and Release. This is crucial for making the sound feel like an instrument rather than just a constant beep.
*   **Filters:** A Low-Pass Filter (LPF) is standard for shaping the harmonics you mentioned.

#### 4. `audio_output` (The Rodio Bridge)
*   **Role:** Bridges your DSP code with the hardware via `rodio`.
*   **Implementation:** In `rodio`, you'll likely implement the `Source` trait for your `SynthEngine`. This trait requires a `next()` method that returns the next audio sample (`f32` or `i16`).
*   **Sync:** This is where the "pull" happens—the audio hardware asks for samples, and your synth must provide them in real-time.

---

### Other modules/considerations?

1.  **Clock/Timing Module:** Even if you aren't building a sequencer, you might want to sync LFOs or effects to a specific BPM.
2.  **Parameters & State:** A way to store and change settings (like "Attack Time" or "Filter Cutoff") that is thread-safe, so the GUI (if you add one) can talk to the audio thread.
3.  **Effects Chain:** Even a simple Reverb or Delay can transform a flat sine wave into something professional.
4.  **Wavetable/Sample Support:** If you want to go beyond basic waveforms later, a module to load and playback small samples or wavetables.
5.  **Data flow:** Midi events, generated samples,and output sound should flow through the system via channels. 

### Suggested Module Hierarchy
```text
src/
├── main.rs          # Entry point, wire everything together
├── midi.rs          # Input handling and event types
├── engine/          # Voice management and polyphony logic
│   ├── mod.rs
│   └── voice.rs
├── dsp/             # Math and signal generation
│   ├── mod.rs
│   ├── oscillator.rs
│   ├── envelope.rs
│   └── filter.rs
└── output.rs        # Rodio Source implementation
```

