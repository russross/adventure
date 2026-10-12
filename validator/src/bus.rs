//! Cartridge/RAM/TIA/RIOT decoding and the ROM-specific semantic observation adapter.

use crate::tia::Tia;
use adventure_trace::{
    ContactTarget, Control, Counter, Game, Mode, Object, Observation, Point, Position,
    PositionAssembly, PositionField, Record, Room, Session, State, Target,
};

const RECORDS: [(Object, usize); 14] = [
    (Object::Player, 0x8a),
    (Object::SafePlayer, 0xe2),
    (Object::Light, 0xd9),
    (Object::Dot, 0xa1),
    (Object::Rhindle, 0xa4),
    (Object::Yorgle, 0xa9),
    (Object::Grundle, 0xae),
    (Object::Magnet, 0xb3),
    (Object::Sword, 0xb6),
    (Object::Chalice, 0xb9),
    (Object::Bridge, 0xbc),
    (Object::GoldKey, 0xbf),
    (Object::WhiteKey, 0xc2),
    (Object::BlackKey, 0xc5),
];

pub struct Bus {
    pub memory: Box<[u8; 65536]>,
    pub session: Session,
    pub pc: u16,
    tia: Tia,
    timer: Option<(u64, u8)>,
    joystick: u8,
    switches: u8,
    fire: u8,
    limit: u64,
    beginning_frame: bool,
    positions: PositionAssembly,
    position_error: Option<String>,
}
impl Bus {
    #[cfg(test)]
    pub fn drawable_position(&self, id: usize) -> i16 {
        self.tia.position(id)
    }
    pub fn diagnostics(&self) -> String {
        format!(
            "player={:?}, {}",
            &self.memory[0x8a..=0x8c],
            self.tia.diagnostics()
        )
    }
    pub fn new(records: Vec<Record>) -> Result<Self, String> {
        let limit = records.last().ok_or("empty trace")?.at.frame;
        let mut memory = Box::new([0; 65536]);
        memory[0xf000..].copy_from_slice(include_bytes!("../../target/validator-rom.bin"));
        Ok(Self {
            memory,
            session: Session::replay(records)?,
            pc: 0,
            tia: Tia::default(),
            timer: None,
            joystick: 255,
            switches: 0x0b,
            fire: 128,
            limit,
            beginning_frame: false,
            positions: PositionAssembly::default(),
            position_error: None,
        })
    }
    pub fn peek(&self, address: u16) -> u8 {
        let a = address & 0x1fff;
        if a & 0x1000 != 0 {
            self.memory[0xf000 + usize::from(a & 0xfff)]
        } else if a & 0x80 != 0 {
            self.memory[usize::from(a & 0x7f) + 0x80]
        } else {
            0
        }
    }
    pub fn advance(&mut self, cycles: u64) {
        self.tia.advance(cycles);
    }
    pub fn write(&mut self, address: u16, value: u8) {
        let a = address & 0x1fff;
        if a & 0x1000 != 0 {
            return;
        }
        if a & 0x80 == 0 {
            self.tia.write(usize::from(a & 0x3f), value);
        } else if a & 0x200 == 0 {
            let address = usize::from(a & 0x7f) + 0x80;
            self.memory[address] = value;
            for (object, base) in RECORDS.into_iter().chain([(Object::Bat, 0xcb)]) {
                let field = match address.checked_sub(base) {
                    Some(0) => ROOMS
                        .get(usize::from(value))
                        .copied()
                        .map(PositionField::Room),
                    Some(1) => Some(PositionField::X(value)),
                    Some(2) => Some(PositionField::Y(value)),
                    _ => continue,
                };
                if let Some(field) = field {
                    self.positions.write(object, field);
                } else {
                    self.position_error = Some(format!("invalid room {value} for {object}"));
                }
                break;
            }
        } else if a & 0x1f == 0x16 {
            self.timer = Some((self.tia.cycles, value));
        }
    }

