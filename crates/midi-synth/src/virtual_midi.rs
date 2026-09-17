use crate::keyboard_midi::KeyboardState;
use crossbeam_channel::Sender;
use midir::os::unix::VirtualOutput;
use midir::{MidiOutput, MidiOutputConnection};
use thiserror::Error;
use wavesynth::midi::MidiEvent;

const VIRTUAL_PORT_NAME: &str = "Keyboard Synth";

#[derive(Error, Debug)]
pub enum VirtualMidiError {
    #[error("failed to initialize MIDI output: {0}")]
    Init(#[from] midir::InitError),
    #[error("failed to create virtual MIDI output port: {0}")]
    PortCreation(#[from] midir::ConnectError<MidiOutput>),
    #[error("failed to send MIDI message: {0}")]
    Send(#[from] midir::SendError),
}

/// A connection to a virtual MIDI output port that other software
/// (DAWs, `aconnect`/`amidi`, Audio MIDI Setup, ...) can see and
/// receive events from. Only available on Linux and macOS -- midir
/// does not support virtual ports on Windows.
pub struct VirtualMidiOut {
    conn: MidiOutputConnection,
}

impl VirtualMidiOut {
    pub fn new() -> Result<Self, VirtualMidiError> {
        let conn = MidiOutput::new(VIRTUAL_PORT_NAME)?.create_virtual(VIRTUAL_PORT_NAME)?;
        Ok(Self { conn })
    }

    pub fn send(&mut self, event: MidiEvent) -> Result<(), VirtualMidiError> {
        if let Some(bytes) = event.to_bytes() {
            self.conn.send(&bytes)?;
        }
        Ok(())
    }
}

/// Spawns a thread that listens for computer-keyboard key presses and
/// releases and turns them into `MidiEvent`s: each one is sent on
/// `midi_tx` (so this app's own synth engine plays it) and, if
/// `virtual_out` is present, also forwarded through the virtual MIDI
/// port so other software sees it too.
///
/// `rdev::listen` blocks the calling thread forever and has no clean
/// shutdown handle, so this spawns its own thread and returns
/// immediately; the listener thread lives until the process exits. If
/// the OS-level hook fails to start (e.g. missing Linux `input`-group
/// permission, or macOS Accessibility not granted), a warning is
/// printed from that thread rather than aborting the program -- the
/// caller has no synchronous way to detect this failure, since a
/// successful `listen` call never returns.
pub fn spawn_keyboard_listener(midi_tx: Sender<MidiEvent>, virtual_out: Option<VirtualMidiOut>) {
    std::thread::spawn(move || {
        let mut state = KeyboardState::new();
        let mut virtual_out = virtual_out;
        let result = rdev::listen(move |event| {
            if let Some(midi_event) = state.on_key_event(&event) {
                let _ = midi_tx.send(midi_event);
                if let Some(out) = virtual_out.as_mut()
                    && let Err(err) = out.send(midi_event)
                {
                    eprintln!("virtual MIDI send failed: {err}");
                }
            }
        });
        if let Err(err) = result {
            eprintln!(
                "keyboard listener failed to start ({err:?}); computer-keyboard MIDI input is unavailable. \
                 On Linux, make sure your user is in the 'input' group; on macOS, grant Accessibility permission."
            );
        }
    });
}
