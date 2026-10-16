//! Adventure with explicit game workflows and complete-picture hardware.

pub mod adventure;
#[cfg(feature = "sdl")]
pub mod hardware;
pub mod input;
pub mod recording;
pub mod replay;
pub mod runtime;
pub mod sound;
#[cfg(feature = "sdl")]
mod streaming_audio;
#[cfg(feature = "validation")]
pub mod validation;
pub mod video;
