mod dsp;
mod engine;
mod keyboard_midi;
mod midi;
mod output;
mod virtual_midi;

use crate::midi::MidiEvent;
use crate::output::SynthSource;
use crossbeam_channel::unbounded;
use midir::{Ignore, MidiInput};
use rodio::{OutputStream, Sink};
use std::error::Error;
use std::io::{stdin, stdout, Write};

fn main() -> Result<(), Box<dyn Error>> {
    let mut midi_in = MidiInput::new("midir input")?;
    midi_in.ignore(Ignore::None);

    let in_ports = midi_in.ports();
    
    if in_ports.is_empty() {
        return Err("No MIDI input ports found. Please connect a MIDI device.".into());
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

    let (midi_tx, midi_rx) = unbounded();

    // The callback will send MIDI events to the synth through the channel
    let _conn = midi_in.connect(
        in_port,
        "midir-read-input",
        move |_stamp, message, _| {
            let event = MidiEvent::parse(message);
            let _ = midi_tx.send(event);
        },
        (),
    )?;

    println!("Connection open, reading input from '{}' (press enter to exit) ...", in_port_name);

    let (_stream, stream_handle) = OutputStream::try_default()?;
    let sink = Sink::try_new(&stream_handle)?;

    let source = SynthSource::new(44100, 8, midi_rx);
    sink.append(source);

    let mut input = String::new();
    stdin().read_line(&mut input)?;

    Ok(())
}
