//! World simulation, picture scheduling, and construction of complete scanlines.

mod actors;
mod assets;
mod effects;
mod interactions;
mod lifecycle;
mod model;
mod movement;
mod rendering;

use model::PreparedDisplay;
pub use model::{
    Bat, Dragon, DragonId, DragonState, Game, GateId, GateState, Gates, ObjectId, Position,
    RecordId, Room, World,
};

use crate::input::Controls;
use crate::sound::Sound;
use crate::video::Video;

#[derive(Default)]
pub struct Adventure {
    pub world: World,
    pub video: Video,
    pub sound: Sound,
    pub controls: Controls,
    display: PreparedDisplay,
    phase: PicturePhase,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum PicturePhase {
    #[default]
    Player,
    BatAndGates,
    RemainingActors,
}

#[cfg(test)]
mod tests;
