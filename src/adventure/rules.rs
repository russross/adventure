//! Movement, interaction, actor, and effect rules in scheduled operation order.

use super::{
    Adventure, BatCargo, BiteCountdown, Carryable, DragonId, DragonState, Game, GateId, GateState,
    HeldObject, MovementPhase, ObjectId, Offset, Position, RecordId, Room, RoomEdge, Surface,
};
use crate::input::{Difficulty, Direction};
use crate::sound::SoundEffect;
use adventure_trace::Point;
use std::num::NonZeroU8;

// Movement and room boundaries.

impl Adventure {
    // Contacts belong to the last drawn picture, including hidden layers and blank display slots.
    pub(super) fn player_touches_slot(&self, slot: usize) -> bool {
        let touching = self.contacts.player_objects[slot];
        self.trace_player_contact(slot, touching);
        touching
    }

    pub(super) fn player_touches_surface(&self, surface: Surface) -> bool {
        let touching = match surface {
            Surface::Walls => self.contacts.player_walls,
            Surface::LeftBarrier => self.contacts.player_barriers[0],
            Surface::RightBarrier => self.contacts.player_barriers[1],
        };
        self.trace_surface_contact(surface, touching);
        touching
    }

    // Ordered queries preserve both collision policy and the observations made before a rollback.
    pub(super) fn player_is_blocked(&self) -> bool {
        self.player_touches_surface(Surface::Walls)
            || self.player_touches_surface(Surface::LeftBarrier)
            || self.player_touches_surface(Surface::RightBarrier)
                && self.display.visible[1] != Some(ObjectId::Dot)
            || self.player_touches_slot(0) && self.display.visible[0] != Some(ObjectId::Light)
            || self.player_touches_slot(1) && self.display.visible[1] != Some(ObjectId::Light)
    }

    // Roll back blocked movement except within the bridge's traversable corridor.
    pub(super) fn move_player(&mut self, phase: MovementPhase) {
        self.trace_at(match phase {
            MovementPhase::Both => Point::PlayerBoth,
            MovementPhase::Vertical => Point::PlayerVertical,
            MovementPhase::Horizontal => Point::PlayerHorizontal,
        });
        let blocked = self.player_is_blocked();
        let bridge_x = self.world.player.x.wrapping_sub(self.world.bridge.x);
        let bridge_y = self.world.bridge.y.wrapping_sub(self.world.player.y);
        let crossing_bridge = phase == MovementPhase::Horizontal
            && self.world.carrying.item != HeldObject::Item(Carryable::Bridge)
            && self.world.player.room == self.world.bridge.room
            && (10..23).contains(&bridge_x)
            && !(25..252).contains(&bridge_y);
        if !blocked || crossing_bridge {
            self.world.joystick_direction = Direction::default();
            self.world.safe_player = self.world.player;
        }
        if phase == MovementPhase::Both {
            self.world.joystick_direction = self.read_left_direction();
        }
        self.world.player = self.world.safe_player;
        let direction = match phase {
            MovementPhase::Both => self.world.joystick_direction,
            MovementPhase::Vertical => self.world.joystick_direction.vertical(),
            MovementPhase::Horizontal => self.world.joystick_direction.horizontal(),
        };
        self.move_object_through_world(RecordId::Player, direction, 3);
    }

    // Open castle exits precede ordinary edges, and the secret edge precedes the right wrap.
    pub(super) fn move_object_through_world(
        &mut self,
        record: RecordId,
        direction: Direction,
        steps: u8,
    ) {
        self.step_object(record, direction, steps);
        for gate in [GateId::Black, GateId::White, GateId::Gold] {
            if self.world.gates.get(gate) == GateState::Closed {
                continue;
            }
            let position = self.world.position(record);
            let (_, _, inside, outside) = gate.castle();
            if position.room != inside || !position.below_edge(13) {
                continue;
            }
            *self.world.position_mut(record) = Position::new(outside, 80, 44);
            self.world.gates.set(gate, GateState::Open);
            self.trace_transition(record);
            return;
        }
        let mut position = self.world.position(record);
        let exit = if !position.below_edge(106) {
            position.y = 13;
            RoomEdge::North
        } else if position.x < 3 || position.x >= 240 {
            position.x = if record == RecordId::Player { 158 } else { 154 };
            RoomEdge::West
        } else if position.y < 13 {
            position.y = 105;
            RoomEdge::South
        } else if record == RecordId::Player {
            if position.x < 159 {
                return;
            }
            if position.room == Room::SecretPassage && self.world.dot.room != Room::BlackMazeDot {
                position.room = Room::RobinettCredits;
                position.x = 3;
                *self.world.position_mut(record) = position;
                self.trace_transition(record);
                return;
            }
            position.x = 3;
            RoomEdge::East
        } else {
            if position.x < 155 {
                return;
            }
            position.x = 3;
            RoomEdge::East
        };
        position.room = self.room_exit(position.room, exit);
        *self.world.position_mut(record) = position;
        self.trace_transition(record);
    }