    // Consume only the control fields used by this instruction; repeated palette reads reuse the sampled color.
    fn input(&mut self, control: Control) {
        let value = self.session.input(control, 0);
        match control {
            Control::LeftDirection | Control::RightDirection => {
                let bits = ((!value & 1) << 3)
                    | ((!value >> 1 & 1) << 2)
                    | (!value >> 2 & 1)
                    | ((!value >> 3 & 1) << 1);
                if control == Control::LeftDirection {
                    self.joystick = self.joystick & 15 | bits << 4;
                } else {
                    self.joystick = self.joystick & 0xf0 | bits;
                }
            }
            Control::Fire => self.fire = if value == 0 { 128 } else { 0 },
            _ => {
                let (mask, enabled) = match control {
                    Control::Reset => (1, value == 0),
                    Control::Select => (2, value == 0),
                    Control::Color => (8, value != 0),
                    Control::LeftDifficulty => (64, value != 0),
                    Control::RightDifficulty => (128, value != 0),
                    _ => unreachable!(),
                };
                self.switches = self.switches & !mask | if enabled { mask } else { 0 };
            }
        }
    }
    pub fn read(&mut self, address: u16) -> u8 {
        let a = address & 0x1fff;
        if a & 0x1000 != 0 || a & 0x280 == 0x80 {
            return self.peek(address);
        }
        match a {
            0x280 => {
                self.input(Control::LeftDirection);
                if self.pc == 0xf2be {
                    self.input(Control::RightDirection);
                }
                self.joystick
            }
            0x282 => {
                match self.pc {
                    0xf384 => self.input(Control::Reset),
                    0xf3b5 => self.input(Control::Select),
                    0xf40c => {
                        self.session.at(Point::History);
                        self.input(Control::Reset);
                        self.input(Control::Select);
                    }
                    0xf2c5 => {
                        self.input(Control::Reset);
                        self.input(Control::Select);
                    }
                    0xf120 => self.input(Control::Color),
                    0xf7f6 => self.input(Control::RightDifficulty),
                    0xf812 => self.input(Control::LeftDifficulty),
                    _ => {}
                }
                self.switches
            }
            0x284 => match self.timer {
                None => 0,
                Some((start, value)) => {
                    let elapsed = self.tia.cycles - start;
                    let underflow = (u64::from(value) + 1) * 64;
                    if elapsed < underflow {
                        value - (elapsed / 64) as u8
                    } else {
                        255u8.wrapping_sub((elapsed - underflow) as u8)
                    }
                }
            },
            _ if a & 0x80 == 0 => {
                let register = usize::from(a & 15);
                if register == 12 {
                    self.input(Control::Fire);
                    return self.fire;
                }
                if register > 7 {
                    return 0;
                }
                let value = self.tia.contact(register);
                let target = match register {
                    2 => catalog(self.memory[0x95]).map(ContactTarget::Object),
                    3 => catalog(self.memory[0x96]).map(ContactTarget::Object),
                    4 => Some(ContactTarget::Surface(Target::LeftBarrier)),
                    5 => Some(ContactTarget::Surface(Target::RightBarrier)),
                    6 => Some(ContactTarget::Surface(Target::Walls)),
                    _ => None,
                };
                if value & if register == 6 { 128 } else { 64 } != 0
                    && let Some(target) = target
                {
                    self.session
                        .observe(Observation::Contact(Object::Player, target));
                }
                if register == 7
                    && value & 128 != 0
                    && let (Some(first), Some(second)) =
                        (catalog(self.memory[0x95]), catalog(self.memory[0x96]))
                {
                    self.session
                        .observe(Observation::Contact(first, ContactTarget::Object(second)));
                }
                value
            }
            _ => 0,
        }
    }

