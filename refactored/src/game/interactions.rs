use super::Adventure;
use super::assets::{CASTLE_INTERIORS, GATE_KEYS, GATE_OBJECTS, MAGNET_REACTIONS, OBJECT_CATALOG};
use super::model::{
    ColorChoice, GateId, GateState, ObjectId, Position, PositionSource, RecordId, SoundEffect,
};
use crate::input::Direction;

impl Adventure {
    // Fire drops an item; the last rendered contacts determine pickup candidates.
    pub(super) fn handle_pickup_and_drop(&mut self) {
        let newly_pressed = self.controls.left.fire && !self.world.fire_was_pressed;
        self.world.fire_was_pressed = self.controls.left.fire;
        if newly_pressed && self.world.held != ObjectId::Empty {
            self.world.held = ObjectId::Empty;
            self.world.sound_effect = SoundEffect::Drop;
            self.world.sound_remaining = 4;
        }
        let touched = if self.video.contacts.player_objects[0] {
            self.world.first_visible
        } else if self.video.contacts.player_objects[1] {
            self.world.second_visible
        } else {
            return;
        };
        if matches!(
            touched,
            ObjectId::Light
                | ObjectId::GoldGate
                | ObjectId::WhiteGate
                | ObjectId::BlackGate
                | ObjectId::Credits
                | ObjectId::GameNumber
                | ObjectId::Rhindle
                | ObjectId::Yorgle
                | ObjectId::Grundle
        ) || self.find_object_position(touched).room != self.world.player.room
        {
            return;
        }
        let PositionSource::Record(record) = OBJECT_CATALOG[touched as usize].position else {
            return;
        };
        if touched != self.world.held {
            self.world.sound_effect = SoundEffect::Pickup;
            self.world.sound_remaining = 4;
        }
        self.world.held = touched;
        self.step_object(record, self.world.joystick_direction, 6);
        let position = self.world.position(record);
        self.world.held_offset_x = position.x.wrapping_sub(self.world.player.x);
        self.world.held_offset_y = position.y.wrapping_sub(self.world.player.y);
    }

    // Keep the carried offset, then resolve boundaries without an extra directional step.
    pub(super) fn move_held_object(&mut self) {
        if self.world.held == ObjectId::Empty {
            return;
        }
        let PositionSource::Record(record) = OBJECT_CATALOG[self.world.held as usize].position
        else {
            return;
        };
        *self.world.position_mut(record) = Position::new(
            self.world.player.room,
            self.world.player.x.wrapping_add(self.world.held_offset_x),
            self.world.player.y.wrapping_add(self.world.held_offset_y),
        );
        self.move_object_through_world(record, Direction::default(), 0);
    }

    // Only the displayed slots can report player contact.
    pub(super) fn is_touching_player(&self, object: ObjectId) -> bool {
        if object == self.world.first_visible {
            self.video.contacts.player_objects[0]
        } else if object == self.world.second_visible {
            self.video.contacts.player_objects[1]
        } else {
            false
        }
    }

    // Return the other member of a displayed object pair that touched.
    pub(super) fn find_touching_object(&self, object: ObjectId) -> ObjectId {
        if !self.video.contacts.objects_touch {
            return ObjectId::Empty;
        }
        if object == self.world.first_visible {
            self.world.second_visible
        } else if object == self.world.second_visible {
            self.world.first_visible
        } else {
            ObjectId::Empty
        }
    }

    // Gates process black, white, then gold; entry precedes each gate's animation.
    pub(super) fn update_castle_gates(&mut self) {
        for gate in [GateId::Black, GateId::White, GateId::Gold] {
            let index = gate.index();
            let touched = self.find_touching_object(GATE_OBJECTS[index]);
            if touched == GATE_KEYS[index] {
                self.world.gates.set(
                    gate,
                    GateState::from_value(self.world.gates.get(gate).value().wrapping_add(1)),
                );
            }
            if self.world.gates.get(gate) != GateState::Closed {
                let record = if self.is_touching_player(GATE_OBJECTS[index]) {
                    self.world.gates.set(gate, GateState::Open);
                    Some(RecordId::Player)
                } else if touched != ObjectId::Empty {
                    match OBJECT_CATALOG[touched as usize].position {
                        PositionSource::Record(record) => Some(record),
                        PositionSource::Fixed(_) => None,
                    }
                } else {
                    None
                };
                if let Some(record) = record {
                    self.world.position_mut(record).room = CASTLE_INTERIORS[index];
                    self.world.position_mut(record).y = 16;
                }
            }
            if let GateState::Moving(value) = self.world.gates.get(gate) {
                let value = value.wrapping_add(1);
                self.world.gates.set(
                    gate,
                    if value == 56 {
                        GateState::Open
                    } else {
                        GateState::from_value(value)
                    },
                );
            }
        }
    }

    // The selected item moves toward the magnet's temporarily adjusted target.
    pub(super) fn update_magnet(&mut self) {
        self.world.magnet.y = self.world.magnet.y.wrapping_sub(8);
        if let Some(chosen) = self.choose_reaction(&MAGNET_REACTIONS, None) {
            self.move_object_through_world(chosen.reaction.from, chosen.direction, 1);
        }
        self.world.magnet.y = self.world.magnet.y.wrapping_add(8);
    }

    // Dark rooms center the light on the player and clamp its horizontal reach.
    pub(super) fn update_dark_room_light(&mut self) {
        if !matches!(
            self.find_room_record(self.world.player.room).color,
            ColorChoice::Fixed(8)
        ) {
            self.world.light.y = 0;
            return;
        }
        self.world.light.room = self.world.player.room;
        self.world.light.x = self.world.player.x.wrapping_sub(14);
        self.world.light.y = self.world.player.y.wrapping_add(14);
        if self.world.light.x >= 240 {
            self.world.light.x = 1;
        } else if self.world.light.x >= 130 {
            self.world.light.x = 129;
        }
    }
}
