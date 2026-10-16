//! Semantic Adventure traces, independent of a game's implementation and host.

mod assembly;
mod machine;
mod text;
mod types;

pub use assembly::{PositionAssembly, PositionField};
pub use machine::{Machine, Session};
pub use types::*;

#[cfg(test)]
mod tests;