    // Commit at game operation boundaries, after all component stores have completed.
    pub fn before(&mut self, pc: u16, _x: u8, y: u8) -> Result<bool, String> {
        if pc == 0xf000 && self.session.stopped_at_boot() {
            self.session.finish()?;
            return Ok(false);
        }
        match pc {
            0xf0ec if !self.beginning_frame => {
                if self.session.frame == self.limit {
                    self.session.finish()?;
                    return Ok(false);
                }
                self.session.next_frame();
                self.beginning_frame = true;
            }
            0xf3d3 => self.session.initialize(self.game()?),
            0xf384 => self.session.at(Point::Console),
            0xf2b2 => self.session.at(Point::Inactivity),
            0xf4c2 => self.session.at(match y {
                0 => Point::PlayerBoth,
                1 => Point::PlayerVertical,
                _ => Point::PlayerHorizontal,
            }),
            0xf10f => self.session.at(Point::Display),
            0xf556 => self.session.at(Point::Pickup),
            0xf3f5 | 0xf3fe | 0xf411 | 0xf30f | 0xf323 | 0xf572 | 0xf337 => self.checkpoint()?,
            0xf328 => {
                self.checkpoint()?;
                self.session.at(Point::Held);
            }
            0xf32b | 0xf33f | 0xf351 | 0xf35f => self.checkpoint()?,
            0xf33c => {
                self.checkpoint()?;
                self.session.at(Point::Light);
            }
            0xf342 => self.session.at(Point::Bat),
            0xf345 => {
                if self.session.crossed(Object::Bat)
                    && let Some(cargo) = record(self.memory[0xd0])
                {
                    self.session.observe(Observation::Transition(
                        cargo,
                        self.position(usize::from(self.memory[0xd0]))?,
                    ));
                }
                self.checkpoint()?;
                self.session.at(Point::Gates);
            }
            0xf348 => self.checkpoint()?,
            0xf34b => self.session.at(Point::Grundle),
            0xf34e => {
                self.checkpoint()?;
                self.session.at(Point::Yorgle);
            }
            0xf359 => {
                self.checkpoint()?;
                self.session.at(Point::Rhindle);
            }
            0xf35c => {
                self.checkpoint()?;
                self.session.at(Point::Magnet);
            }
            _ => {}
        }
        Ok(true)
    }
    pub fn after(&mut self, pc: u16, x: u8) -> Result<(), String> {
        if pc == 0xf10e {
            self.beginning_frame = false;
        }
        if matches!(pc, 0xf6a9 | 0xf68d | 0xf62b) {
            let object = record(x).ok_or("transition for unknown object record")?;
            self.session.observe(Observation::Transition(
                object,
                self.position(usize::from(x))?,
            ));
        }
        Ok(())
    }
    fn position(&self, address: usize) -> Result<Position, String> {
        Ok(Position {
            room: *ROOMS
                .get(usize::from(self.memory[address]))
                .ok_or("invalid room in ROM state")?,
            x: self.memory[address + 1],
            y: self.memory[address + 2],
        })
    }
    fn state(&mut self, object: Object, state: State) {
        self.session.observe(Observation::State(object, state));
    }
    fn game(&self) -> Result<Game, String> {
        match self.memory[0xdd] {
            0 => Ok(Game::One),
            2 => Ok(Game::Two),
            4 => Ok(Game::Three),
            value => Err(format!("invalid game selection {value}")),
        }
    }
    fn checkpoint(&mut self) -> Result<(), String> {
        if let Some(error) = &self.position_error {
            return Err(error.clone());
        }
        let order: Vec<_> = RECORDS
            .iter()
            .map(|(object, _)| *object)
            .chain([Object::Bat])
            .collect();
        for observation in self.positions.commit(&order)? {
            self.session.observe(observation);
        }
        for (object, address) in [
            (Object::Rhindle, 0xa4),
            (Object::Yorgle, 0xa9),
            (Object::Grundle, 0xae),
        ] {
            let raw = self.memory[address + 4];
            let mode = match raw {
                0 => Mode::Wandering,
                1 => Mode::Dead,
                2 => Mode::Swallowed,
                _ => Mode::Biting,
            };
            self.state(object, State::Mode(mode));
            self.state(
                object,
                State::Direction(direction(self.memory[address + 3])),
            );
            self.state(
                object,
                State::Counter(Counter::Bite, if raw > 2 { raw } else { 0 }),
            );
        }
        for (object, address) in [
            (Object::GoldGate, 0xc8),
            (Object::WhiteGate, 0xc9),
            (Object::BlackGate, 0xca),
        ] {
            let raw = self.memory[address];
            let mode = match raw {
                1 => Mode::Open,
                28 => Mode::Closed,
                0..28 => Mode::Closing,
                _ => Mode::Opening,
            };
            self.state(object, State::Mode(mode));
            self.state(
                object,
                State::Counter(Counter::Gate, if raw == 1 || raw == 28 { 0 } else { raw }),
            );
        }
        self.state(
            Object::Player,
            State::Carry {
                item: catalog(self.memory[0x9d]),
                x: self.memory[0x9e],
                y: self.memory[0x9f],
            },
        );
        self.state(
            Object::Bat,
            State::Carry {
                item: record(self.memory[0xd0]),
                x: 8,
                y: 0,
            },
        );
        self.state(Object::Bat, State::Direction(direction(self.memory[0xce])));
        self.state(
            Object::Bat,
            State::Counter(Counter::Animation, self.memory[0xcf]),
        );
        self.state(
            Object::Bat,
            State::Counter(Counter::Flight, self.memory[0xd1]),
        );
        self.state(Object::World, State::Game(self.game()?));
        self.state(Object::World, State::Won(self.memory[0xde] != 0));
        self.state(
            Object::World,
            State::Counter(Counter::IdleTicks, self.memory[0xe5]),
        );
        self.state(
            Object::World,
            State::Counter(Counter::IdlePeriods, self.memory[0xe6]),
        );
        Ok(())
    }
}
fn direction(value: u8) -> u8 {
    let v = !value;
    (v >> 7 & 1) | (v >> 6 & 1) << 1 | (v >> 4 & 1) << 2 | (v >> 5 & 1) << 3
}
fn record(address: u8) -> Option<Object> {
    if address == 0xcb {
        return Some(Object::Bat);
    }
    RECORDS
        .iter()
        .find(|(_, a)| *a == usize::from(address))
        .map(|(o, _)| *o)
}
fn catalog(offset: u8) -> Option<Object> {
    if !offset.is_multiple_of(9) {
        return None;
    }
    OBJECTS.get(usize::from(offset / 9)).copied()
}

