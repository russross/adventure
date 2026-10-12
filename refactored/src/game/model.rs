//! World identities, records, and validated game domains.

use super::assets::{EMPTY_IMAGE, WALLS_ENCLOSED_ROOM};
use crate::input::Direction;
use crate::video::{Appearance, ObjectLine, PlayfieldRow, SpriteWidth};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum Room {
    #[default]
    GameSelection,
    WestHall,
    BelowGoldCastle,
    SecretPassage,
    BlueMazeNorth,
    BlueMaze05,
    BlueMazeSouth,
    BlueMazeCenter,
    BlueMazeEntrance,
    CatacombsCenter,
    CatacombsEntrance,
    CatacombsSide,
    SidePassage0c,
    SidePassage0d,
    AboveWhiteCastle,
    WhiteCastle,
    BlackCastle,
    GoldCastle,
    GoldTreasury,
    BlackMaze13,
    BlackMaze14,
    BlackMazeDot,
    BlackMazeEntrance,
    RedMaze17,
    RedMazeNorth,
    RedMazeSouth,
    WhiteCastleInterior,
    BlackCastleInterior,
    PurpleChamber,
    RedHall,
    RobinettCredits,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ObjectId {
    Light,
    GoldGate,
    WhiteGate,
    BlackGate,
    Credits,
    GameNumber,
    Rhindle,
    Yorgle,
    Grundle,
    Sword,
    Bridge,
    GoldKey,
    WhiteKey,
    BlackKey,
    Bat,
    Dot,
    Chalice,
    Magnet,
    Empty,
}

pub(super) const OBJECT_IDS: [ObjectId; 19] = [
    ObjectId::Light,
    ObjectId::GoldGate,
    ObjectId::WhiteGate,
    ObjectId::BlackGate,
    ObjectId::Credits,
    ObjectId::GameNumber,
    ObjectId::Rhindle,
    ObjectId::Yorgle,
    ObjectId::Grundle,
    ObjectId::Sword,
    ObjectId::Bridge,
    ObjectId::GoldKey,
    ObjectId::WhiteKey,
    ObjectId::BlackKey,
    ObjectId::Bat,
    ObjectId::Dot,
    ObjectId::Chalice,
    ObjectId::Magnet,
    ObjectId::Empty,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecordId {
    Player,
    Light,
    Dot,
    Rhindle,
    Yorgle,
    Grundle,
    Magnet,
    Sword,
    Chalice,
    Bridge,
    GoldKey,
    WhiteKey,
    BlackKey,
    Bat,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Position {
    pub room: Room,
    pub x: u8,
    pub y: u8,
}

impl Position {
    pub(super) const fn new(room: Room, x: u8, y: u8) -> Self {
        Self { room, x, y }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Game {
    #[default]
    One,
    Two,
    Three,
}

impl Game {
    pub(super) fn index(self) -> usize {
        match self {
            Self::One => 0,
            Self::Two => 1,
            Self::Three => 2,
        }
    }
    pub(super) fn next(self) -> Self {
        match self {
            Self::One => Self::Two,
            Self::Two => Self::Three,
            Self::Three => Self::One,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DragonState {
    #[default]
    Wandering,
    Dead,
    Swallowed,
    Biting(u8),
}

impl DragonState {
    pub(super) fn value(self) -> u8 {
        match self {
            Self::Wandering => 0,
            Self::Dead => 1,
            Self::Swallowed => 2,
            Self::Biting(value) => value,
        }
    }
    pub(super) fn from_value(value: u8) -> Self {
        match value {
            0 => Self::Wandering,
            1 => Self::Dead,
            2 => Self::Swallowed,
            value => Self::Biting(value),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DragonId {
    Rhindle,
    Yorgle,
    Grundle,
}

impl DragonId {
    pub(super) fn record(self) -> RecordId {
        match self {
            Self::Rhindle => RecordId::Rhindle,
            Self::Yorgle => RecordId::Yorgle,
            Self::Grundle => RecordId::Grundle,
        }
    }
    pub(super) fn object(self) -> ObjectId {
        match self {
            Self::Rhindle => ObjectId::Rhindle,
            Self::Yorgle => ObjectId::Yorgle,
            Self::Grundle => ObjectId::Grundle,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Dragon {
    pub position: Position,
    pub direction: Direction,
    pub state: DragonState,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Bat {
    pub position: Position,
    pub direction: Direction,
    pub animation: u8,
    pub carried: Option<RecordId>,
    pub flight_timer: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GateState {
    Open,
    Closed,
    Moving(u8),
}

impl Default for GateState {
    fn default() -> Self {
        Self::Moving(0)
    }
}

impl GateState {
    pub(super) fn value(self) -> u8 {
        match self {
            Self::Open => 1,
            Self::Closed => 28,
            Self::Moving(value) => value,
        }
    }
    pub(super) fn from_value(value: u8) -> Self {
        match value {
            1 => Self::Open,
            28 => Self::Closed,
            value => Self::Moving(value),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GateId {
    Gold,
    White,
    Black,
}

impl GateId {
    pub(super) fn index(self) -> usize {
        match self {
            Self::Gold => 0,
            Self::White => 1,
            Self::Black => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Gates {
    pub gold: GateState,
    pub white: GateState,
    pub black: GateState,
}

impl Gates {
    pub(super) fn get(self, gate: GateId) -> GateState {
        match gate {
            GateId::Gold => self.gold,
            GateId::White => self.white,
            GateId::Black => self.black,
        }
    }
    pub(super) fn set(&mut self, gate: GateId, value: GateState) {
        match gate {
            GateId::Gold => self.gold = value,
            GateId::White => self.white = value,
            GateId::Black => self.black = value,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum SoundEffect {
    #[default]
    Victory,
    Roar,
    Swallow,
    DragonSlain,
    Drop,
    Pickup,
}

#[derive(Clone, Copy, Debug)]
pub(super) enum Creature {
    Dragon(DragonId),
    Bat,
}

impl Creature {
    pub(super) fn record(self) -> RecordId {
        match self {
            Self::Dragon(dragon) => dragon.record(),
            Self::Bat => RecordId::Bat,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MovementPhase {
    Both,
    Vertical,
    Horizontal,
}

// World records retain positions, actor state, input history, and display observations.
pub struct World {
    pub player: Position,
    pub safe_player: Position,
    pub light: Position,
    pub dot: Position,
    pub rhindle: Dragon,
    pub yorgle: Dragon,
    pub grundle: Dragon,
    pub magnet: Position,
    pub sword: Position,
    pub chalice: Position,
    pub bridge: Position,
    pub gold_key: Position,
    pub white_key: Position,
    pub black_key: Position,
    pub bat: Bat,
    pub gates: Gates,
    pub game: Game,
    pub won: bool,
    pub held: ObjectId,
    pub held_offset_x: u8,
    pub held_offset_y: u8,
    pub first_visible: ObjectId,
    pub second_visible: ObjectId,
    pub last_candidate: ObjectId,
    pub joystick_direction: Direction,
    pub idle_ticks: u8,
    pub idle_periods: u8,
    pub(super) previous_reset: bool,
    pub(super) previous_select: bool,
    pub(super) fire_was_pressed: bool,
    pub(super) sound_remaining: u8,
    pub(super) sound_effect: SoundEffect,
}

impl Default for World {
    fn default() -> Self {
        Self {
            player: Position::default(),
            safe_player: Position::default(),
            light: Position::default(),
            dot: Position::default(),
            rhindle: Dragon::default(),
            yorgle: Dragon::default(),
            grundle: Dragon::default(),
            magnet: Position::default(),
            sword: Position::default(),
            chalice: Position::default(),
            bridge: Position::default(),
            gold_key: Position::default(),
            white_key: Position::default(),
            black_key: Position::default(),
            bat: Bat::default(),
            gates: Gates::default(),
            game: Game::One,
            won: false,
            held: ObjectId::Light,
            held_offset_x: 0,
            held_offset_y: 0,
            first_visible: ObjectId::Light,
            second_visible: ObjectId::Light,
            last_candidate: ObjectId::Light,
            joystick_direction: Direction::default(),
            idle_ticks: 0,
            idle_periods: 0,
            previous_reset: true,
            previous_select: true,
            fire_was_pressed: true,
            sound_remaining: 0,
            sound_effect: SoundEffect::Victory,
        }
    }
}

impl World {
    pub(super) fn position(&self, record: RecordId) -> Position {
        match record {
            RecordId::Player => self.player,
            RecordId::Light => self.light,
            RecordId::Dot => self.dot,
            RecordId::Rhindle => self.rhindle.position,
            RecordId::Yorgle => self.yorgle.position,
            RecordId::Grundle => self.grundle.position,
            RecordId::Magnet => self.magnet,
            RecordId::Sword => self.sword,
            RecordId::Chalice => self.chalice,
            RecordId::Bridge => self.bridge,
            RecordId::GoldKey => self.gold_key,
            RecordId::WhiteKey => self.white_key,
            RecordId::BlackKey => self.black_key,
            RecordId::Bat => self.bat.position,
        }
    }

    pub(super) fn position_mut(&mut self, record: RecordId) -> &mut Position {
        match record {
            RecordId::Player => &mut self.player,
            RecordId::Light => &mut self.light,
            RecordId::Dot => &mut self.dot,
            RecordId::Rhindle => &mut self.rhindle.position,
            RecordId::Yorgle => &mut self.yorgle.position,
            RecordId::Grundle => &mut self.grundle.position,
            RecordId::Magnet => &mut self.magnet,
            RecordId::Sword => &mut self.sword,
            RecordId::Chalice => &mut self.chalice,
            RecordId::Bridge => &mut self.bridge,
            RecordId::GoldKey => &mut self.gold_key,
            RecordId::WhiteKey => &mut self.white_key,
            RecordId::BlackKey => &mut self.black_key,
            RecordId::Bat => &mut self.bat.position,
        }
    }

    pub(super) fn dragon(&self, dragon: DragonId) -> Dragon {
        match dragon {
            DragonId::Rhindle => self.rhindle,
            DragonId::Yorgle => self.yorgle,
            DragonId::Grundle => self.grundle,
        }
    }

    pub(super) fn dragon_mut(&mut self, dragon: DragonId) -> &mut Dragon {
        match dragon {
            DragonId::Rhindle => &mut self.rhindle,
            DragonId::Yorgle => &mut self.yorgle,
            DragonId::Grundle => &mut self.grundle,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Bitmap {
    pub(super) rows: &'static [u8],
    pub(super) start: usize,
}

impl Bitmap {
    pub(super) const fn new(rows: &'static [u8]) -> Self {
        Self { rows, start: 0 }
    }
    pub(super) fn row(self, index: usize) -> u8 {
        self.rows.get(self.start + index).copied().unwrap_or(0)
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ImageChoice {
    pub(super) ceiling: u8,
    pub(super) bitmap: Bitmap,
}

#[derive(Clone, Copy, Debug)]
pub(super) enum ColorChoice {
    Fixed(u8),
    Cycling,
}

#[derive(Clone, Copy, Debug)]
pub(super) enum PositionSource {
    Record(RecordId),
    Fixed(Position),
}

#[derive(Clone, Copy, Debug)]
pub(super) enum StateSource {
    Fixed(u8),
    Dragon(DragonId),
    Gate(GateId),
    GameNumber,
    BatAnimation,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ObjectDescriptor {
    pub(super) position: PositionSource,
    pub(super) state: StateSource,
    pub(super) images: &'static [ImageChoice],
    pub(super) color: ColorChoice,
    pub(super) monochrome: ColorChoice,
    pub(super) style: SpriteWidth,
}

#[derive(Clone, Copy, Debug)]
pub(super) enum Exit {
    Room(Room),
    ByGame(usize),
}

#[derive(Clone, Copy, Debug)]
pub(super) struct RoomDefinition {
    pub(super) walls: &'static [PlayfieldRow; 7],
    pub(super) color: ColorChoice,
    pub(super) monochrome: ColorChoice,
    pub(super) mirrored: bool,
    pub(super) foreground_priority: bool,
    pub(super) player_width: u8,
    pub(super) left_barrier: bool,
    pub(super) right_barrier: bool,
    pub(super) exits: [Exit; 4],
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Reaction {
    pub(super) from: RecordId,
    pub(super) to: RecordId,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ChosenReaction {
    pub(super) reaction: Reaction,
    pub(super) direction: Direction,
}

#[derive(Clone, Copy)]
pub(super) struct StartingWorld {
    pub(super) dot: Position,
    pub(super) rhindle: Dragon,
    pub(super) yorgle: Dragon,
    pub(super) grundle: Dragon,
    pub(super) magnet: Position,
    pub(super) sword: Position,
    pub(super) chalice: Position,
    pub(super) bridge: Position,
    pub(super) gold_key: Position,
    pub(super) white_key: Position,
    pub(super) black_key: Position,
    pub(super) gates: Gates,
    pub(super) bat: Bat,
}

#[derive(Clone, Copy)]
pub(super) struct RoomBounds {
    pub(super) record: RecordId,
    pub(super) first: Room,
    pub(super) last: Room,
}

pub(super) struct PreparedDisplay {
    pub(super) appearance: Appearance,
    pub(super) objects: [ObjectLine; 2],
    pub(super) player_visible: bool,
    pub(super) room_walls: &'static [PlayfieldRow; 7],
    pub(super) first_image: Bitmap,
    pub(super) second_image: Bitmap,
    pub(super) first_position: Position,
    pub(super) second_position: Position,
}

impl Default for PreparedDisplay {
    fn default() -> Self {
        Self {
            appearance: Appearance {
                mirrored: true,
                player_width: 4,
                ..Appearance::default()
            },
            objects: [ObjectLine::default(); 2],
            player_visible: false,
            room_walls: &WALLS_ENCLOSED_ROOM,
            first_image: Bitmap::new(&EMPTY_IMAGE),
            second_image: Bitmap::new(&EMPTY_IMAGE),
            first_position: Position::default(),
            second_position: Position::default(),
        }
    }
}

impl ObjectDescriptor {
    pub(super) fn position_matches(self, record: RecordId) -> bool {
        matches!(self.position, PositionSource::Record(candidate) if candidate == record)
    }
}
