//! Console switches and joystick directions, independent of SDL event delivery.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Direction {
    pub right: bool,
    pub left: bool,
    pub up: bool,
    pub down: bool,
}

impl Direction {
    pub fn is_idle(self) -> bool {
        !self.right && !self.left && !self.up && !self.down
    }

    // The three movement passes first allow both axes, then retry one axis at a time.
    pub fn vertical(self) -> Self {
        Self {
            right: false,
            left: false,
            ..self
        }
    }

    pub fn horizontal(self) -> Self {
        Self {
            up: false,
            down: false,
            ..self
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Joystick {
    pub direction: Direction,
    pub fire: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Difficulty {
    A,
    #[default]
    B,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Controls {
    pub left: Joystick,
    pub right: Joystick,
    pub select: bool,
    pub reset: bool,
    pub color: bool,
    pub left_difficulty: Difficulty,
    pub right_difficulty: Difficulty,
}

impl Default for Controls {
    fn default() -> Self {
        Self {
            left: Joystick::default(),
            right: Joystick::default(),
            select: false,
            reset: false,
            color: true,
            left_difficulty: Difficulty::B,
            right_difficulty: Difficulty::B,
        }
    }
}
