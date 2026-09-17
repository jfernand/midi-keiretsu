#[derive(Debug, Clone, Copy, PartialEq)]
enum Stage {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

/// A linear ADSR (Attack/Decay/Sustain/Release) envelope generator.
///
/// `next_sample` returns an amplitude multiplier in `[0.0, 1.0]` that a
/// voice multiplies its oscillator output by.
pub struct Envelope {
    sample_rate: f32,
    attack: f32,
    decay: f32,
    sustain_level: f32,
    release: f32,
    stage: Stage,
    level: f32,
    release_rate: f32,
}

impl Envelope {
    pub fn new(
        sample_rate: f32,
        attack: f32,
        decay: f32,
        sustain_level: f32,
        release: f32,
    ) -> Self {
        Self {
            sample_rate,
            attack: attack.max(1e-6),
            decay: decay.max(1e-6),
            sustain_level: sustain_level.clamp(0.0, 1.0),
            release: release.max(1e-6),
            stage: Stage::Idle,
            level: 0.0,
            release_rate: 0.0,
        }
    }

    pub fn note_on(&mut self) {
        self.stage = Stage::Attack;
    }

    pub fn note_off(&mut self) {
        self.release_rate = self.level / (self.release * self.sample_rate);
        self.stage = Stage::Release;
    }

    pub fn is_active(&self) -> bool {
        self.stage != Stage::Idle
    }

    pub fn next_sample(&mut self) -> f32 {
        match self.stage {
            Stage::Idle => 0.0,
            Stage::Attack => {
                self.level += 1.0 / (self.attack * self.sample_rate);
                if self.level >= 1.0 {
                    self.level = 1.0;
                    self.stage = Stage::Decay;
                }
                self.level
            }
            Stage::Decay => {
                self.level -= (1.0 - self.sustain_level) / (self.decay * self.sample_rate);
                if self.level <= self.sustain_level {
                    self.level = self.sustain_level;
                    self.stage = Stage::Sustain;
                }
                self.level
            }
            Stage::Sustain => {
                self.level = self.sustain_level;
                self.level
            }
            Stage::Release => {
                self.level -= self.release_rate;
                if self.level <= 0.0 {
                    self.level = 0.0;
                    self.stage = Stage::Idle;
                }
                self.level
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idle_by_default_and_silent() {
        let mut env = Envelope::new(100.0, 1.0, 1.0, 0.5, 1.0);
        assert!(!env.is_active());
        assert_eq!(env.next_sample(), 0.0);
    }

    #[test]
    fn attack_ramps_up_to_full_level() {
        let mut env = Envelope::new(10.0, 1.0, 1.0, 0.5, 1.0); // 10 samples per second, 1s attack
        env.note_on();

        let mut last = 0.0;
        for _ in 0..10 {
            let sample = env.next_sample();
            assert!(
                sample >= last,
                "envelope should be non-decreasing during attack"
            );
            last = sample;
        }
        assert!((last - 1.0).abs() < 1e-3);
    }

    #[test]
    fn decay_settles_at_sustain_level() {
        let mut env = Envelope::new(10.0, 0.01, 1.0, 0.4, 1.0);
        env.note_on();
        for _ in 0..50 {
            env.next_sample();
        }
        assert!((env.next_sample() - 0.4).abs() < 1e-3);
    }

    #[test]
    fn sustain_holds_steady() {
        let mut env = Envelope::new(10.0, 0.01, 0.01, 0.6, 1.0);
        env.note_on();
        for _ in 0..50 {
            env.next_sample();
        }
        let a = env.next_sample();
        let b = env.next_sample();
        assert!((a - 0.6).abs() < 1e-3);
        assert!((b - 0.6).abs() < 1e-3);
    }

    #[test]
    fn release_decays_to_zero_and_goes_idle() {
        let mut env = Envelope::new(10.0, 0.01, 0.01, 0.5, 1.0);
        env.note_on();
        for _ in 0..50 {
            env.next_sample();
        }
        env.note_off();
        for _ in 0..10 {
            env.next_sample();
        }
        assert_eq!(env.next_sample(), 0.0);
        assert!(!env.is_active());
    }

    #[test]
    fn is_active_during_attack_decay_and_release() {
        let mut env = Envelope::new(10.0, 0.01, 0.01, 0.5, 0.01);
        env.note_on();
        env.next_sample();
        assert!(env.is_active());
        env.note_off();
        assert!(env.is_active());
    }
}