    // Each independent direction modifies a byte coordinate with wrapping arithmetic.
    pub(super) fn step_object(&mut self, record: RecordId, direction: Direction, steps: u8) {
        let position = self.world.position_mut(record);
        position.x = position
            .x
            .wrapping_add(if direction.right { steps } else { 0 })
            .wrapping_sub(if direction.left { steps } else { 0 });
        position.y = position
            .y
            .wrapping_add(if direction.up { steps } else { 0 })
            .wrapping_sub(if direction.down { steps } else { 0 });
    }

    // Connections describe the map directly; castle entrances are handled before ordinary edges.
    pub(super) fn room_exit(&self, room: Room, edge: RoomEdge) -> Room {
        room.definition().exit(edge, self.world.game)
    }
}

// Pickup, gates, magnet, and light.

impl Adventure {
    // Fire drops an item; the last rendered contacts determine pickup candidates.
    pub(super) fn handle_pickup_and_drop(&mut self) {
        self.trace_at(Point::Pickup);
        let fire = self.read_fire();
        let newly_pressed = fire && !self.world.fire_was_pressed;
        self.world.fire_was_pressed = fire;
        if newly_pressed && self.world.carrying.item != HeldObject::Empty {
            self.world.carrying.item = HeldObject::Empty;
            self.start_effect(SoundEffect::Drop, 4);
            self.trace_checkpoint();
        }
        let touched = if self.player_touches_slot(0) {
            self.display.visible[0]
        } else if self.player_touches_slot(1) {
            self.display.visible[1]
        } else {
            return;
        };
        let Some(touched) = touched else {
            return;
        };
        let Some(item) = Carryable::from_object(touched) else {
            return;
        };
        if self.find_object_position(touched).room != self.world.player.room {
            return;
        }
        if self.world.carrying.item != HeldObject::Item(item) {
            self.start_effect(SoundEffect::Pickup, 4);
        }
        self.world.carrying.item = HeldObject::Item(item);
        let record = item.record();
        self.step_object(record, self.world.joystick_direction, 6);
        let position = self.world.position(record);
        self.world.carrying.offset = Offset {
            x: position.x.wrapping_sub(self.world.player.x),
            y: position.y.wrapping_sub(self.world.player.y),
        };
    }

    // Keep the carried offset, then resolve boundaries without an extra directional step.
    pub(super) fn move_held_object(&mut self) {
        let Some(record) = self.world.carrying.item.record() else {
            return;
        };
        *self.world.position_mut(record) = Position::new(
            self.world.player.room,
            self.world
                .player
                .x
                .wrapping_add(self.world.carrying.offset.x),
            self.world
                .player
                .y
                .wrapping_add(self.world.carrying.offset.y),
        );
        self.move_object_through_world(record, Direction::default(), 0);
    }

    // Only the displayed slots can report player contact.
    pub(super) fn is_touching_player(&self, object: ObjectId) -> bool {
        if Some(object) == self.display.visible[0] {
            self.player_touches_slot(0)
        } else if Some(object) == self.display.visible[1] {
            self.player_touches_slot(1)
        } else {
            false
        }
    }

    // Return the other member of a displayed object pair that touched.
    pub(super) fn find_touching_object(&self, object: ObjectId) -> Option<ObjectId> {
        self.trace_object_contact();
        if !self.contacts.objects_touch {
            return None;
        }
        if Some(object) == self.display.visible[0] {
            self.display.visible[1]
        } else if Some(object) == self.display.visible[1] {
            self.display.visible[0]
        } else {
            None
        }
    }

