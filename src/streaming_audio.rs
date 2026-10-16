//! Picture-clocked PCM generation with oscillator state retained for the host's lifetime.

use std::num::NonZeroU32;

use sdl2::AudioSubsystem;
use sdl2::audio::{AudioQueue, AudioSpecDesired};

use crate::sound::{PictureSynthesizer, Sound};

pub(crate) struct StreamingAudio {
    pub(crate) device: AudioQueue<f32>,
    synthesizer: PictureSynthesizer,
    sample_rate: u32,
    muted: bool,
}

impl StreamingAudio {
    pub(crate) fn new(audio: &AudioSubsystem) -> Result<Self, String> {
        let device = audio
            .open_queue(
                None,
                &AudioSpecDesired {
                    freq: Some(48_000),
                    channels: Some(1),
                    samples: Some(512),
                },
            )
            .map_err(|error| format!("cannot open audio output: {error}"))?;
        let sample_rate = u32::try_from(device.spec().freq)
            .ok()
            .and_then(NonZeroU32::new)
            .ok_or("invalid SDL audio sample rate")?;
        device.resume();
        Ok(Self {
            device,
            synthesizer: PictureSynthesizer::new(sample_rate),
            sample_rate: sample_rate.get(),
            muted: false,
        })
    }

    // Muting suppresses delivery, while synthesis and fractional clocks keep advancing.
    pub(crate) fn present(&mut self, sound: &Sound, muted: bool) -> Result<(), String> {
        if self.muted != muted {
            self.device.clear();
            if muted {
                self.device.pause();
            } else {
                self.device.resume();
            }
            self.muted = muted;
        }
        let samples = self.synthesizer.render(sound);

        // Bound stale queued output when simulation runs ahead of the real device.
        if self.device.size() > self.sample_rate / 60 * 3 * size_of::<f32>() as u32 {
            self.device.clear();
        }
        if !muted {
            self.device.queue_audio(samples)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sound::Synthesizer;

    #[test]
    #[ignore = "requires an SDL audio driver"]
    fn queued_audio_preserves_phase_through_silence_effect_changes_and_mute() {
        let sdl = sdl2::init().unwrap();
        let mut audio = StreamingAudio::new(&sdl.audio().unwrap()).unwrap();
        let mut reference = Synthesizer::new(NonZeroU32::new(audio.sample_rate).unwrap());
        let mut remainder = 0;
        let mut sound = Sound::default();
        for picture in 0..240 {
            if picture % 31 == 0 {
                sound.voice((picture % 16) as u8, (picture % 32) as u8, 9);
            }
            if picture % 31 == 15 {
                sound.silence();
            }
            let muted = (53..116).contains(&picture);
            audio.present(&sound, muted).unwrap();
            remainder += audio.sample_rate;
            let mut expected = vec![0.0; (remainder / 60) as usize];
            remainder %= 60;
            reference.render(&sound, &mut expected);
            assert_eq!(audio.synthesizer.samples(), expected, "picture {picture}");
            if muted {
                assert_eq!(audio.device.size(), 0);
                assert_eq!(audio.device.status(), sdl2::audio::AudioStatus::Paused);
            }
        }
        assert_eq!(audio.device.status(), sdl2::audio::AudioStatus::Playing);
    }

    #[test]
    #[ignore = "requires an SDL audio driver"]
    fn backlog_replacement_discards_delivery_without_restarting_synthesis() {
        let sdl = sdl2::init().unwrap();
        let mut audio = StreamingAudio::new(&sdl.audio().unwrap()).unwrap();
        // Freeze the consumer so queue sizes exercise the policy deterministically.
        audio.device.pause();
        let mut reference = PictureSynthesizer::new(NonZeroU32::new(audio.sample_rate).unwrap());
        let mut sound = Sound::default();
        sound.voice(8, 7, 9);
        let mut queued_samples = 0;
        for _ in 0..9 {
            let expected = reference.render(&sound);
            if queued_samples > audio.sample_rate / 60 * 3 {
                queued_samples = 0;
            }
            audio.present(&sound, false).unwrap();
            queued_samples += expected.len() as u32;
            assert_eq!(audio.synthesizer.samples(), expected);
            assert_eq!(
                audio.device.size(),
                queued_samples * size_of::<f32>() as u32
            );
        }
    }
}
