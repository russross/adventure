//! TIA channel controls and persistent tone/noise generation, independent of SDL playback.

use std::f32::consts::TAU;
use std::num::NonZeroU32;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Channel {
    pub waveform: u8,
    pub frequency: u8,
    pub volume: u8,
}

#[derive(Default)]
pub struct Sound {
    pub channels: [Channel; 2],
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

#[cfg(test)]
#[path = "sound_tests.rs"]
mod tests;
