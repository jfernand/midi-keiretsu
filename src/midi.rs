pub enum MidiEvent {
    NoteOn { pitch: u8, velocity: u8 },
    NoteOff { pitch: u8 },
    Other,
}

impl MidiEvent {
    pub fn parse(data: &[u8]) -> Self {
        if data.is_empty() {
            return MidiEvent::Other;
        }

        let status = data[0];
        let message_type = status & 0xF0;

        match message_type {
            0x90 => {
                if data.len() >= 3 {
                    let pitch = data[1];
                    let velocity = data[2];
                    if velocity > 0 {
                        MidiEvent::NoteOn { pitch, velocity }
                    } else {
                        MidiEvent::NoteOff { pitch }
                    }
                } else {
                    MidiEvent::Other
                }
            }
            0x80 => {
                if data.len() >= 2 {
                    let pitch = data[1];
                    MidiEvent::NoteOff { pitch }
                } else {
                    MidiEvent::Other
                }
            }
            _ => MidiEvent::Other,
        }
    }
}