    // Gates process black, white, then gold; entry precedes each gate's animation.
    pub(super) fn update_castle_gates(&mut self) {
        for gate in [GateId::Black, GateId::White, GateId::Gold] {
            let (object, key, inside, _) = gate.castle();
            let touched = self.find_touching_object(object);
            if touched == Some(key) {
                self.world
                    .gates
                    .set(gate, self.world.gates.get(gate).on_key_contact());
            }
            if self.world.gates.get(gate) != GateState::Closed {
                let record = if self.is_touching_player(object) {
                    self.world.gates.set(gate, GateState::Open);
                    Some(RecordId::Player)
                } else {
                    touched.and_then(ObjectId::record)
                };
                if let Some(record) = record {
                    self.world.position_mut(record).room = inside;
                    self.world.position_mut(record).y = 16;
                }
            }
            self.world
                .gates
                .set(gate, self.world.gates.get(gate).advance_animation());
        }
    }

    // The attraction point sits eight wrapping coordinate units below the magnet.
    pub(super) fn update_magnet(&mut self) {
        let target = Position {
            y: self.world.magnet.y.wrapping_sub(8),
            ..self.world.magnet
        };
        // The first metal object in the room wins, even if it is already at the target.
        for item in [
            RecordId::GoldKey,
            RecordId::WhiteKey,
            RecordId::BlackKey,
            RecordId::Sword,
            RecordId::Bridge,
            RecordId::Chalice,
        ] {
            if let Some(direction) = direction_toward(self.world.position(item), target) {
                self.move_object_through_world(item, direction, 1);
                break;
            }
        }
    }

