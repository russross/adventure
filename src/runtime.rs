//! Host boundaries and optional session services, separate from game rules.

use std::cell::RefCell;

use adventure_trace::{Control, Session};

use crate::input::Controls;
use crate::recording::Recording;
use crate::sound::Sound;
#[cfg(feature = "validation")]
use crate::validation::Validation;
use crate::video::Frame;

pub trait FrameHost {
    /// Sample controls without presenting, generating audio, or advancing the frame limit.
    fn sample_controls(&mut self) -> Result<Option<Controls>, String>;

    /// Process input and present picture and sound; None means the host is stopping.
    fn present(&mut self, frame: &Frame, sound: &Sound) -> Result<Option<Controls>, String>;
}

impl<H: FrameHost> FrameHost for &mut H {
    fn sample_controls(&mut self) -> Result<Option<Controls>, String> {
        (**self).sample_controls()
    }

    fn present(&mut self, frame: &Frame, sound: &Sound) -> Result<Option<Controls>, String> {
        (**self).present(frame, sound)
    }
}

// A validated session owns its trace and oracle together. Replacing the trace ends that pairing.
enum TraceSession {
    Observed(RefCell<Session>),
    #[cfg(feature = "validation")]
    Validated {
        trace: RefCell<Session>,
        validator: Box<Validation>,
    },
}

#[derive(Default)]
pub struct Runtime {
    trace: Option<TraceSession>,
    pub recording: Option<RefCell<Recording>>,
}

impl Runtime {
    pub fn set_trace(&mut self, session: Session) {
        self.trace = Some(TraceSession::Observed(RefCell::new(session)));
    }

    pub fn trace(&self) -> Option<&RefCell<Session>> {
        self.trace.as_ref().map(|session| match session {
            TraceSession::Observed(trace) => trace,
            #[cfg(feature = "validation")]
            TraceSession::Validated { trace, .. } => trace,
        })
    }

    #[cfg(feature = "validation")]
    pub fn enable_validation(&mut self) -> Result<(), String> {
        let Some(session) = self.trace.take() else {
            return Err("validation requires a trace session".into());
        };
        let trace = match session {
            TraceSession::Observed(trace) => trace,
            validated @ TraceSession::Validated { .. } => {
                self.trace = Some(validated);
                return Ok(());
            }
        };
        let result = Validation::attach(&mut trace.borrow_mut());
        match result {
            Ok(validator) => {
                self.trace = Some(TraceSession::Validated {
                    trace,
                    validator: Box::new(validator),
                })
            }
            Err(error) => {
                self.trace = Some(TraceSession::Observed(trace));
                return Err(error);
            }
        }
        Ok(())
    }

    // Recordings and semantic replay supply values only at a game's logical input read.
    pub(crate) fn input(&self, control: Control, live: u8) -> u8 {
        let live = self.recording.as_ref().map_or(live, |recording| {
            recording.borrow_mut().input(control, live)
        });
        self.trace()
            .map_or(live, |trace| trace.borrow_mut().input(control, live))
    }

    pub(crate) fn stopped_at_boot(&self) -> bool {
        self.recording
            .as_ref()
            .is_some_and(|recording| recording.borrow().stopped_at_boot())
            || self
                .trace()
                .is_some_and(|trace| trace.borrow().stopped_at_boot())
    }

    pub(crate) fn check(&self) -> Result<(), String> {
        if let Some(recording) = &self.recording {
            recording.borrow().check()?;
        }
        if let Some(trace) = self.trace() {
            trace.borrow().check()?;
        }
        Ok(())
    }

    // Only accepted presentation boundaries advance either session's clock.
    pub(crate) fn next_frame(&self) {
        if let Some(recording) = &self.recording {
            recording.borrow_mut().next_frame();
        }
        if let Some(trace) = self.trace() {
            trace.borrow_mut().next_frame();
        }
    }

    // Validation finishes preceding game work before the host accepts another picture.
    pub(crate) fn catch_up(&mut self) -> Result<(), String> {
        #[cfg(feature = "validation")]
        if let Some(TraceSession::Validated { trace, validator }) = &mut self.trace {
            validator.catch_up(trace.borrow().frame)?;
        }
        Ok(())
    }

    pub(crate) fn finish(&mut self) -> Result<(), String> {
        if let Some(recording) = &self.recording {
            recording.borrow_mut().finish()?;
        }
        if let Some(trace) = self.trace() {
            trace.borrow_mut().finish()?;
        }
        #[cfg(feature = "validation")]
        if let Some(TraceSession::Validated { trace, validator }) = &mut self.trace {
            validator.finish(trace.borrow().frame)?;
        }
        Ok(())
    }
}
