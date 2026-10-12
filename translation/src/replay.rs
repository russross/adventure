//! Headless host for semantic replay; controls are supplied at logical reads by the trace session.

use crate::hardware::FrameHost;
use crate::input::Controls;
use crate::sound::Sound;
use crate::tia::Frame;

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
    fn vertical_sync(
        &mut self,
        _frame: &Frame,
        _sound: &Sound,
    ) -> Result<Option<Controls>, String> {
        if self.remaining == 0 {
            return Ok(None);
        }
        self.remaining -= 1;
        Ok(Some(Controls::default()))
    }
}
