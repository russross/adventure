//! Typed logical input reads, retaining the most recently consumed control values.

use super::Adventure;
use crate::input::{Difficulty, Direction};
use adventure_trace::Control;

impl Adventure {
    pub(super) fn read_reset(&mut self) -> bool {
        self.controls.reset = self.input_value(Control::Reset, u8::from(self.controls.reset)) != 0;
        self.controls.reset
    }

    pub(super) fn read_select(&mut self) -> bool {
        self.controls.select =
            self.input_value(Control::Select, u8::from(self.controls.select)) != 0;
        self.controls.select
    }

    // A read can consume a recorded value during playback; subsequent queries see that value.
    pub(super) fn read_color(&mut self) -> bool {
        self.controls.color = self.input_value(Control::Color, u8::from(self.controls.color)) != 0;
        self.controls.color
    }

    pub(super) fn read_fire(&mut self) -> bool {
        self.controls.left.fire =
            self.input_value(Control::Fire, u8::from(self.controls.left.fire)) != 0;
        self.controls.left.fire
    }

    pub(super) fn read_left_direction(&mut self) -> Direction {
        self.controls.left.direction = decode_direction(self.input_value(
            Control::LeftDirection,
            encode_direction(self.controls.left.direction),
        ));
        self.controls.left.direction
    }

    pub(super) fn read_right_direction(&mut self) -> Direction {
        self.controls.right.direction = decode_direction(self.input_value(
            Control::RightDirection,
            encode_direction(self.controls.right.direction),
        ));
        self.controls.right.direction
    }

    // Difficulty is sampled at the actor operation that uses it, rather than once per turn.
    pub(super) fn read_left_difficulty(&mut self) -> Difficulty {
        self.controls.left_difficulty = decode_difficulty(self.input_value(
            Control::LeftDifficulty,
            u8::from(self.controls.left_difficulty == Difficulty::A),
        ));
        self.controls.left_difficulty
    }

    pub(super) fn read_right_difficulty(&mut self) -> Difficulty {
        self.controls.right_difficulty = decode_difficulty(self.input_value(
            Control::RightDifficulty,
            u8::from(self.controls.right_difficulty == Difficulty::A),
        ));
        self.controls.right_difficulty
    }
}

fn encode_direction(value: Direction) -> u8 {
    u8::from(value.right)
        | u8::from(value.left) << 1
        | u8::from(value.up) << 2
        | u8::from(value.down) << 3
}

fn decode_direction(value: u8) -> Direction {
    Direction {
        right: value & 1 != 0,
        left: value & 2 != 0,
        up: value & 4 != 0,
        down: value & 8 != 0,
    }
}

fn decode_difficulty(value: u8) -> Difficulty {
    if value == 0 {
        Difficulty::B
    } else {
        Difficulty::A
    }
}
