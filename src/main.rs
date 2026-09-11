mod dsp;
mod engine;
mod keyboard_midi;
mod midi;
mod output;
mod virtual_midi;

use crate::dsp::OscillatorKind;
use crate::midi::MidiEvent;
use crate::output::SynthSource;
use crate::virtual_midi::VirtualMidiOut;
use crossbeam_channel::unbounded;
use midir::{Ignore, MidiInput, MidiInputConnection};
use rodio::{OutputStream, Sink};
use std::error::Error;
use std::io::{Write, stdin, stdout};

const KEYBOARD_LAYOUT_HINT: &str = "\
Computer-keyboard MIDI controller enabled. Key layout (one octave from C4):
  Black keys: S  D     G  H  J
  White keys: Z  X  C  V  B  N  M  ,";

fn main() -> Result<(), Box<dyn Error>> {
    let (midi_tx, midi_rx) = unbounded();

    // Kept alive for the rest of main's scope so the connection stays open.
    let _hardware_conn = connect_hardware_midi_input(midi_tx.clone())?;

    let keyboard_enabled =
        prompt_yes_no("Enable computer-keyboard MIDI controller (with virtual MIDI output)?")?;

    if _hardware_conn.is_none() && !keyboard_enabled {
        return Err(
            "No MIDI input available: no hardware port connected and keyboard input declined."
                .into(),
        );
    }

    if keyboard_enabled {
        let virtual_out = match VirtualMidiOut::new() {
            Ok(out) => {
                println!(
                    "Virtual MIDI port \"Keyboard Synth\" created; other MIDI software can connect to it."
                );
                Some(out)
            }
            Err(err) => {
                eprintln!(
                    "Could not create a virtual MIDI port ({err}); the keyboard will still drive this app's synth, \
                     but won't be visible to other MIDI software. Virtual ports are only supported on Linux and macOS."
                );
                None
            }
        };
        virtual_midi::spawn_keyboard_listener(midi_tx, virtual_out);
        println!("{KEYBOARD_LAYOUT_HINT}");
    }

    let oscillator_kind = prompt_oscillator_kind()?;

    let (_stream, stream_handle) = OutputStream::try_default()?;
    let sink = Sink::try_new(&stream_handle)?;

    let source = SynthSource::new(44100, 8, oscillator_kind, midi_rx);
    sink.append(source);

    println!("\nPress enter to exit ...");
    let mut input = String::new();
    stdin().read_line(&mut input)?;

    Ok(())
}

/// Looks for a hardware MIDI input port, letting the user pick one if
/// several are available, and connects it so incoming messages are
/// parsed and sent on `midi_tx`. Returns `Ok(None)` if there were
/// simply no hardware ports to use; the caller must keep the returned
/// connection alive for as long as it wants to keep receiving input.
fn connect_hardware_midi_input(
    midi_tx: crossbeam_channel::Sender<MidiEvent>,
) -> Result<Option<MidiInputConnection<()>>, Box<dyn Error>> {
    let mut midi_in = MidiInput::new("midir input")?;
    midi_in.ignore(Ignore::None);

    let in_ports = midi_in.ports();

    if in_ports.is_empty() {
        println!("No hardware MIDI input ports found.");
        return Ok(None);
    }

    let in_port = if in_ports.len() == 1 {
        println!(
            "Choosing the only available input port: {}",
            midi_in.port_name(&in_ports[0])?
        );
        &in_ports[0]
    } else {
        println!("\nAvailable input ports:");
        for (i, p) in in_ports.iter().enumerate() {
            println!("{}: {}", i, midi_in.port_name(p)?);
        }
        print!("Please select input port: ");
        stdout().flush()?;
        let mut input = String::new();
        stdin().read_line(&mut input)?;
        in_ports
            .get(input.trim().parse::<usize>()?)
            .ok_or("invalid port number")?
    };

    println!("\nOpening connection");
    let in_port_name = midi_in.port_name(in_port)?;

    // The callback will send MIDI events to the synth through the channel.
    let conn: MidiInputConnection<()> = midi_in.connect(
        in_port,
        "midir-read-input",
        move |_stamp, message, _| {
            let event = MidiEvent::parse(message);
            let _ = midi_tx.send(event);
        },
        (),
    )?;

    println!("Connection open, reading input from '{in_port_name}'.");
    Ok(Some(conn))
}

fn prompt_yes_no(question: &str) -> Result<bool, Box<dyn Error>> {
    print!("{question} [y/N]: ");
    stdout().flush()?;
    let mut input = String::new();
    stdin().read_line(&mut input)?;
    Ok(matches!(input.trim().to_lowercase().as_str(), "y" | "yes"))
}

fn prompt_oscillator_kind() -> Result<OscillatorKind, Box<dyn Error>> {
    println!("\nChoose a sound:");
    println!("1: Sine wave (default)");
    println!("2: Karplus-Strong plucked string");
    print!("Selection: ");
    stdout().flush()?;
    let mut input = String::new();
    stdin().read_line(&mut input)?;
    Ok(match input.trim() {
        "2" => OscillatorKind::KarplusStrong,
        _ => OscillatorKind::Sine,
    })
}
