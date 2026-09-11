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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_note_on() {
        let event = MidiEvent::parse(&[0x90, 60, 100]);
        assert!(matches!(event, MidiEvent::NoteOn { pitch: 60, velocity: 100 }));
    }

    #[test]
    fn note_on_with_zero_velocity_is_note_off() {
        let event = MidiEvent::parse(&[0x90, 60, 0]);
        assert!(matches!(event, MidiEvent::NoteOff { pitch: 60 }));
    }

    #[test]
    fn parses_note_off() {
        let event = MidiEvent::parse(&[0x80, 60, 64]);
        assert!(matches!(event, MidiEvent::NoteOff { pitch: 60 }));
    }

    #[test]
    fn respects_channel_nibble() {
        // Note On, channel 5 (0x95) should still be recognized as NoteOn.
        let event = MidiEvent::parse(&[0x95, 60, 100]);
        assert!(matches!(event, MidiEvent::NoteOn { pitch: 60, velocity: 100 }));
    }

    #[test]
    fn empty_data_is_other() {
        assert!(matches!(MidiEvent::parse(&[]), MidiEvent::Other));
    }

    #[test]
    fn truncated_note_on_is_other() {
        assert!(matches!(MidiEvent::parse(&[0x90, 60]), MidiEvent::Other));
    }

    #[test]
    fn truncated_note_off_is_other() {
        assert!(matches!(MidiEvent::parse(&[0x80]), MidiEvent::Other));
    }

    #[test]
    fn unknown_status_is_other() {
        // Control Change (0xB0) is not handled yet.
        assert!(matches!(MidiEvent::parse(&[0xB0, 1, 127]), MidiEvent::Other));
    }
}