const ROOMS: [Room; 31] = [
    Room::GameSelection,
    Room::WestHall,
    Room::BelowGoldCastle,
    Room::SecretPassage,
    Room::BlueMazeNorth,
    Room::BlueMaze05,
    Room::BlueMazeSouth,
    Room::BlueMazeCenter,
    Room::BlueMazeEntrance,
    Room::CatacombsCenter,
    Room::CatacombsEntrance,
    Room::CatacombsSide,
    Room::SidePassage0c,
    Room::SidePassage0d,
    Room::AboveWhiteCastle,
    Room::WhiteCastle,
    Room::BlackCastle,
    Room::GoldCastle,
    Room::GoldTreasury,
    Room::BlackMaze13,
    Room::BlackMaze14,
    Room::BlackMazeDot,
    Room::BlackMazeEntrance,
    Room::RedMaze17,
    Room::RedMazeNorth,
    Room::RedMazeSouth,
    Room::WhiteCastleInterior,
    Room::BlackCastleInterior,
    Room::PurpleChamber,
    Room::RedHall,
    Room::RobinettCredits,
];

const OBJECTS: [Object; 18] = [
    Object::Light,
    Object::GoldGate,
    Object::WhiteGate,
    Object::BlackGate,
    Object::Credits,
    Object::GameNumber,
    Object::Rhindle,
    Object::Yorgle,
    Object::Grundle,
    Object::Sword,
    Object::Bridge,
    Object::GoldKey,
    Object::WhiteKey,
    Object::BlackKey,
    Object::Bat,
    Object::Dot,
    Object::Chalice,
    Object::Magnet,
];

#[cfg(test)]
mod tests {
    use super::Bus;
    use adventure_trace::Session;

    #[test]
    fn riot_countdown_underflow_and_ram_stack_mirroring() {
        let mut recording = Session::record();
        recording.finish().unwrap();
        let mut bus = Bus::new(recording.records().to_vec()).unwrap();
        bus.write(0x1ff, 42);
        assert_eq!(bus.peek(0xff), 42);
        bus.write(0x296, 2);
        assert_eq!(bus.read(0x284), 2);
        bus.advance(63);
        assert_eq!(bus.read(0x284), 2);
        bus.advance(1);
        assert_eq!(bus.read(0x284), 1);
        bus.advance(64);
        assert_eq!(bus.read(0x284), 0);
        bus.advance(64);
        assert_eq!(bus.read(0x284), 255);
        bus.advance(1);
        assert_eq!(bus.read(0x284), 254);
    }
}
