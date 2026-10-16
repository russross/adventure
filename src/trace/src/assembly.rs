//! Assemble component writes into committed whole-object position observations.

use std::collections::BTreeMap;

use crate::{Object, Observation, Position, Room};

#[derive(Clone, Copy, Debug)]
pub enum PositionField {
    Room(Room),
    X(u8),
    Y(u8),
}
#[derive(Default)]
struct Parts {
    room: Option<Room>,
    x: Option<u8>,
    y: Option<u8>,
}

// Partial stores retain their previous components until an explicit semantic commit boundary.
#[derive(Default)]
pub struct PositionAssembly {
    objects: BTreeMap<Object, Parts>,
}
impl PositionAssembly {
    pub fn write(&mut self, object: Object, field: PositionField) {
        let parts = self.objects.entry(object).or_default();
        match field {
            PositionField::Room(v) => parts.room = Some(v),
            PositionField::X(v) => parts.x = Some(v),
            PositionField::Y(v) => parts.y = Some(v),
        }
    }
    pub fn commit(&self, order: &[Object]) -> Result<Vec<Observation>, String> {
        order
            .iter()
            .filter_map(|object| {
                self.objects
                    .get(object)
                    .map(|parts| match (parts.room, parts.x, parts.y) {
                        (Some(room), Some(x), Some(y)) => {
                            Ok(Observation::Position(*object, Position { room, x, y }))
                        }
                        _ => Err(format!("incomplete position for {object}")),
                    })
            })
            .collect()
    }
}
