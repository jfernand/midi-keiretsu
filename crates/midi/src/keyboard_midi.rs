use midi_core::midi::MidiEvent;
use rdev::{Event, EventType, Key};
use std::collections::HashSet;

/// Maps a computer-keyboard key to a MIDI pitch, using a fixed
/// one-octave piano layout starting at C4 (60):
///
/// ```text
/// Black keys: S  D     G  H  J
/// White keys: Z  X  C  V  B  N  M  ,
///             C4 D4 E4 F4 G4 A4 B4 C5
/// ```
pub fn key_to_pitch(key: Key) -> Option<u8> {
    match key {
        Key::KeyZ => Some(60),  // C4
        Key::KeyS => Some(61),  // C#4
        Key::KeyX => Some(62),  // D4
        Key::KeyD => Some(63),  // D#4
        Key::KeyC => Some(64),  // E4
        Key::KeyV => Some(65),  // F4
        Key::KeyG => Some(66),  // F#4
        Key::KeyB => Some(67),  // G4
        Key::KeyH => Some(68),  // G#4
        Key::KeyN => Some(69),  // A4
        Key::KeyJ => Some(70),  // A#4
        Key::KeyM => Some(71),  // B4
        Key::Comma => Some(72), // C5
        _ => None,
    }
}

const VELOCITY: u8 = 100;

/// Tracks which pitches are currently held down so that OS key-repeat
/// doesn't retrigger NoteOn, and turns raw key events into `MidiEvent`s.
#[derive(Default)]
pub struct KeyboardState {
    held: HashSet<u8>,
}

impl KeyboardState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn on_key_event(&mut self, event: &Event) -> Option<MidiEvent> {
        match event.event_type {
            EventType::KeyPress(key) => {
                let pitch = key_to_pitch(key)?;
                if self.held.insert(pitch) {
                    Some(MidiEvent::NoteOn {
                        pitch,
                        velocity: VELOCITY,
                    })
                } else {
                    None
                }
            }
            EventType::KeyRelease(key) => {
                let pitch = key_to_pitch(key)?;
                if self.held.remove(&pitch) {
                    Some(MidiEvent::NoteOff { pitch })
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    fn key_event(event_type: EventType) -> Event {
        Event {
            time: SystemTime::now(),
            name: None,
            event_type,
        }
    }

    #[test]
    fn maps_white_keys_to_the_c_major_scale() {
        assert_eq!(key_to_pitch(Key::KeyZ), Some(60));
        assert_eq!(key_to_pitch(Key::KeyX), Some(62));
        assert_eq!(key_to_pitch(Key::KeyC), Some(64));
        assert_eq!(key_to_pitch(Key::KeyV), Some(65));
        assert_eq!(key_to_pitch(Key::KeyB), Some(67));
        assert_eq!(key_to_pitch(Key::KeyN), Some(69));
        assert_eq!(key_to_pitch(Key::KeyM), Some(71));
        assert_eq!(key_to_pitch(Key::Comma), Some(72));
    }

    #[test]
    fn maps_black_keys_to_sharps() {
        assert_eq!(key_to_pitch(Key::KeyS), Some(61));
        assert_eq!(key_to_pitch(Key::KeyD), Some(63));
        assert_eq!(key_to_pitch(Key::KeyG), Some(66));
        assert_eq!(key_to_pitch(Key::KeyH), Some(68));
        assert_eq!(key_to_pitch(Key::KeyJ), Some(70));
    }

    #[test]
    fn unmapped_key_is_none() {
        assert_eq!(key_to_pitch(Key::KeyQ), None);
    }

    #[test]
    fn press_emits_note_on() {
        let mut state = KeyboardState::new();
        let event = state.on_key_event(&key_event(EventType::KeyPress(Key::KeyZ)));
        assert_eq!(
            event,
            Some(MidiEvent::NoteOn {
                pitch: 60,
                velocity: VELOCITY
            })
        );
    }

    #[test]
    fn held_key_repeat_does_not_retrigger_note_on() {
        let mut state = KeyboardState::new();
        state.on_key_event(&key_event(EventType::KeyPress(Key::KeyZ)));
        let repeat = state.on_key_event(&key_event(EventType::KeyPress(Key::KeyZ)));
        assert_eq!(repeat, None);
    }

    #[test]
    fn release_emits_note_off() {
        let mut state = KeyboardState::new();
        state.on_key_event(&key_event(EventType::KeyPress(Key::KeyZ)));
        let event = state.on_key_event(&key_event(EventType::KeyRelease(Key::KeyZ)));
        assert_eq!(event, Some(MidiEvent::NoteOff { pitch: 60 }));
    }

    #[test]
    fn release_without_prior_press_is_ignored() {
        let mut state = KeyboardState::new();
        let event = state.on_key_event(&key_event(EventType::KeyRelease(Key::KeyZ)));
        assert_eq!(event, None);
    }

    #[test]
    fn duplicate_release_is_ignored() {
        let mut state = KeyboardState::new();
        state.on_key_event(&key_event(EventType::KeyPress(Key::KeyZ)));
        state.on_key_event(&key_event(EventType::KeyRelease(Key::KeyZ)));
        let duplicate = state.on_key_event(&key_event(EventType::KeyRelease(Key::KeyZ)));
        assert_eq!(duplicate, None);
    }

    #[test]
    fn two_keys_are_tracked_independently() {
        let mut state = KeyboardState::new();
        let z_on = state.on_key_event(&key_event(EventType::KeyPress(Key::KeyZ)));
        let x_on = state.on_key_event(&key_event(EventType::KeyPress(Key::KeyX)));
        assert_eq!(
            z_on,
            Some(MidiEvent::NoteOn {
                pitch: 60,
                velocity: VELOCITY
            })
        );
        assert_eq!(
            x_on,
            Some(MidiEvent::NoteOn {
                pitch: 62,
                velocity: VELOCITY
            })
        );

        let z_off = state.on_key_event(&key_event(EventType::KeyRelease(Key::KeyZ)));
        assert_eq!(z_off, Some(MidiEvent::NoteOff { pitch: 60 }));
        // X should remain held.
        let x_repeat = state.on_key_event(&key_event(EventType::KeyPress(Key::KeyX)));
        assert_eq!(x_repeat, None);
    }

    #[test]
    fn unmapped_key_press_and_release_are_ignored() {
        let mut state = KeyboardState::new();
        assert_eq!(
            state.on_key_event(&key_event(EventType::KeyPress(Key::KeyQ))),
            None
        );
        assert_eq!(
            state.on_key_event(&key_event(EventType::KeyRelease(Key::KeyQ))),
            None
        );
    }
}
