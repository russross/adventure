use super::Adventure;
use super::assets::{CASTLE_INTERIORS, CASTLE_OUTSIDES, GAME_DEPENDENT_EXITS};
use super::model::{Exit, GateId, GateState, MovementPhase, ObjectId, Position, RecordId, Room};
use crate::input::Direction;

impl Adventure {
    // Roll back blocked movement except within the bridge's traversable corridor.
    pub(super) fn move_player(&mut self, phase: MovementPhase) {
        let contacts = self.video.contacts;
        let blocked = contacts.player_walls
            || contacts.player_barriers[0]
            || contacts.player_barriers[1] && self.world.second_visible != ObjectId::Dot
            || contacts.player_objects[0] && self.world.first_visible != ObjectId::Light
            || contacts.player_objects[1] && self.world.second_visible != ObjectId::Light;
        let bridge_x = self.world.player.x.wrapping_sub(self.world.bridge.x);
        let bridge_y = self.world.bridge.y.wrapping_sub(self.world.player.y);
        let crossing_bridge = phase == MovementPhase::Horizontal
            && self.world.held != ObjectId::Bridge
            && self.world.player.room == self.world.bridge.room
            && (10..23).contains(&bridge_x)
            && !(25..252).contains(&bridge_y);
        if !blocked || crossing_bridge {
            self.world.joystick_direction = Direction::default();
            self.world.safe_player = self.world.player;
        }
        if phase == MovementPhase::Both {
            self.world.joystick_direction = self.controls.left.direction;
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
            if position.room != CASTLE_INTERIORS[gate.index()]
                || (position.y.wrapping_sub(13) as i8) >= 0
            {
                continue;
            }
            *self.world.position_mut(record) = Position::new(CASTLE_OUTSIDES[gate.index()], 80, 44);
            self.world.gates.set(gate, GateState::Open);
            return;
        }
        let mut position = self.world.position(record);
        let exit = if (position.y.wrapping_sub(106) as i8) >= 0 {
            position.y = 13;
            0
        } else if position.x < 3 || position.x >= 240 {
            position.x = if record == RecordId::Player { 158 } else { 154 };
            3
        } else if position.y < 13 {
            position.y = 105;
            2
        } else if record == RecordId::Player {
            if position.x < 159 {
                return;
            }
            if position.room == Room::SecretPassage && self.world.dot.room != Room::BlackMazeDot {
                position.room = Room::RobinettCredits;
                position.x = 3;
                *self.world.position_mut(record) = position;
                return;
            }
            position.x = 3;
            1
        } else {
            if position.x < 155 {
                return;
            }
            position.x = 3;
            1
        };
        let destination = self.find_room_record(position.room).exits[exit];
        position.room = self.resolve_game_exit(destination);
        *self.world.position_mut(record) = position;
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

    // Tagged exits select the current game's member of a three-room group.
    pub(super) fn resolve_game_exit(&self, exit: Exit) -> Room {
        match exit {
            Exit::Room(room) => room,
            Exit::ByGame(group) => GAME_DEPENDENT_EXITS[group][self.world.game.index()],
        }
    }
}
