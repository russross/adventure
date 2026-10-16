//! Replay hosts; input sessions supply controls at logical gameplay reads.

use crate::input::Controls;
use crate::runtime::FrameHost;
use crate::sound::Sound;
use crate::video::Frame;

pub struct ReplayHost {
    remaining: u64,
}
impl ReplayHost {
    pub fn new(frames: u64) -> Self {
        Self { remaining: frames }
    }
}
impl FrameHost for ReplayHost {
    fn sample_controls(&mut self) -> Result<Option<Controls>, String> {
        Ok(Some(Controls::default()))
    }

    // Presentation advances virtual time without opening a window, producing audio, or sleeping.
    fn present(&mut self, _frame: &Frame, _sound: &Sound) -> Result<Option<Controls>, String> {
        if self.remaining == 0 {
            return Ok(None);
        }
        self.remaining -= 1;
        Ok(Some(Controls::default()))
    }
}

// Playback keeps the recorded frame boundary while delegating pictures, audio,
// pacing, and window shortcuts to the presentation host.
pub struct PlaybackHost<H> {
    host: H,
    clock: ReplayHost,
    interrupted: bool,
}

impl<H: FrameHost> PlaybackHost<H> {
    pub fn new(host: H, frames: u64) -> Self {
        Self {
            host,
            clock: ReplayHost::new(frames),
            interrupted: false,
        }
    }

    pub fn interrupted(&self) -> bool {
        self.interrupted
    }
}

impl<H: FrameHost> FrameHost for PlaybackHost<H> {
    fn sample_controls(&mut self) -> Result<Option<Controls>, String> {
        self.interrupted = self.host.sample_controls()?.is_none();
        Ok((!self.interrupted).then_some(Controls::default()))
    }

    // Live gameplay keys cannot supply defaults for inputs absent from the trace.
    // Stop before presenting the extra boundary used to finish verification.
    fn present(&mut self, frame: &Frame, sound: &Sound) -> Result<Option<Controls>, String> {
        if self.clock.present(frame, sound)?.is_none() {
            return Ok(None);
        }
        self.interrupted = self.host.present(frame, sound)?.is_none();
        Ok((!self.interrupted).then_some(Controls::default()))
    }
}
