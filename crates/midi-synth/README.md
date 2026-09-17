# midi-synth

A MIDI-driven desktop synthesizer built on [`wavesynth`](https://crates.io/crates/wavesynth).
Play it from a hardware MIDI controller, or from your own computer
keyboard — which also exposes a real virtual MIDI port that other
software (DAWs, `aconnect`/`amidi`, Audio MIDI Setup) can see and
receive from.

```
cargo install midi-synth
midi-synth
```

## What it does

1. Looks for a hardware MIDI input port; picks it automatically if
   there's only one, or lets you choose.
2. Offers to enable a computer-keyboard MIDI controller. Saying yes
   tries to create a virtual MIDI output port named "Keyboard Synth"
   (visible to other MIDI software) and prints the key layout:
   ```
   Black keys: S  D     G  H  J
   White keys: Z  X  C  V  B  N  M  ,
   ```
3. Lets you pick an instrument (Sine Pad, Square Lead, Saw Bass,
   Flute, Plucked String, Electric Piano, Bell — see `wavesynth`'s
   README for what each one is built from).
4. Plays it through your default audio output.

## Platform notes

- Virtual MIDI ports are only supported on **Linux and macOS**
  (a `midir` limitation) — on Windows, or if virtual-port creation
  fails for any other reason, the keyboard still drives the synth
  locally, just without external MIDI visibility.
- The keyboard listener needs OS-level permission to capture key
  events: on Linux, your user needs to be in the `input` (or
  `plugdev`) group; on macOS, the terminal/binary needs Accessibility
  permission granted. If it can't start, you'll see a warning and the
  app keeps running on hardware MIDI input alone.

## License

Licensed under either of [Apache License, Version 2.0](../../LICENSE-APACHE)
or [MIT license](../../LICENSE-MIT) at your option.
