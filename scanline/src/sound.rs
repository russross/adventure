//! TIA channel controls and persistent tone/noise generation, independent of SDL playback.

use std::f32::consts::TAU;
use std::num::NonZeroU32;

// Effects describe the game's finite envelopes; cadence follows the picture scheduler.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum SoundEffect {
    #[default]
    Victory,
    Roar,
    Swallow,
    DragonSlain,
    Drop,
    Pickup,
}

impl SoundEffect {
    const ALL: [Self; 6] = [
        Self::Victory,
        Self::Roar,
        Self::Swallow,
        Self::DragonSlain,
        Self::Drop,
        Self::Pickup,
    ];

    fn duration(self) -> u8 {
        match self {
            Self::Victory => 255,
            Self::Roar | Self::Swallow | Self::DragonSlain => 16,
            Self::Drop | Self::Pickup => 4,
        }
    }

    // Register widths apply after the envelope calculation, as in the cartridge.
    fn channel(self, remaining: u8) -> Channel {
        let (waveform, frequency, volume) = match self {
            Self::Victory => (remaining, remaining >> 3, remaining >> 1),
            Self::Roar => (
                if remaining & 1 != 0 { 3 } else { 8 },
                (remaining >> 2).wrapping_add(28),
                remaining,
            ),
            Self::Swallow => (6, remaining ^ 15, (remaining >> 1).wrapping_add(8)),
            Self::DragonSlain => (4, remaining ^ 31, remaining),
            Self::Drop => (6, remaining ^ 3, 5),
            Self::Pickup => (6, remaining, 5),
        };
        Channel {
            waveform: waveform & 15,
            frequency: frequency & 31,
            volume: volume & 15,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EffectCadence {
    EveryPicture,
    EveryTurn,
}

impl EffectCadence {
    fn pictures(self) -> usize {
        match self {
            Self::EveryPicture => 1,
            Self::EveryTurn => 3,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct EffectStep {
    effect: SoundEffect,
    remaining: u8,
    cadence: EffectCadence,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Channel {
    pub waveform: u8,
    pub frequency: u8,
    pub volume: u8,
}

#[derive(Default)]
pub struct Sound {
    pub channels: [Channel; 2],
    step: Option<EffectStep>,
    revision: u64,
}

impl Sound {
    // Updating a complete voice preserves oscillator phase at the next frame boundary.
    pub fn voice(&mut self, waveform: u8, frequency: u8, volume: u8) {
        self.channels[0] = Channel {
            waveform: waveform & 15,
            frequency: frequency & 31,
            volume: volume & 15,
        };
    }

    pub fn silence(&mut self) {
        for channel in &mut self.channels {
            channel.volume = 0;
        }
        if self.step.take().is_some() {
            self.revision = self.revision.wrapping_add(1);
        }
    }

    // Every envelope update is observable, including consecutive identical restarts.
    pub(crate) fn effect(&mut self, effect: SoundEffect, remaining: u8, cadence: EffectCadence) {
        self.channels[0] = effect.channel(remaining);
        self.step = Some(EffectStep {
            effect,
            remaining,
            cadence,
        });
        self.revision = self.revision.wrapping_add(1);
    }
}

#[derive(Default)]
struct Oscillator {
    divider: u8,
    pulse: u8,
    noise: u8,
    enabled: bool,
    noise_feedback: bool,
    noise_bit: bool,
    hold: bool,
}

impl Oscillator {
    // The divider supplies a clock to the pulse/noise circuit. A frequency change
    // compares against the running counter, which also wraps at its maximum.
    fn tick(&mut self, channel: Channel) {
        let mode = channel.waveform & 15;
        if self.enabled {
            self.noise_bit = self.noise & 1 != 0;
            self.hold = match mode & 3 {
                2 => self.noise & 30 != 2,
                3 => !self.noise_bit,
                _ => false,
            };

            // Feedback selects the short repeating noise, the longer noise, or
            // a periodic pulse. The zero-state terms let noise start at power-on.
            self.noise_feedback = if mode & 3 == 0 {
                (self.pulse ^ self.noise) & 1 != 0
                    || (self.noise == 0 && self.pulse == 10)
                    || mode & 12 == 0
            } else {
                ((self.noise >> 2) ^ self.noise) & 1 != 0 || self.noise == 0
            };
        }
        let frequency = channel.frequency & 31;
        self.enabled = self.divider == frequency;
        self.divider = if self.enabled || self.divider == 31 {
            0
        } else {
            self.divider + 1
        };

        // The second clock phase consumes the held feedback. Divider and mode
        // changes retain these latches rather than reseeding either circuit.
        if self.enabled {
            let pulse_feedback = match mode >> 2 {
                0 => ((self.pulse >> 1) ^ self.pulse) & 1 != 0 && self.pulse != 10 && mode & 3 != 0,
                1 => self.pulse & 8 == 0,
                2 => !self.noise_bit,
                3 => self.pulse & 2 == 0 && self.pulse & 14 != 0,
                _ => unreachable!(),
            };

            // Both feedback values use the old state. Holding the pulse circuit
            // gates its clock, while the noise circuit continues to advance.
            self.noise = (self.noise >> 1) | (u8::from(self.noise_feedback) << 4);
            if !self.hold {
                self.pulse = (!(self.pulse >> 1) & 7) | (u8::from(pulse_feedback) << 3);
            }
        }
    }

    fn level(&self, channel: Channel) -> f32 {
        f32::from((self.pulse & 1) * (channel.volume & 15))
    }
}

pub struct Synthesizer {
    oscillators: [Oscillator; 2],
    clock_period: u64,
    clock_phase: u64,
    previous_input: f32,
    previous_output: f32,
    dc_decay: f32,
}

impl Synthesizer {
    pub fn new(sample_rate: NonZeroU32) -> Self {
        Self {
            oscillators: [Oscillator::default(), Oscillator::default()],
            clock_period: u64::from(sample_rate.get()) * 114,
            clock_phase: 0,
            previous_input: 0.0,
            previous_output: 0.0,
            dc_decay: (-TAU * 20.0 / sample_rate.get() as f32).exp(),
        }
    }

    // Integrate the native NTSC clock over each output sample, retaining partial
    // clocks between buffers. Audio clocks never depend on CPU or render timing.
    pub fn render(&mut self, sound: &Sound, samples: &mut [f32]) {
        const NTSC_CLOCK: u64 = 3_579_545;
        for sample in samples {
            let mut remaining = NTSC_CLOCK;
            let mut integral = 0.0;
            while remaining != 0 {
                let span = remaining.min(self.clock_period - self.clock_phase);
                let level: f32 = self
                    .oscillators
                    .iter()
                    .zip(sound.channels)
                    .map(|(oscillator, channel)| oscillator.level(channel))
                    .sum();
                integral += level * span as f32;
                remaining -= span;
                self.clock_phase += span;
                if self.clock_phase == self.clock_period {
                    self.clock_phase = 0;
                    for (oscillator, channel) in self.oscillators.iter_mut().zip(sound.channels) {
                        oscillator.tick(channel);
                    }
                }
            }

            // The console mixes both channels to mono. Remove the constant voltage
            // component and leave headroom for volume changes and simultaneous tones.
            let input = integral / NTSC_CLOCK as f32 / 30.0 * 0.6;
            let output = input - self.previous_input + self.dc_decay * self.previous_output;
            self.previous_input = input;
            self.previous_output = output;
            *sample = output;
        }
    }
}

// Each cached envelope retains phase internally. Different effects use independent
// starting states so playback needs only a clip cursor, not a running TIA circuit.
struct EffectClip {
    samples: Vec<f32>,
    offsets: Vec<usize>,
}

impl EffectClip {
    fn new(effect: SoundEffect, cadence: EffectCadence, rate: NonZeroU32) -> Self {
        let mut synthesizer = Synthesizer::new(rate);
        let mut sound = Sound::default();
        let mut samples = Vec::new();
        let mut offsets = vec![0; usize::from(effect.duration())];
        for (step, remaining) in (0..effect.duration()).rev().enumerate() {
            offsets[usize::from(remaining)] = samples.len();
            sound.channels[0] = effect.channel(remaining);
            let end = (step + 1) * cadence.pictures() * rate.get() as usize / 60;
            let start = samples.len();
            samples.resize(end, 0.0);
            synthesizer.render(&sound, &mut samples[start..]);
        }

        // Preserve the DC filter's release, then taper its quiet endpoint to zero.
        let start = samples.len();
        let tail = (rate.get() as usize / 40).max(1);
        samples.resize(start + tail, 0.0);
        sound.silence();
        synthesizer.render(&sound, &mut samples[start..]);
        for (index, sample) in samples[start..].iter_mut().enumerate() {
            *sample *= (tail - index - 1) as f32 / tail as f32;
        }
        Self { samples, offsets }
    }
}

struct SoundBank {
    clips: [[EffectClip; 2]; 6],
}

impl SoundBank {
    fn new(rate: NonZeroU32) -> Self {
        Self {
            clips: SoundEffect::ALL.map(|effect| {
                [EffectCadence::EveryPicture, EffectCadence::EveryTurn]
                    .map(|cadence| EffectClip::new(effect, cadence, rate))
            }),
        }
    }

    fn clip(&self, step: EffectStep) -> &EffectClip {
        let cadence = match step.cadence {
            EffectCadence::EveryPicture => 0,
            EffectCadence::EveryTurn => 1,
        };
        &self.clips[step.effect as usize][cadence]
    }
}

#[derive(Clone, Copy)]
struct ClipCursor {
    step: EffectStep,
    position: usize,
}

// The audio thread only reads cached PCM. Main-thread commands retain the game's
// update boundaries; successive steps continue playback without restarting phase.
pub(crate) struct EffectPlayer {
    bank: SoundBank,
    cursor: Option<ClipCursor>,
    observed: Option<(u64, Option<EffectStep>)>,
    transition_samples: usize,
    transition_left: usize,
    transition_origin: f32,
    last_output: f32,
}

impl EffectPlayer {
    pub(crate) fn new(rate: NonZeroU32) -> Self {
        Self {
            bank: SoundBank::new(rate),
            cursor: None,
            observed: None,
            transition_samples: (rate.get() as usize / 500).max(2),
            transition_left: 0,
            transition_origin: 0.0,
            last_output: 0.0,
        }
    }

    pub(crate) fn update(&mut self, sound: &Sound, muted: bool) {
        if muted {
            if self.observed.take().is_some() {
                self.transition(None);
            }
            return;
        }
        if self
            .observed
            .is_some_and(|(revision, _)| revision == sound.revision)
        {
            return;
        }
        let previous = self.observed.and_then(|(_, step)| step);
        self.observed = Some((sound.revision, sound.step));
        let continuing = previous.zip(sound.step).is_some_and(|(old, new)| {
            old.effect == new.effect
                && old.cadence == new.cadence
                && old.remaining.checked_sub(1) == Some(new.remaining)
        });
        if continuing {
            return;
        }

        // Seek only for starts, interruptions, or cadence changes. Muting discards
        // the observed command so unmuting resumes at the current envelope step.
        let cursor = sound.step.and_then(|step| {
            self.bank
                .clip(step)
                .offsets
                .get(usize::from(step.remaining))
                .map(|&position| ClipCursor { step, position })
        });
        self.transition(cursor);
    }

    fn transition(&mut self, cursor: Option<ClipCursor>) {
        if self.cursor.is_none() && cursor.is_none() && self.last_output == 0.0 {
            return;
        }
        self.cursor = cursor;
        self.transition_origin = self.last_output;
        self.transition_left = self.transition_samples;
    }

    pub(crate) fn idle(&self) -> bool {
        self.cursor.is_none() && self.transition_left == 0
    }

    pub(crate) fn render(&mut self, output: &mut [f32]) {
        for sample in output {
            let raw = if let Some(cursor) = &mut self.cursor {
                let clip = self.bank.clip(cursor.step);
                let value = clip.samples[cursor.position];
                cursor.position += 1;
                if cursor.position == clip.samples.len() {
                    self.cursor = None;
                }
                value
            } else {
                0.0
            };

            // A two-millisecond ramp starts at the last delivered value. This
            // also handles interruption during a ramp without a new amplitude jump.
            let value = if self.transition_left != 0 {
                let weight =
                    (self.transition_left - 1) as f32 / (self.transition_samples - 1) as f32;
                self.transition_left -= 1;
                self.transition_origin * weight + raw * (1.0 - weight)
            } else {
                raw
            };
            self.last_output = value;
            *sample = value;
        }
    }
}

#[cfg(test)]
#[path = "tests/sound.rs"]
mod tests;