    // Dark rooms center the light on the player and clamp its horizontal reach.
    pub(super) fn update_dark_room_light(&mut self) {
        if !self.world.player.room.is_dark() {
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

// Dragon and bat behavior.

impl Adventure {
    // Dragons flee their fears before pursuing the player or their guarded treasures.
    pub(super) fn dragon_direction(&self, dragon: DragonId, difficulty: Difficulty) -> Direction {
        let record = dragon.record();
        if difficulty == Difficulty::A
            && let Some(direction) = direction_toward(self.world.sword, self.world.position(record))
        {
            return direction;
        }
        if dragon == DragonId::Yorgle
            && let Some(direction) =
                direction_toward(self.world.gold_key, self.world.position(record))
        {
            return direction;
        }

        // Pursuit priority varies by dragon. An aligned first target still takes precedence.
        let targets: &[RecordId] = match dragon {
            DragonId::Rhindle => &[RecordId::Player, RecordId::Chalice, RecordId::WhiteKey],
            DragonId::Yorgle => &[RecordId::Player, RecordId::Chalice],
            DragonId::Grundle => &[
                RecordId::Player,
                RecordId::Chalice,
                RecordId::Bridge,
                RecordId::Magnet,
                RecordId::BlackKey,
            ],
        };
        targets
            .iter()
            .find_map(|&target| {
                direction_toward(self.world.position(record), self.world.position(target))
            })
            .unwrap_or(self.world.dragon(dragon).direction)
    }

    // Preserve wandering, sword contact, swallowing, and delayed-bite branch order.
    pub(super) fn update_dragon(&mut self, dragon: DragonId) {
        let object = dragon.object();
        match self.world.dragon(dragon).state {
            DragonState::Wandering => {
                let difficulty = self.read_right_difficulty();
                let speed = match dragon {
                    DragonId::Rhindle => 3,
                    DragonId::Yorgle | DragonId::Grundle => 2,
                };
                let direction = self.dragon_direction(dragon, difficulty);
                self.world.dragon_mut(dragon).direction = direction;
                self.move_object_through_world(dragon.record(), direction, speed);
                if self.is_touching_player(object) {
                    let difficulty = self.read_left_difficulty();
                    let delay = match (self.world.game, difficulty) {
                        (Game::One, Difficulty::B) => 48,
                        (Game::One, Difficulty::A) => 24,
                        (Game::Two | Game::Three, Difficulty::B) => 16,
                        (Game::Two | Game::Three, Difficulty::A) => 10,
                    };
                    self.world.dragon_mut(dragon).state =
                        DragonState::Biting(BiteCountdown::new(delay));
                    self.world.dragon_mut(dragon).position.x = self.world.safe_player.x;
                    self.world.dragon_mut(dragon).position.y = self.world.safe_player.y;
                    self.start_effect(SoundEffect::Roar, 16);
                }
                if self.find_touching_object(object) == Some(ObjectId::Sword) {
                    self.world.dragon_mut(dragon).state = DragonState::Dead;
                    self.start_effect(SoundEffect::DragonSlain, 16);
                }
            }
            DragonState::Dead => {}
            DragonState::Swallowed => {
                let position = self.world.dragon(dragon).position;
                self.world.player = Position::new(
                    position.room,
                    position.x.wrapping_add(3),
                    position.y.wrapping_sub(10),
                );
                self.world.safe_player = self.world.player;
            }
            DragonState::Biting(count) => {
                let next = count.advance();
                self.world.dragon_mut(dragon).state = next;
                if !matches!(next, DragonState::Biting(remaining) if remaining.can_capture())
                    || !self.is_touching_player(object)
                {
                    return;
                }
                self.world.dragon_mut(dragon).state = DragonState::Swallowed;
                self.start_effect(SoundEffect::Swallow, 16);
                let position = &mut self.world.dragon_mut(dragon).position;
                position.x = position.x.min(155);
                position.y = position.y.max(23);
            }
        }
    }

    // The bat reuses its previous heading during its flight commitment.
    pub(super) fn update_bat(&mut self) {
        self.world.bat.animation = self.world.bat.animation.wrapping_add(1);
        if self.world.bat.animation == 8 {
            self.world.bat.animation = 0;
        }
        if let Some(remaining) = self.world.bat.flight_remaining {
            self.world.bat.flight_remaining = NonZeroU8::new(remaining.get() - 1);
            self.move_object_through_world(RecordId::Bat, self.world.bat.direction, 3);
        } else {
            // Cargo is excluded from the bat's ordered targets until its commitment ends.
            let target = [
                BatCargo::Item(Carryable::Chalice),
                BatCargo::Item(Carryable::Sword),
                BatCargo::Item(Carryable::Bridge),
                BatCargo::Item(Carryable::GoldKey),
                BatCargo::Item(Carryable::WhiteKey),
                BatCargo::Item(Carryable::BlackKey),
                BatCargo::Dragon(DragonId::Rhindle),
                BatCargo::Dragon(DragonId::Yorgle),
                BatCargo::Dragon(DragonId::Grundle),
                BatCargo::Item(Carryable::Magnet),
            ]
            .into_iter()
            .find(|&target| {
                self.world.bat.carried.map(BatCargo::record) != Some(RecordId::Bat)
                    && self.world.bat.carried != Some(target)
                    && self.world.position(target.record()).room == self.world.bat.position.room
            });
            if let Some(target) = target {
                self.world.bat.direction = direction_toward(
                    self.world.bat.position,
                    self.world.position(target.record()),
                )
                .expect("the bat target is in its room");
            }
            self.move_object_through_world(RecordId::Bat, self.world.bat.direction, 3);
            if let Some(cargo) = target {
                let target = self.world.position(cargo.record());
                let bat = self.world.bat.position;
                if bat.within_bat_reach(target) {
                    self.world.bat.carried = Some(cargo);
                    self.world.bat.flight_remaining = NonZeroU8::new(240);
                }
            }
        }
        if let Some(cargo) = self.world.bat.carried {
            let cargo = cargo.record();
            let bat = self.world.bat.position;
            *self.world.position_mut(cargo) = Position::new(bat.room, bat.x.wrapping_add(8), bat.y);
            self.trace_cargo_attachment(cargo);
            if self.world.carrying.item.record() == Some(cargo) {
                self.world.carrying.item = HeldObject::Empty;
            }
        }
    }
}

// Game sound effects.

impl Adventure {
    pub(super) fn start_effect(&mut self, effect: SoundEffect, duration: u8) {
        self.world.sound_effect = effect;
        self.world.sound_remaining = duration;
    }

    // Duration advances before voice selection; victory also animates the retained room color.
    pub(super) fn play_game_sound(&mut self) {
        if self.world.sound_remaining == 0 {
            self.sound.silence();
            return;
        }
        self.world.sound_remaining = self.world.sound_remaining.wrapping_sub(1);
        let remaining = self.world.sound_remaining;
        if self.world.sound_effect == SoundEffect::Victory {
            self.display.appearance.foreground = remaining;
        }
        self.sound.effect(self.world.sound_effect, remaining);
    }
}

// An absent target retains actor motion; an aligned target supplies idle motion and wins priority.
fn direction_toward(from: Position, to: Position) -> Option<Direction> {
    (from.room == to.room).then_some(Direction {
        right: to.x > from.x,
        left: to.x < from.x,
        up: to.y > from.y,
        down: to.y < from.y,
    })
}
