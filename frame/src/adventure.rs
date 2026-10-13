//! World simulation, picture scheduling, rendering, and immutable game data.

use crate::hardware::FrameHost;
use crate::input::{Controls, Difficulty, Direction};
use crate::sound::Sound;
use crate::video::{
    Appearance, HEIGHT, Object, ObjectLine, Picture, PlayfieldRow, SpriteWidth, Video, WIDTH,
};
use Exit::{ByGame, Fixed};
use adventure_trace::{Point, Session};
use std::cell::RefCell;

#[path = "controls.rs"]
mod controls;
#[path = "observation.rs"]
mod observation;

// World identities, records, and display state.

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

impl Room {
    fn is_dark(self) -> bool {
        self.definition().features.dark
    }
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
}

const OBJECT_IDS: [ObjectId; 18] = [
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
    const fn new(room: Room, x: u8, y: u8) -> Self {
        Self { room, x, y }
    }
}

impl ObjectId {
    // Scenery has no movable record; every other identity names a live world position.
    fn record(self) -> Option<RecordId> {
        match self {
            Self::Light => Some(RecordId::Light),
            Self::Rhindle => Some(RecordId::Rhindle),
            Self::Yorgle => Some(RecordId::Yorgle),
            Self::Grundle => Some(RecordId::Grundle),
            Self::Sword => Some(RecordId::Sword),
            Self::Bridge => Some(RecordId::Bridge),
            Self::GoldKey => Some(RecordId::GoldKey),
            Self::WhiteKey => Some(RecordId::WhiteKey),
            Self::BlackKey => Some(RecordId::BlackKey),
            Self::Bat => Some(RecordId::Bat),
            Self::Dot => Some(RecordId::Dot),
            Self::Chalice => Some(RecordId::Chalice),
            Self::Magnet => Some(RecordId::Magnet),
            Self::GoldGate
            | Self::WhiteGate
            | Self::BlackGate
            | Self::Credits
            | Self::GameNumber => None,
        }
    }
}

// Player cargo excludes scenery and dragons. The cleared startup identity is modeled separately.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Carryable {
    Sword,
    Bridge,
    GoldKey,
    WhiteKey,
    BlackKey,
    Bat,
    Dot,
    Chalice,
    Magnet,
}

impl Carryable {
    fn from_object(object: ObjectId) -> Option<Self> {
        match object {
            ObjectId::Sword => Some(Self::Sword),
            ObjectId::Bridge => Some(Self::Bridge),
            ObjectId::GoldKey => Some(Self::GoldKey),
            ObjectId::WhiteKey => Some(Self::WhiteKey),
            ObjectId::BlackKey => Some(Self::BlackKey),
            ObjectId::Bat => Some(Self::Bat),
            ObjectId::Dot => Some(Self::Dot),
            ObjectId::Chalice => Some(Self::Chalice),
            ObjectId::Magnet => Some(Self::Magnet),
            _ => None,
        }
    }

    pub fn object(self) -> ObjectId {
        match self {
            Self::Sword => ObjectId::Sword,
            Self::Bridge => ObjectId::Bridge,
            Self::GoldKey => ObjectId::GoldKey,
            Self::WhiteKey => ObjectId::WhiteKey,
            Self::BlackKey => ObjectId::BlackKey,
            Self::Bat => ObjectId::Bat,
            Self::Dot => ObjectId::Dot,
            Self::Chalice => ObjectId::Chalice,
            Self::Magnet => ObjectId::Magnet,
        }
    }

    fn record(self) -> RecordId {
        self.object().record().expect("cargo has a movable record")
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HeldObject {
    #[default]
    StartupLight,
    Empty,
    Item(Carryable),
}

impl HeldObject {
    pub fn object(self) -> Option<ObjectId> {
        match self {
            Self::StartupLight => Some(ObjectId::Light),
            Self::Empty => None,
            Self::Item(item) => Some(item.object()),
        }
    }

    fn record(self) -> Option<RecordId> {
        self.object().and_then(ObjectId::record)
    }
}

// Offsets survive dropping and initialization even when the player has no cargo.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Carrying {
    pub item: HeldObject,
    pub offset: Offset,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Offset {
    pub x: u8,
    pub y: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BatCargo {
    Item(Carryable),
    Dragon(DragonId),
}

impl BatCargo {
    fn record(self) -> RecordId {
        match self {
            Self::Item(item) => item.record(),
            Self::Dragon(dragon) => dragon.record(),
        }
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
    fn next(self) -> Self {
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
    fn from_value(value: u8) -> Self {
        match value {
            0 => Self::Wandering,
            1 => Self::Dead,
            2 => Self::Swallowed,
            value => Self::Biting(value),
        }
    }

    // A missed bite wraps back to wandering; intermediate values keep the biting image.
    fn advance_bite(count: u8) -> Self {
        Self::from_value(count.wrapping_add(1))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DragonId {
    Rhindle,
    Yorgle,
    Grundle,
}

impl DragonId {
    fn record(self) -> RecordId {
        match self {
            Self::Rhindle => RecordId::Rhindle,
            Self::Yorgle => RecordId::Yorgle,
            Self::Grundle => RecordId::Grundle,
        }
    }
    fn object(self) -> ObjectId {
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
    pub carried: Option<BatCargo>,
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
    fn value(self) -> u8 {
        match self {
            Self::Open => 1,
            Self::Closed => 28,
            Self::Moving(value) => value,
        }
    }
    fn from_value(value: u8) -> Self {
        match value {
            1 => Self::Open,
            28 => Self::Closed,
            value => Self::Moving(value),
        }
    }

    fn on_key_contact(self) -> Self {
        Self::from_value(self.value().wrapping_add(1))
    }

    // Entry can stop the animation before its next tick. The endpoints retain their encoded values.
    fn advance_animation(self) -> Self {
        match self {
            Self::Moving(value) => match value.wrapping_add(1) {
                56 => Self::Open,
                next => Self::from_value(next),
            },
            _ => self,
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
    // Each castle binds its gate and key to the rooms on either side of the entrance.
    fn castle(self) -> (ObjectId, ObjectId, Room, Room) {
        match self {
            Self::Gold => (
                ObjectId::GoldGate,
                ObjectId::GoldKey,
                Room::GoldTreasury,
                Room::GoldCastle,
            ),
            Self::White => (
                ObjectId::WhiteGate,
                ObjectId::WhiteKey,
                Room::WhiteCastleInterior,
                Room::WhiteCastle,
            ),
            Self::Black => (
                ObjectId::BlackGate,
                ObjectId::BlackKey,
                Room::BlackCastleInterior,
                Room::BlackCastle,
            ),
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
    fn get(self, gate: GateId) -> GateState {
        match gate {
            GateId::Gold => self.gold,
            GateId::White => self.white,
            GateId::Black => self.black,
        }
    }
    fn set(&mut self, gate: GateId, value: GateState) {
        match gate {
            GateId::Gold => self.gold = value,
            GateId::White => self.white = value,
            GateId::Black => self.black = value,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum SoundEffect {
    #[default]
    Victory,
    Roar,
    Swallow,
    DragonSlain,
    Drop,
    Pickup,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MovementPhase {
    Both,
    Vertical,
    Horizontal,
}

#[derive(Clone, Copy)]
enum Surface {
    Walls,
    LeftBarrier,
    RightBarrier,
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
    pub carrying: Carrying,
    pub visible: [Option<ObjectId>; 2],
    pub last_candidate: ObjectId,
    pub joystick_direction: Direction,
    pub idle_ticks: u8,
    pub idle_periods: u8,
    previous_reset: bool,
    previous_select: bool,
    fire_was_pressed: bool,
    sound_remaining: u8,
    sound_effect: SoundEffect,
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
            carrying: Carrying::default(),
            visible: [Some(ObjectId::Light); 2],
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
    fn position(&self, record: RecordId) -> Position {
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

    fn position_mut(&mut self, record: RecordId) -> &mut Position {
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

    fn dragon(&self, dragon: DragonId) -> Dragon {
        match dragon {
            DragonId::Rhindle => self.rhindle,
            DragonId::Yorgle => self.yorgle,
            DragonId::Grundle => self.grundle,
        }
    }

    fn dragon_mut(&mut self, dragon: DragonId) -> &mut Dragon {
        match dragon {
            DragonId::Rhindle => &mut self.rhindle,
            DragonId::Yorgle => &mut self.yorgle,
            DragonId::Grundle => &mut self.grundle,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Bitmap {
    rows: &'static [u8],
}

impl Bitmap {
    const fn new(rows: &'static [u8]) -> Self {
        Self { rows }
    }
    fn row(self, index: usize) -> u8 {
        self.rows.get(index).copied().unwrap_or(0)
    }
}

#[derive(Clone, Copy, Debug)]
struct ObjectDisplay {
    position: Position,
    bitmap: Bitmap,
    color: u8,
    width: SpriteWidth,
}

#[derive(Clone, Copy, Debug)]
enum ColorChoice {
    Fixed(u8),
    Cycling,
}

#[derive(Clone, Copy)]
enum RoomEdge {
    North,
    East,
    South,
    West,
}

impl ObjectDisplay {
    fn prepare(self, slot: usize) -> Object {
        Object {
            line: ObjectLine {
                x: object_screen_x(self.position.x, self.width),
                color: self.color,
                width: self.width,
                bits: 0,
            },
            rows: sprite_rows(self.bitmap, self.position.y, slot),
        }
    }
}

struct PreparedDisplay {
    appearance: Appearance,
    objects: [Object; 2],
    walls: [PlayfieldRow; HEIGHT],
}

impl Default for PreparedDisplay {
    fn default() -> Self {
        Self {
            appearance: Appearance {
                mirrored: true,
                player_width: 4,
                ..Appearance::default()
            },
            objects: [Object {
                line: ObjectLine::default(),
                rows: [0; HEIGHT],
            }; 2],
            walls: wall_rows(&WALLS_ENCLOSED_ROOM),
        }
    }
}

// Startup and picture scheduling.

#[derive(Default)]
pub struct Adventure {
    pub world: World,
    pub video: Video,
    pub sound: Sound,
    pub controls: Controls,
    pub trace: Option<RefCell<Session>>,
    display: PreparedDisplay,
    player_visible: bool,
    phase: PicturePhase,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum PicturePhase {
    #[default]
    Player,
    BatAndGates,
    RemainingActors,
}

impl Adventure {
    // The entry keeps startup and the repeated game-turn sequence separate.
    pub fn run(&mut self, host: &mut impl FrameHost) -> Result<(), String> {
        if !self.power_on(host)? {
            if let Some(trace) = &self.trace {
                trace.borrow_mut().finish()?;
            }
            return Ok(());
        }
        while self.advance_picture(host)? {
            if let Some(trace) = &self.trace {
                trace.borrow().check()?;
            }
        }
        if let Some(trace) = &self.trace {
            trace.borrow_mut().finish()?;
        }
        Ok(())
    }

    // Clear state once, place the fixed barriers, and initialize the selected world.
    fn power_on(&mut self, host: &mut impl FrameHost) -> Result<bool, String> {
        self.world = World::default();
        self.phase = PicturePhase::Player;
        self.video = Video::default();
        self.display = PreparedDisplay::default();
        self.player_visible = false;
        self.sound = Sound::default();
        if self
            .trace
            .as_ref()
            .is_some_and(|trace| trace.borrow().stopped_at_boot())
        {
            return Ok(false);
        }
        let Some(controls) = host.sample_controls()? else {
            return Ok(false);
        };
        self.controls = controls;
        self.initialize_world(host)
    }

    // A turn ends after three active pictures or one victory picture.
    pub fn advance_turn(&mut self, host: &mut impl FrameHost) -> Result<bool, String> {
        loop {
            if !self.advance_picture(host)? {
                return Ok(false);
            }
            if self.phase == PicturePhase::Player {
                return Ok(true);
            }
        }
    }

    // Each phase makes its update boundary explicit and presents exactly once.
    pub fn advance_picture(&mut self, host: &mut impl FrameHost) -> Result<bool, String> {
        match self.phase {
            PicturePhase::Player => {
                if !self.poll_console_switches(host)? {
                    return Ok(false);
                }
                self.play_game_sound();
                self.track_inactivity();
                self.trace_checkpoint();
                if self.world.won {
                    if !self.present_picture(host)? {
                        return Ok(false);
                    }
                    self.draw_room();
                    self.prepare_room_display();
                    return Ok(true);
                }
                if self.world.chalice.room == Room::GoldTreasury {
                    self.world.won = true;
                    self.start_effect(SoundEffect::Victory, 255);
                }
                self.trace_checkpoint();

                // Player movement precedes presentation; pickup uses the newly drawn contacts.
                self.move_player(MovementPhase::Both);
                self.trace_checkpoint();
                self.trace_at(Point::Held);
                self.move_held_object();
                self.trace_checkpoint();
                if !self.present_picture(host)? {
                    return Ok(false);
                }
                self.prepare_room_display();
                self.draw_room();
                self.handle_pickup_and_drop();
                self.trace_checkpoint();
                self.phase = PicturePhase::BatAndGates;
            }
            PicturePhase::BatAndGates => {
                self.move_player(MovementPhase::Vertical);
                self.trace_checkpoint();
                self.trace_at(Point::Light);
                self.update_dark_room_light();
                self.trace_checkpoint();
                if !self.present_picture(host)? {
                    return Ok(false);
                }
                self.trace_at(Point::Bat);
                self.update_bat();
                self.trace_checkpoint();
                self.trace_at(Point::Gates);
                self.update_castle_gates();
                self.trace_checkpoint();
                self.draw_room();
                self.trace_at(Point::Grundle);
                self.update_dragon(DragonId::Grundle);
                self.trace_checkpoint();
                self.trace_at(Point::Yorgle);
                self.update_dragon(DragonId::Yorgle);
                self.trace_checkpoint();
                self.phase = PicturePhase::RemainingActors;
            }
            PicturePhase::RemainingActors => {
                if !self.present_picture(host)? {
                    return Ok(false);
                }
                self.move_player(MovementPhase::Horizontal);
                self.trace_checkpoint();
                self.trace_at(Point::Rhindle);
                self.update_dragon(DragonId::Rhindle);
                self.trace_checkpoint();
                self.trace_at(Point::Magnet);
                self.update_magnet();
                self.trace_checkpoint();
                self.draw_room();
                self.phase = PicturePhase::Player;
            }
        }
        Ok(true)
    }

    // Present the accumulated frame and sample controls at the vertical boundary.
    fn present_picture(&mut self, host: &mut impl FrameHost) -> Result<bool, String> {
        match host.present(&self.video.frame, &self.sound)? {
            Some(controls) => {
                self.controls = controls;
                if let Some(trace) = &self.trace {
                    trace.borrow_mut().next_frame();
                }
                Ok(true)
            }
            None => Ok(false),
        }
    }

    // Button edges preserve RESET's partial revival and SELECT's world reset.
    fn poll_console_switches(&mut self, host: &mut impl FrameHost) -> Result<bool, String> {
        self.trace_at(Point::Console);
        if self.read_reset() && !self.world.previous_reset {
            if self.world.won {
                return self.initialize_world(host);
            }
            self.world.player = Position::new(Room::GoldCastle, 80, 32);
            self.world.safe_player = self.world.player;
            self.world.rhindle.state = DragonState::Wandering;
            self.world.yorgle.state = DragonState::Wandering;
            self.world.grundle.state = DragonState::Wandering;
            self.world.sound_remaining = 0;
            self.world.carrying.item = HeldObject::Empty;
        }
        if self.read_select() && !self.world.previous_select {
            if self.world.player.room == Room::GameSelection {
                self.world.game = self.world.game.next();
            }
            return self.initialize_world(host);
        }
        self.remember_console_switches();
        self.trace_checkpoint();
        Ok(true)
    }

    // Place the selected game's actors and items; display caches and counters remain live.
    pub fn initialize_world(&mut self, host: &mut impl FrameHost) -> Result<bool, String> {
        self.trace_initialize();
        self.world.player.room = Room::GameSelection;
        self.world.safe_player.room = Room::GameSelection;
        self.world.player.y = 0;
        self.world.safe_player.y = 0;
        let world = &mut self.world;
        world.dot = Position::new(Room::BlackMazeDot, 81, 18);
        world.gates = Gates {
            gold: GateState::Closed,
            white: GateState::Closed,
            black: GateState::Closed,
        };
        world.bat = Bat::default();

        // Games 2 and 3 share the same starting layout before Game 3 relocates its objects.
        match world.game {
            Game::One => {
                world.rhindle.position = Position::new(Room::AboveWhiteCastle, 80, 32);
                world.yorgle.position = Position::new(Room::WestHall, 80, 32);
                world.grundle.position = Position::new(Room::RedHall, 80, 32);
                world.magnet = Position::new(Room::BlackCastleInterior, 128, 32);
                world.sword = Position::new(Room::GoldTreasury, 32, 32);
                world.chalice = Position::new(Room::PurpleChamber, 48, 32);
                world.bridge = Position::new(Room::BlueMazeNorth, 41, 55);
                world.gold_key = Position::new(Room::GoldCastle, 32, 64);
                world.white_key = Position::new(Room::AboveWhiteCastle, 32, 64);
                world.black_key = Position::new(Room::RedHall, 32, 64);
                world.bat.position = Position::new(Room::WhiteCastleInterior, 32, 32);
            }
            Game::Two | Game::Three => {
                world.rhindle.position = Position::new(Room::BlackMaze14, 80, 32);
                world.yorgle.position = Position::new(Room::RedMazeSouth, 80, 32);
                world.grundle.position = Position::new(Room::BlueMazeNorth, 80, 32);
                world.magnet = Position::new(Room::AboveWhiteCastle, 128, 32);
                world.sword = Position::new(Room::GoldCastle, 32, 32);
                world.chalice = Position::new(Room::BlackMaze14, 48, 32);
                world.bridge = Position::new(Room::CatacombsSide, 64, 64);
                world.gold_key = Position::new(Room::CatacombsCenter, 32, 64);
                world.white_key = Position::new(Room::BlueMazeSouth, 32, 64);
                world.black_key = Position::new(Room::RedMazeSouth, 32, 64);
                world.bat.position = Position::new(Room::BelowGoldCastle, 32, 32);
            }
        }

        // Opposing directions cancel in Game 1; later games begin with committed headings.
        let first_game = world.game == Game::One;
        for dragon in [&mut world.rhindle, &mut world.yorgle, &mut world.grundle] {
            dragon.state = DragonState::Wandering;
            dragon.direction = Direction {
                right: first_game,
                left: true,
                up: true,
                down: first_game,
            };
        }
        world.bat.direction = Direction {
            right: first_game,
            left: true,
            up: first_game,
            down: true,
        };

        self.trace_checkpoint();
        if self.world.game == Game::Three {
            self.randomize_game_three();
            self.trace_checkpoint();
            if !self.present_picture(host)? {
                return Ok(false);
            }
            self.draw_room();
        }
        self.world.won = false;
        self.world.carrying.item = HeldObject::Empty;
        self.remember_console_switches();
        self.trace_checkpoint();
        Ok(true)
    }

    // Initialization and console polling both resample switch history at their completion boundary.
    fn remember_console_switches(&mut self) {
        self.trace_at(Point::History);
        let reset = self.read_reset();
        let select = self.read_select();
        self.world.previous_reset = reset;
        self.world.previous_select = select;
    }

    // Placement order and rejection sampling retain the counter's seeded sequence.
    fn randomize_game_three(&mut self) {
        for record in [
            RecordId::Magnet,
            RecordId::Bat,
            RecordId::BlackKey,
            RecordId::WhiteKey,
            RecordId::GoldKey,
            RecordId::Bridge,
            RecordId::Sword,
            RecordId::Grundle,
            RecordId::Yorgle,
            RecordId::Rhindle,
            RecordId::Chalice,
        ] {
            let (first, last) = match record {
                RecordId::Chalice => (Room::BlackMaze13, Room::WhiteCastleInterior),
                RecordId::BlackKey => (Room::WestHall, Room::GoldTreasury),
                RecordId::WhiteKey => (Room::WestHall, Room::BlackMazeEntrance),
                _ => (Room::WestHall, Room::RedHall),
            };
            self.world.position_mut(record).room = self.random_room(first, last);
        }
    }

    fn random_room(&mut self, first: Room, last: Room) -> Room {
        loop {
            let old = self.world.idle_ticks;
            self.world.idle_ticks = (old >> 5).wrapping_add(old).wrapping_add(1);
            let candidate = self.world.idle_ticks & 31;
            if candidate >= first as u8 && candidate <= last as u8 {
                return ROOM_IDS[usize::from(candidate)];
            }
        }
    }

    // The counter also supplies dynamic colors and Game 3's placement seed.
    fn track_inactivity(&mut self) {
        self.trace_at(Point::Inactivity);
        let left = self.read_left_direction();
        let right = self.read_right_direction();
        if left.is_idle() && right.is_idle() {
            self.read_reset();
            self.read_select();
        }
        self.world.idle_ticks = self.world.idle_ticks.wrapping_add(1);
        if self.world.idle_ticks == 0 {
            self.world.idle_periods = self.world.idle_periods.wrapping_add(1);
            if self.world.idle_periods == 0 {
                self.world.idle_periods = 0x80;
            }
        }
        if !self.controls.left.direction.is_idle()
            || !self.controls.right.direction.is_idle()
            || self.controls.reset
            || self.controls.select
        {
            self.world.idle_periods = 0;
        }
    }
}

// Movement and room boundaries.

impl Adventure {
    // Contacts belong to the last drawn picture, including hidden layers and blank display slots.
    fn player_touches_slot(&self, slot: usize) -> bool {
        let touching = self.video.contacts.player_objects[slot];
        self.trace_player_contact(slot, touching);
        touching
    }

    fn player_touches_surface(&self, surface: Surface) -> bool {
        let touching = match surface {
            Surface::Walls => self.video.contacts.player_walls,
            Surface::LeftBarrier => self.video.contacts.player_barriers[0],
            Surface::RightBarrier => self.video.contacts.player_barriers[1],
        };
        self.trace_surface_contact(surface, touching);
        touching
    }

    // Ordered queries preserve both collision policy and the observations made before a rollback.
    fn player_is_blocked(&self) -> bool {
        self.player_touches_surface(Surface::Walls)
            || self.player_touches_surface(Surface::LeftBarrier)
            || self.player_touches_surface(Surface::RightBarrier)
                && self.world.visible[1] != Some(ObjectId::Dot)
            || self.player_touches_slot(0) && self.world.visible[0] != Some(ObjectId::Light)
            || self.player_touches_slot(1) && self.world.visible[1] != Some(ObjectId::Light)
    }

    // Roll back blocked movement except within the bridge's traversable corridor.
    fn move_player(&mut self, phase: MovementPhase) {
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
    fn move_object_through_world(&mut self, record: RecordId, direction: Direction, steps: u8) {
        self.step_object(record, direction, steps);
        for gate in [GateId::Black, GateId::White, GateId::Gold] {
            if self.world.gates.get(gate) == GateState::Closed {
                continue;
            }
            let position = self.world.position(record);
            let (_, _, inside, outside) = gate.castle();
            if position.room != inside || (position.y.wrapping_sub(13) as i8) >= 0 {
                continue;
            }
            *self.world.position_mut(record) = Position::new(outside, 80, 44);
            self.world.gates.set(gate, GateState::Open);
            self.trace_transition(record);
            return;
        }
        let mut position = self.world.position(record);
        let exit = if (position.y.wrapping_sub(106) as i8) >= 0 {
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
    fn step_object(&mut self, record: RecordId, direction: Direction, steps: u8) {
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
    fn room_exit(&self, room: Room, edge: RoomEdge) -> Room {
        room.definition().exit(edge, self.world.game)
    }
}

// Pickup, gates, magnet, and light.

impl Adventure {
    // Fire drops an item; the last rendered contacts determine pickup candidates.
    fn handle_pickup_and_drop(&mut self) {
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
            self.world.visible[0]
        } else if self.player_touches_slot(1) {
            self.world.visible[1]
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
    fn move_held_object(&mut self) {
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
    fn is_touching_player(&self, object: ObjectId) -> bool {
        if Some(object) == self.world.visible[0] {
            self.player_touches_slot(0)
        } else if Some(object) == self.world.visible[1] {
            self.player_touches_slot(1)
        } else {
            false
        }
    }

    // Return the other member of a displayed object pair that touched.
    fn find_touching_object(&self, object: ObjectId) -> Option<ObjectId> {
        self.trace_object_contact();
        if !self.video.contacts.objects_touch {
            return None;
        }
        if Some(object) == self.world.visible[0] {
            self.world.visible[1]
        } else if Some(object) == self.world.visible[1] {
            self.world.visible[0]
        } else {
            None
        }
    }

    // Gates process black, white, then gold; entry precedes each gate's animation.
    fn update_castle_gates(&mut self) {
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
    fn update_magnet(&mut self) {
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
    fn update_dark_room_light(&mut self) {
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
    fn dragon_direction(&self, dragon: DragonId, difficulty: Difficulty) -> Direction {
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
    fn update_dragon(&mut self, dragon: DragonId) {
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
                        (Game::One, Difficulty::B) => 0xd0,
                        (Game::One, Difficulty::A) => 0xe8,
                        (Game::Two | Game::Three, Difficulty::B) => 0xf0,
                        (Game::Two | Game::Three, Difficulty::A) => 0xf6,
                    };
                    self.world.dragon_mut(dragon).state = DragonState::Biting(delay);
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
                self.world.dragon_mut(dragon).state = DragonState::advance_bite(count);
                let count = count.wrapping_add(1);
                if count < 252 || !self.is_touching_player(object) {
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
    fn update_bat(&mut self) {
        self.world.bat.animation = self.world.bat.animation.wrapping_add(1);
        if self.world.bat.animation == 8 {
            self.world.bat.animation = 0;
        }
        if self.world.bat.flight_timer != 0 {
            self.world.bat.flight_timer = self.world.bat.flight_timer.wrapping_add(1);
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
                if target.room == bat.room
                    && target.x.wrapping_sub(bat.x).wrapping_add(4) & 0xf8 == 0
                    && target.y.wrapping_sub(bat.y).wrapping_add(4) & 0xf8 == 0
                {
                    self.world.bat.carried = Some(cargo);
                    self.world.bat.flight_timer = 16;
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
    fn start_effect(&mut self, effect: SoundEffect, duration: u8) {
        self.world.sound_effect = effect;
        self.world.sound_remaining = duration;
    }

    // Duration advances before voice selection; victory also animates the retained room color.
    fn play_game_sound(&mut self) {
        if self.world.sound_remaining == 0 {
            self.sound.silence();
            return;
        }
        self.world.sound_remaining = self.world.sound_remaining.wrapping_sub(1);
        let remaining = self.world.sound_remaining;
        let (waveform, frequency, volume) = match self.world.sound_effect {
            SoundEffect::Victory => {
                self.display.appearance.foreground = remaining;
                (remaining, remaining >> 3, remaining >> 1)
            }
            SoundEffect::Roar => (
                if remaining & 1 != 0 { 3 } else { 8 },
                (remaining >> 2).wrapping_add(28),
                remaining,
            ),
            SoundEffect::Swallow => (6, remaining ^ 15, (remaining >> 1).wrapping_add(8)),
            SoundEffect::DragonSlain => (4, remaining ^ 31, remaining),
            SoundEffect::Drop => (6, remaining ^ 3, 5),
            SoundEffect::Pickup => (6, remaining, 5),
        };
        self.sound.voice(waveform, frequency, volume);
    }
}

// Display preparation and complete-picture construction.

impl Adventure {
    // Room layers remain frozen between display refreshes; the player is sampled for every picture.
    fn draw_room(&mut self) {
        let player_rows = player_rows(self.world.player.y, self.player_visible);
        let picture = Picture {
            appearance: self.display.appearance,
            objects: self.display.objects,
            walls: self.display.walls,
            player_x: screen_x(positioned_object_x(self.world.player.x) - 1),
            player_visible: player_rows,
        };
        self.player_visible = player_rows[HEIGHT - 1];
        self.video.paint(&picture);
    }

    // Appearance and images are a snapshot, reused by the following two pictures.
    fn prepare_room_display(&mut self) {
        self.trace_at(Point::Display);
        let color = self.read_color();
        let (walls, appearance) = self.room_appearance(self.world.player.room, color);
        self.display.walls = wall_rows(walls);
        self.display.appearance = appearance;
        self.select_visible_objects();
        if self.world.visible[0] == Some(ObjectId::Light)
            || self.world.visible[0] == Some(ObjectId::Bridge)
                && self.world.visible[1] != Some(ObjectId::Light)
        {
            self.world.visible.swap(0, 1);
        }

        // Resolve the live object's appearance once; following pictures reuse this snapshot.
        for (slot, object) in self.world.visible.into_iter().enumerate() {
            self.display.objects[slot] = match object {
                Some(object) => self.object_display(object).prepare(slot),
                // A blank slot retains the bridge position and black color but cannot make contacts.
                None => ObjectDisplay {
                    position: self.world.bridge,
                    bitmap: Bitmap::new(&EMPTY_IMAGE),
                    color: self.resolve_display_color(ColorChoice::Fixed(0)),
                    width: SpriteWidth::Normal,
                }
                .prepare(slot),
            };
        }
    }

    // Rotate over the real inhabitants, retaining at most two in this room.
    fn select_visible_objects(&mut self) {
        self.world.visible = [None; 2];
        let mut candidate = self.world.last_candidate;
        loop {
            candidate = OBJECT_IDS[(candidate as usize + 1) % OBJECT_IDS.len()];
            if self.find_object_position(candidate).room == self.world.player.room {
                if self.world.visible[0].is_none() {
                    self.world.visible[0] = Some(candidate);
                } else {
                    self.world.visible[1] = Some(candidate);
                    break;
                }
            }
            if candidate == self.world.last_candidate {
                break;
            }
        }
        self.world.last_candidate = candidate;
    }

    // Wall artwork varies by room; darkness controls both the light and foreground priority.
    fn room_appearance(&self, room: Room, color: bool) -> (&'static [PlayfieldRow; 7], Appearance) {
        let definition = room.definition();
        let foreground = if color {
            definition.color
        } else {
            definition.monochrome
        };
        (
            definition.walls,
            Appearance {
                foreground: self.resolve_display_color(ColorChoice::Fixed(foreground)),
                background: self.resolve_display_color(ColorChoice::Fixed(8)),
                mirrored: definition.features.mirrored,
                walls_in_front: definition.features.dark,
                player_width: 4,
                barriers: definition.features.barriers,
            },
        )
    }

    // Resolve literal or counter-driven colors, then apply the idle transform.
    fn resolve_display_color(&self, color: ColorChoice) -> u8 {
        let mut value = match color {
            ColorChoice::Fixed(value) => value >> 1,
            ColorChoice::Cycling => self.world.idle_ticks,
        };
        if self.world.idle_periods & 0x80 != 0 {
            value = (value ^ self.world.idle_periods) & 0xfb;
        }
        value.wrapping_shl(1)
    }

    // Live objects use their world positions; scenery stays in its room.
    fn find_object_position(&self, object: ObjectId) -> Position {
        if let Some(record) = object.record() {
            return self.world.position(record);
        }
        match object {
            ObjectId::GoldGate => Position::new(Room::GoldCastle, 77, 49),
            ObjectId::WhiteGate => Position::new(Room::WhiteCastle, 77, 49),
            ObjectId::BlackGate => Position::new(Room::BlackCastle, 77, 49),
            ObjectId::Credits => Position::new(Room::RobinettCredits, 80, 105),
            ObjectId::GameNumber => Position::new(Room::GameSelection, 80, 64),
            _ => unreachable!("live objects have records"),
        }
    }

    // Artwork follows named live states; static objects need no state lookup.
    fn object_display(&self, object: ObjectId) -> ObjectDisplay {
        let rows: &'static [u8] = match object {
            ObjectId::Light => &LIGHT_IMAGE,
            ObjectId::GoldGate => self.gate_image(GateId::Gold),
            ObjectId::WhiteGate => self.gate_image(GateId::White),
            ObjectId::BlackGate => self.gate_image(GateId::Black),
            ObjectId::Credits => &ROBINETT_CREDITS_IMAGE,
            ObjectId::GameNumber => match self.world.game {
                Game::One => &GAME_ONE_IMAGE,
                Game::Two => &GAME_TWO_IMAGE,
                Game::Three => &GAME_THREE_IMAGE,
            },
            ObjectId::Rhindle => dragon_image(self.world.rhindle.state),
            ObjectId::Yorgle => dragon_image(self.world.yorgle.state),
            ObjectId::Grundle => dragon_image(self.world.grundle.state),
            ObjectId::Sword => &SWORD_IMAGE,
            ObjectId::Bridge => &BRIDGE_IMAGE,
            ObjectId::GoldKey | ObjectId::WhiteKey | ObjectId::BlackKey => &KEY_IMAGE,
            ObjectId::Bat if self.world.bat.animation < 4 => &BAT_FRAME_ONE,
            ObjectId::Bat => &BAT_FRAME_TWO,
            ObjectId::Dot => &DOT_IMAGE,
            ObjectId::Chalice => &CHALICE_IMAGE,
            ObjectId::Magnet => &MAGNET_IMAGE,
        };

        // Only the light and bridge are enlarged. Cycling objects share the idle palette rule.
        let width = if matches!(object, ObjectId::Light | ObjectId::Bridge) {
            SpriteWidth::Wide
        } else {
            SpriteWidth::Normal
        };
        let color = if self.controls.color {
            match object {
                ObjectId::Light => ColorChoice::Fixed(0x28),
                ObjectId::Credits | ObjectId::Chalice => ColorChoice::Cycling,
                ObjectId::GameNumber | ObjectId::Grundle => ColorChoice::Fixed(0xc8),
                ObjectId::Rhindle => ColorChoice::Fixed(0x36),
                ObjectId::Yorgle | ObjectId::Sword | ObjectId::GoldKey => ColorChoice::Fixed(0x1a),
                ObjectId::Bridge => ColorChoice::Fixed(0x66),
                ObjectId::WhiteKey => ColorChoice::Fixed(0x0e),
                ObjectId::Dot => ColorChoice::Fixed(0x08),
                _ => ColorChoice::Fixed(0),
            }
        } else {
            ColorChoice::Fixed(match object {
                ObjectId::Light => 0x0c,
                ObjectId::Rhindle | ObjectId::WhiteKey => 0x0e,
                ObjectId::Yorgle
                | ObjectId::Sword
                | ObjectId::GoldKey
                | ObjectId::Chalice
                | ObjectId::Magnet => 0x06,
                ObjectId::Bridge => 0x02,
                ObjectId::Dot => 0x08,
                _ => 0,
            })
        };
        ObjectDisplay {
            position: self.find_object_position(object),
            bitmap: Bitmap::new(rows),
            color: self.resolve_display_color(color),
            width,
        }
    }

    // Gates grow to full height, then retract, changing two bitmap rows every four ticks.
    fn gate_image(&self, gate: GateId) -> &'static [u8] {
        let value = self.world.gates.get(gate).value();
        let stage = usize::from(value.saturating_sub(1) / 4).min(12);
        let start = stage.abs_diff(6) * 2;
        &GATE_IMAGE[start..]
    }
}

fn dragon_image(state: DragonState) -> &'static [u8] {
    match state {
        DragonState::Wandering | DragonState::Swallowed => &DRAGON_WALKING_IMAGE,
        DragonState::Dead => &DRAGON_DEAD_IMAGE,
        DragonState::Biting(_) => &DRAGON_BITING_IMAGE,
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

// Wall changes begin on the second row of a logical band boundary.
fn wall_rows(bands: &[PlayfieldRow; 7]) -> [PlayfieldRow; HEIGHT] {
    let mut rows = [bands[0]; HEIGHT];
    let mut band = 0;
    for (index, pair) in rows.chunks_exact_mut(2).enumerate() {
        pair[0] = bands[band];
        if (104 - index) % 16 == 0 {
            band += 1;
        }
        pair[1] = bands[band];
    }
    rows
}

// Each artwork row occupies the second physical row and the following first row.
fn sprite_rows(bitmap: Bitmap, y: u8, slot: usize) -> [u8; HEIGHT] {
    let mut rows = [0; HEIGHT];
    let mut cursor = 0;
    let mut bits = 0;
    for (index, pair) in rows.chunks_exact_mut(2).enumerate() {
        pair[0] = bits;
        let row = 104 - index as u8;
        if below(row, y) && (slot == 0 || index != 0) {
            bits = bitmap.row(cursor);
            cursor += usize::from(bits != 0);
        } else if slot == 0 {
            bits = 0;
        }
        // The second slot retains its last fetched row across a wrapping eligibility boundary.
        pair[1] = bits;
    }
    rows
}

// The player's first physical row comes from the preceding picture rather than its current position.
fn player_rows(y: u8, retained: bool) -> [bool; HEIGHT] {
    let mut rows = [false; HEIGHT];
    let mut visible = retained;
    let bottom = y.wrapping_sub(4);
    for (index, pair) in rows.chunks_exact_mut(2).enumerate() {
        pair[0] = visible;
        visible = (104 - index as u8).wrapping_sub(bottom) & 0xfc == 0;
        pair[1] = visible;
    }
    rows
}

// Coordinates retain byte wrapping before their signed vertical comparison.
fn below(row: u8, y: u8) -> bool {
    (row.wrapping_sub(y) as i8) < 0
}

// Game coordinates encode the cartridge's calibrated placement, including line crossings.
fn positioned_object_x(x: u8) -> i16 {
    match x {
        0..=164 => i16::from(x) - 1,
        165..=239 => i16::from(x % 15) - 4,
        240..=255 => i16::from(x) - 229,
    }
}

fn object_screen_x(x: u8, width: SpriteWidth) -> u8 {
    let expansion_shift = i16::from(matches!(width, SpriteWidth::Wide));
    screen_x(positioned_object_x(x) + expansion_shift)
}

// Normalize after the signed correction so the left edge wraps through pixel 159.
fn screen_x(x: i16) -> u8 {
    x.rem_euclid(WIDTH as i16) as u8
}

const ROOM_IDS: [Room; 31] = [
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

// Room definitions.

#[derive(Clone, Copy)]
enum Exit {
    Fixed(Room),
    ByGame { first: Room, later: Room },
}

impl Exit {
    fn destination(self, game: Game) -> Room {
        match self {
            Self::Fixed(room) => room,
            Self::ByGame { first, later } => match game {
                Game::One => first,
                Game::Two | Game::Three => later,
            },
        }
    }
}

// Named fields keep the map readable without depending on directional indices.
struct Exits {
    north: Exit,
    east: Exit,
    south: Exit,
    west: Exit,
}

impl Exits {
    fn new(north: Exit, east: Exit, south: Exit, west: Exit) -> Self {
        Self {
            north,
            east,
            south,
            west,
        }
    }
}

#[derive(Clone, Copy)]
struct Features {
    dark: bool,
    mirrored: bool,
    barriers: [bool; 2],
}

// Dark rooms use foreground priority as well as the player's light.
const ORDINARY: Features = Features {
    dark: false,
    mirrored: true,
    barriers: [false; 2],
};
const DARK: Features = Features {
    dark: true,
    ..ORDINARY
};
const DARK_REPEATED: Features = Features {
    mirrored: false,
    ..DARK
};
const LEFT_BARRIER: Features = Features {
    barriers: [true, false],
    ..ORDINARY
};
const RIGHT_BARRIER: Features = Features {
    barriers: [false, true],
    ..ORDINARY
};

struct RoomDefinition {
    exits: Exits,
    walls: &'static [PlayfieldRow; 7],
    color: u8,
    monochrome: u8,
    features: Features,
}

impl RoomDefinition {
    fn exit(&self, edge: RoomEdge, game: Game) -> Room {
        match edge {
            RoomEdge::North => self.exits.north,
            RoomEdge::East => self.exits.east,
            RoomEdge::South => self.exits.south,
            RoomEdge::West => self.exits.west,
        }
        .destination(game)
    }
}

impl Room {
    // Each definition describes ordinary exits. Castle openings and secret entry are movement rules.
    #[rustfmt::skip]
    fn definition(self) -> RoomDefinition {
        let (exits, walls, color, monochrome, features) = match self {
            Self::GameSelection | Self::GoldTreasury => (
                Exits::new(Fixed(self), Fixed(self), Fixed(self), Fixed(self)),
                &WALLS_ENCLOSED_ROOM, if self == Self::GoldTreasury { 0x1a } else { 0x66 }, 0x0a, ORDINARY,
            ),
            Self::WestHall => (
                Exits::new(Fixed(Self::BlueMazeEntrance), Fixed(Self::BelowGoldCastle), ByGame { first: Self::BlackCastle, later: Self::WhiteCastle }, Fixed(Self::SecretPassage)),
                &WALLS_OPEN_HALL, 0xd8, 0x0a, LEFT_BARRIER,
            ),
            Self::BelowGoldCastle => (
                Exits::new(Fixed(Self::GoldCastle), Fixed(Self::SecretPassage), ByGame { first: Self::BlueMaze05, later: Self::GoldCastle }, Fixed(Self::WestHall)),
                &WALLS_OPEN_HALL, 0xc8, 0x0a, ORDINARY,
            ),
            Self::SecretPassage => (
                Exits::new(Fixed(Self::BlueMazeSouth), Fixed(Self::WestHall), ByGame { first: Self::RedHall, later: Self::CatacombsEntrance }, Fixed(Self::BelowGoldCastle)),
                &WALLS_SECRET_PASSAGE, 0xe8, 0x0a, RIGHT_BARRIER,
            ),

            // The blue maze joins the outdoor halls and the Black Castle facade.
            Self::BlueMazeNorth => (
                Exits::new(Fixed(Self::BlackCastle), Fixed(Self::BlueMaze05), Fixed(Self::BlueMazeCenter), Fixed(Self::BlueMazeSouth)),
                &WALLS_BLUE_MAZE_NORTH, 0x86, 0x0a, ORDINARY,
            ),
            Self::BlueMaze05 => (
                Exits::new(Fixed(Self::RedHall), Fixed(Self::BlueMazeSouth), Fixed(Self::BlueMazeEntrance), Fixed(Self::BlueMazeNorth)),
                &WALLS_BLUE_MAZE_05, 0x86, 0x0a, ORDINARY,
            ),
            Self::BlueMazeSouth => (
                Exits::new(Fixed(Self::BlueMazeCenter), Fixed(Self::BlueMazeNorth), Fixed(Self::SecretPassage), Fixed(Self::BlueMaze05)),
                &WALLS_BLUE_MAZE_SOUTH, 0x86, 0x0a, ORDINARY,
            ),
            Self::BlueMazeCenter => (
                Exits::new(Fixed(Self::BlueMazeNorth), Fixed(Self::BlueMazeEntrance), Fixed(Self::BlueMazeSouth), Fixed(Self::BlueMazeEntrance)),
                &WALLS_BLUE_MAZE_CENTER, 0x86, 0x0a, ORDINARY,
            ),
            Self::BlueMazeEntrance => (
                Exits::new(Fixed(Self::BlueMaze05), Fixed(Self::BlueMazeCenter), Fixed(Self::WestHall), Fixed(Self::BlueMazeCenter)),
                &WALLS_BLUE_MAZE_ENTRANCE, 0x86, 0x0a, ORDINARY,
            ),

            // The catacombs share a dark palette and open into the two side passages.
            Self::CatacombsCenter => (
                Exits::new(Fixed(Self::CatacombsEntrance), Fixed(Self::CatacombsEntrance), Fixed(Self::CatacombsSide), Fixed(Self::CatacombsEntrance)),
                &WALLS_CATACOMBS_CENTER, 0x08, 0x08, DARK,
            ),
            Self::CatacombsEntrance => (
                Exits::new(Fixed(Self::SecretPassage), Fixed(Self::CatacombsCenter), Fixed(Self::CatacombsCenter), Fixed(Self::CatacombsCenter)),
                &WALLS_CATACOMBS_ENTRANCE, 0x08, 0x08, DARK,
            ),
            Self::CatacombsSide => (
                Exits::new(Fixed(Self::CatacombsCenter), Fixed(Self::SidePassage0c), Fixed(Self::PurpleChamber), Fixed(Self::SidePassage0d)),
                &WALLS_CATACOMBS_SIDE, 0x08, 0x08, DARK,
            ),
            Self::SidePassage0c => (
                Exits::new(Fixed(Self::PurpleChamber), Fixed(Self::SidePassage0d), Fixed(Self::RedHall), Fixed(Self::CatacombsSide)),
                &WALLS_SIDE_PASSAGE, 0x98, 0x0a, RIGHT_BARRIER,
            ),
            Self::SidePassage0d => (
                Exits::new(Fixed(Self::WhiteCastle), Fixed(Self::CatacombsSide), Fixed(Self::AboveWhiteCastle), Fixed(Self::SidePassage0c)),
                &WALLS_SIDE_PASSAGE, 0xb8, 0x0a, LEFT_BARRIER,
            ),

            // Facade edges remain ordinary map connections outside the gate openings.
            Self::AboveWhiteCastle => (
                Exits::new(Fixed(Self::SidePassage0d), Fixed(Self::BlackCastle), Fixed(Self::WhiteCastle), Fixed(Self::BlackCastle)),
                &WALLS_TOP_OPENING, 0xa8, 0x0a, ORDINARY,
            ),
            Self::WhiteCastle => (
                Exits::new(Fixed(Self::AboveWhiteCastle), Fixed(Self::WhiteCastle), Fixed(Self::SidePassage0d), Fixed(Self::WhiteCastle)),
                &WALLS_CASTLE, 0x0c, 0x0c, ORDINARY,
            ),
            Self::BlackCastle => (
                Exits::new(Fixed(Self::WestHall), Fixed(Self::PurpleChamber), Fixed(Self::BlueMazeNorth), Fixed(Self::PurpleChamber)),
                &WALLS_CASTLE, 0x00, 0x02, ORDINARY,
            ),
            Self::GoldCastle => (
                Exits::new(Fixed(Self::BlueMazeSouth), Fixed(Self::SecretPassage), Fixed(Self::BelowGoldCastle), Fixed(Self::WestHall)),
                &WALLS_CASTLE, 0x1a, 0x0a, ORDINARY,
            ),

            // Two dark-maze rooms repeat their wall artwork instead of mirroring it.
            Self::BlackMaze13 => (
                Exits::new(Fixed(Self::BlackMazeDot), Fixed(Self::BlackMaze14), Fixed(Self::BlackMazeDot), Fixed(Self::BlackMazeEntrance)),
                &WALLS_BLACK_MAZE_13, 0x08, 0x08, DARK,
            ),
            Self::BlackMaze14 => (
                Exits::new(Fixed(Self::BlackMazeEntrance), Fixed(Self::BlackMazeDot), Fixed(Self::BlackMazeEntrance), Fixed(Self::BlackMaze13)),
                &WALLS_BLACK_MAZE_14, 0x08, 0x08, DARK_REPEATED,
            ),
            Self::BlackMazeDot => (
                Exits::new(Fixed(Self::BlackMaze13), Fixed(Self::BlackMazeEntrance), Fixed(Self::BlackMaze13), Fixed(Self::BlackMaze14)),
                &WALLS_BLACK_MAZE_DOT, 0x08, 0x08, DARK_REPEATED,
            ),
            Self::BlackMazeEntrance => (
                Exits::new(Fixed(Self::BlackMaze14), Fixed(Self::BlackMaze13), Fixed(Self::BlackCastleInterior), Fixed(Self::BlackMazeDot)),
                &WALLS_BLACK_MAZE_ENTRANCE, 0x08, 0x08, DARK,
            ),

            // White Castle contains the red maze; Black Castle's ordinary exits depend on the game.
            Self::RedMaze17 => (
                Exits::new(Fixed(Self::RedMazeSouth), Fixed(Self::RedMazeNorth), Fixed(Self::RedMazeSouth), Fixed(Self::RedMazeNorth)),
                &WALLS_RED_MAZE_17, 0x36, 0x0a, ORDINARY,
            ),
            Self::RedMazeNorth => (
                Exits::new(Fixed(Self::WhiteCastleInterior), Fixed(Self::RedMaze17), Fixed(Self::WhiteCastleInterior), Fixed(Self::RedMaze17)),
                &WALLS_RED_MAZE_NORTH, 0x36, 0x0a, ORDINARY,
            ),
            Self::RedMazeSouth => (
                Exits::new(Fixed(Self::RedMaze17), Fixed(Self::WhiteCastleInterior), Fixed(Self::RedMaze17), Fixed(Self::WhiteCastleInterior)),
                &WALLS_RED_MAZE_SOUTH, 0x36, 0x0a, ORDINARY,
            ),
            Self::WhiteCastleInterior => (
                Exits::new(Fixed(Self::RedMazeNorth), Fixed(Self::RedMazeSouth), Fixed(Self::RedMazeNorth), Fixed(Self::RedMazeSouth)),
                &WALLS_WHITE_CASTLE_INTERIOR, 0x36, 0x0a, ORDINARY,
            ),
            Self::BlackCastleInterior => (
                Exits::new(ByGame { first: Self::PurpleChamber, later: Self::BlackMazeEntrance }, ByGame { first: Self::PurpleChamber, later: Self::BlackMazeEntrance }, ByGame { first: Self::PurpleChamber, later: Self::BlackMazeEntrance }, ByGame { first: Self::PurpleChamber, later: Self::BlackMazeEntrance }),
                &WALLS_BLACK_CASTLE_INTERIOR, 0x36, 0x0a, ORDINARY,
            ),

            // The remaining chambers connect the two castle mazes to the outdoor map.
            Self::PurpleChamber => (
                Exits::new(Fixed(Self::RedHall), Fixed(Self::BlueMazeCenter), ByGame { first: Self::BlackCastleInterior, later: Self::SidePassage0c }, Fixed(Self::BlueMazeEntrance)),
                &WALLS_ENCLOSED_ROOM, 0x66, 0x0a, ORDINARY,
            ),
            Self::RedHall => (
                Exits::new(ByGame { first: Self::SecretPassage, later: Self::SidePassage0c }, Fixed(Self::WestHall), Fixed(Self::BlackCastle), Fixed(Self::SecretPassage)),
                &WALLS_TOP_OPENING, 0x36, 0x0a, ORDINARY,
            ),
            Self::RobinettCredits => (
                Exits::new(Fixed(Self::BlueMazeSouth), Fixed(Self::WestHall), Fixed(Self::BlueMazeSouth), Fixed(Self::SecretPassage)),
                &WALLS_OPEN_HALL, 0x66, 0x0a, ORDINARY,
            ),
        };
        RoomDefinition {
            exits,
            walls,
            color,
            monochrome,
            features,
        }
    }
}

// Wall artwork.

static WALLS_SECRET_PASSAGE: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0xff, 0xff),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0xf0, 0xff, 0x0f),
];

static WALLS_OPEN_HALL: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0xff, 0x0f),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0xf0, 0xff, 0xff),
];

static WALLS_SIDE_PASSAGE: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0xff, 0x0f),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0xf0, 0xff, 0x0f),
];

static WALLS_ENCLOSED_ROOM: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0xff, 0xff),
    PlayfieldRow::new(0x30, 0x00, 0x00),
    PlayfieldRow::new(0x30, 0x00, 0x00),
    PlayfieldRow::new(0x30, 0x00, 0x00),
    PlayfieldRow::new(0x30, 0x00, 0x00),
    PlayfieldRow::new(0x30, 0x00, 0x00),
    PlayfieldRow::new(0xf0, 0xff, 0x0f),
];

static WALLS_BLACK_CASTLE_INTERIOR: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0xff, 0x0f),
    PlayfieldRow::new(0x30, 0x00, 0x00),
    PlayfieldRow::new(0x30, 0x00, 0x00),
    PlayfieldRow::new(0x30, 0x00, 0x00),
    PlayfieldRow::new(0x30, 0x00, 0x00),
    PlayfieldRow::new(0x30, 0x00, 0x00),
    PlayfieldRow::new(0xf0, 0xff, 0x0f),
];

static WALLS_BLUE_MAZE_NORTH: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0xff, 0x0f),
    PlayfieldRow::new(0x00, 0x0c, 0x0c),
    PlayfieldRow::new(0xf0, 0x0c, 0x3c),
    PlayfieldRow::new(0xf0, 0x0c, 0x00),
    PlayfieldRow::new(0xf0, 0xff, 0x3f),
    PlayfieldRow::new(0x00, 0x30, 0x30),
    PlayfieldRow::new(0xf0, 0x33, 0x3f),
];

static WALLS_BLUE_MAZE_05: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0xff, 0xff),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0xf0, 0xfc, 0xff),
    PlayfieldRow::new(0xf0, 0x00, 0xc0),
    PlayfieldRow::new(0xf0, 0x3f, 0xcf),
    PlayfieldRow::new(0x00, 0x30, 0xcc),
    PlayfieldRow::new(0xf0, 0xf3, 0xcc),
];

static WALLS_BLUE_MAZE_SOUTH: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0xf3, 0x0c),
    PlayfieldRow::new(0x00, 0x30, 0x0c),
    PlayfieldRow::new(0xf0, 0x3f, 0x0f),
    PlayfieldRow::new(0xf0, 0x00, 0x00),
    PlayfieldRow::new(0xf0, 0xf0, 0x00),
    PlayfieldRow::new(0x00, 0x30, 0x00),
    PlayfieldRow::new(0xf0, 0xff, 0xff),
];

static WALLS_BLUE_MAZE_CENTER: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0x33, 0x3f),
    PlayfieldRow::new(0x00, 0x30, 0x3c),
    PlayfieldRow::new(0xf0, 0xff, 0x3c),
    PlayfieldRow::new(0x00, 0x03, 0x3c),
    PlayfieldRow::new(0xf0, 0x33, 0x3c),
    PlayfieldRow::new(0x00, 0x33, 0x0c),
    PlayfieldRow::new(0xf0, 0xf3, 0x0c),
];

static WALLS_BLUE_MAZE_ENTRANCE: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0xf3, 0xcc),
    PlayfieldRow::new(0x00, 0x33, 0x0c),
    PlayfieldRow::new(0xf0, 0x33, 0xfc),
    PlayfieldRow::new(0x00, 0x33, 0x00),
    PlayfieldRow::new(0xf0, 0xf3, 0xff),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0xf0, 0xff, 0x0f),
];

static WALLS_CATACOMBS_CENTER: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0xff, 0xcc),
    PlayfieldRow::new(0x00, 0x00, 0xcc),
    PlayfieldRow::new(0xf0, 0x03, 0xcf),
    PlayfieldRow::new(0x00, 0x03, 0x00),
    PlayfieldRow::new(0xf0, 0xf3, 0xfc),
    PlayfieldRow::new(0x00, 0x33, 0x0c),
    PlayfieldRow::new(0xf0, 0x33, 0xcc),
];

static WALLS_CATACOMBS_SIDE: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0x33, 0xcc),
    PlayfieldRow::new(0x00, 0x30, 0xcc),
    PlayfieldRow::new(0x00, 0x3f, 0xcf),
    PlayfieldRow::new(0x00, 0x00, 0xc0),
    PlayfieldRow::new(0x00, 0x3f, 0xc3),
    PlayfieldRow::new(0x00, 0x30, 0xc0),
    PlayfieldRow::new(0xf0, 0xff, 0xff),
];

static WALLS_CATACOMBS_ENTRANCE: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0xff, 0x0f),
    PlayfieldRow::new(0x00, 0x30, 0x00),
    PlayfieldRow::new(0xf0, 0x30, 0xff),
    PlayfieldRow::new(0x00, 0x30, 0xc0),
    PlayfieldRow::new(0xf0, 0xf3, 0xc0),
    PlayfieldRow::new(0x00, 0x03, 0xc0),
    PlayfieldRow::new(0xf0, 0xff, 0xcc),
];

static WALLS_CASTLE: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0xfe, 0x15),
    PlayfieldRow::new(0x30, 0x03, 0x1f),
    PlayfieldRow::new(0x30, 0x03, 0xff),
    PlayfieldRow::new(0x30, 0x00, 0xff),
    PlayfieldRow::new(0x30, 0x00, 0x3f),
    PlayfieldRow::new(0x30, 0x00, 0x00),
    PlayfieldRow::new(0xf0, 0xff, 0x0f),
];

static WALLS_RED_MAZE_17: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0xff, 0xff),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0xf0, 0xff, 0x0f),
    PlayfieldRow::new(0x00, 0x00, 0x0c),
    PlayfieldRow::new(0xf0, 0xff, 0x0c),
    PlayfieldRow::new(0xf0, 0x03, 0xcc),
    PlayfieldRow::new(0xf0, 0x33, 0xcf),
];

static WALLS_RED_MAZE_SOUTH: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0x33, 0xcf),
    PlayfieldRow::new(0xf0, 0x30, 0x00),
    PlayfieldRow::new(0xf0, 0x33, 0xff),
    PlayfieldRow::new(0x00, 0x33, 0x00),
    PlayfieldRow::new(0xf0, 0xff, 0x00),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0xf0, 0xff, 0x0f),
];

static WALLS_RED_MAZE_NORTH: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0xff, 0xff),
    PlayfieldRow::new(0x00, 0x00, 0xc0),
    PlayfieldRow::new(0xf0, 0xff, 0xcf),
    PlayfieldRow::new(0x00, 0x00, 0xcc),
    PlayfieldRow::new(0xf0, 0x33, 0xff),
    PlayfieldRow::new(0xf0, 0x33, 0x00),
    PlayfieldRow::new(0xf0, 0x3f, 0x0c),
];

static WALLS_WHITE_CASTLE_INTERIOR: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0x3f, 0x0c),
    PlayfieldRow::new(0xf0, 0x00, 0x0c),
    PlayfieldRow::new(0xf0, 0xff, 0x0f),
    PlayfieldRow::new(0x00, 0x30, 0x00),
    PlayfieldRow::new(0xf0, 0x30, 0x00),
    PlayfieldRow::new(0x00, 0x30, 0x00),
    PlayfieldRow::new(0xf0, 0xff, 0x0f),
];

static WALLS_TOP_OPENING: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0xff, 0x0f),
    PlayfieldRow::new(0x30, 0x00, 0x00),
    PlayfieldRow::new(0x30, 0x00, 0x00),
    PlayfieldRow::new(0x30, 0x00, 0x00),
    PlayfieldRow::new(0x30, 0x00, 0x00),
    PlayfieldRow::new(0x30, 0x00, 0x00),
    PlayfieldRow::new(0xf0, 0xff, 0xff),
];

static WALLS_BLACK_MAZE_13: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0xf0, 0xff),
    PlayfieldRow::new(0x00, 0x00, 0x03),
    PlayfieldRow::new(0xf0, 0xff, 0x03),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0x30, 0x3f, 0xff),
    PlayfieldRow::new(0x00, 0x30, 0x00),
    PlayfieldRow::new(0xf0, 0xf0, 0xff),
];

static WALLS_BLACK_MAZE_DOT: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0xf0, 0xff),
    PlayfieldRow::new(0x30, 0x00, 0x00),
    PlayfieldRow::new(0x30, 0x3f, 0xff),
    PlayfieldRow::new(0x00, 0x30, 0x00),
    PlayfieldRow::new(0xf0, 0xf0, 0xff),
    PlayfieldRow::new(0x30, 0x00, 0x03),
    PlayfieldRow::new(0xf0, 0xf0, 0xff),
];

static WALLS_BLACK_MAZE_14: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0xff, 0xff),
    PlayfieldRow::new(0x00, 0x00, 0xc0),
    PlayfieldRow::new(0xf0, 0xff, 0xcf),
    PlayfieldRow::new(0x00, 0x00, 0x0c),
    PlayfieldRow::new(0xf0, 0x0f, 0xff),
    PlayfieldRow::new(0x00, 0x0f, 0xc0),
    PlayfieldRow::new(0x30, 0xcf, 0xcc),
];

static WALLS_BLACK_MAZE_ENTRANCE: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0x30, 0xcf, 0xcc),
    PlayfieldRow::new(0x00, 0xc0, 0xcc),
    PlayfieldRow::new(0xf0, 0xff, 0x0f),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0xf0, 0xff, 0x0f),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0xf0, 0xff, 0x0f),
];

// Sprite artwork.

static GATE_IMAGE: [u8; 17] = [
    0xfe, // |███████ |
    0xaa, // |█ █ █ █ |
    0xfe, // |███████ |
    0xaa, // |█ █ █ █ |
    0xfe, // |███████ |
    0xaa, // |█ █ █ █ |
    0xfe, // |███████ |
    0xaa, // |█ █ █ █ |
    0xfe, // |███████ |
    0xaa, // |█ █ █ █ |
    0xfe, // |███████ |
    0xaa, // |█ █ █ █ |
    0xfe, // |███████ |
    0xaa, // |█ █ █ █ |
    0xfe, // |███████ |
    0xaa, // |█ █ █ █ |
    0x00, // |        |
];

static LIGHT_IMAGE: [u8; 33] = [
    0xff, // |████████|
    0xff, // |████████|
    0xff, // |████████|
    0xff, // |████████|
    0xff, // |████████|
    0xff, // |████████|
    0xff, // |████████|
    0xff, // |████████|
    0xff, // |████████|
    0xff, // |████████|
    0xff, // |████████|
    0xff, // |████████|
    0xff, // |████████|
    0xff, // |████████|
    0xff, // |████████|
    0xff, // |████████|
    0xff, // |████████|
    0xff, // |████████|
    0xff, // |████████|
    0xff, // |████████|
    0xff, // |████████|
    0xff, // |████████|
    0xff, // |████████|
    0xff, // |████████|
    0xff, // |████████|
    0xff, // |████████|
    0xff, // |████████|
    0xff, // |████████|
    0xff, // |████████|
    0xff, // |████████|
    0xff, // |████████|
    0xff, // |████████|
    0x00, // |        |
];

static BRIDGE_IMAGE: [u8; 25] = [
    0xc3, // |██    ██|
    0xc3, // |██    ██|
    0xc3, // |██    ██|
    0xc3, // |██    ██|
    0x42, // | █    █ |
    0x42, // | █    █ |
    0x42, // | █    █ |
    0x42, // | █    █ |
    0x42, // | █    █ |
    0x42, // | █    █ |
    0x42, // | █    █ |
    0x42, // | █    █ |
    0x42, // | █    █ |
    0x42, // | █    █ |
    0x42, // | █    █ |
    0x42, // | █    █ |
    0x42, // | █    █ |
    0x42, // | █    █ |
    0x42, // | █    █ |
    0x42, // | █    █ |
    0xc3, // |██    ██|
    0xc3, // |██    ██|
    0xc3, // |██    ██|
    0xc3, // |██    ██|
    0x00, // |        |
];

static GAME_ONE_IMAGE: [u8; 8] = [
    0x04, // |     █  |
    0x0c, // |    ██  |
    0x04, // |     █  |
    0x04, // |     █  |
    0x04, // |     █  |
    0x04, // |     █  |
    0x0e, // |    ███ |
    0x00, // |        |
];

static KEY_IMAGE: [u8; 4] = [
    0x07, // |     ███|
    0xfd, // |██████ █|
    0xa7, // |█ █  ███|
    0x00, // |        |
];

static GAME_TWO_IMAGE: [u8; 8] = [
    0x0e, // |    ███ |
    0x11, // |   █   █|
    0x01, // |       █|
    0x02, // |      █ |
    0x04, // |     █  |
    0x08, // |    █   |
    0x1f, // |   █████|
    0x00, // |        |
];

static GAME_THREE_IMAGE: [u8; 8] = [
    0x0e, // |    ███ |
    0x11, // |   █   █|
    0x01, // |       █|
    0x06, // |     ██ |
    0x01, // |       █|
    0x11, // |   █   █|
    0x0e, // |    ███ |
    0x00, // |        |
];

static BAT_FRAME_ONE: [u8; 8] = [
    0x81, // |█      █|
    0x81, // |█      █|
    0xc3, // |██    ██|
    0xc3, // |██    ██|
    0xff, // |████████|
    0x5a, // | █ ██ █ |
    0x66, // | ██  ██ |
    0x00, // |        |
];

static BAT_FRAME_TWO: [u8; 12] = [
    0x01, // |       █|
    0x80, // |█       |
    0x01, // |       █|
    0x80, // |█       |
    0x3c, // |  ████  |
    0x5a, // | █ ██ █ |
    0x66, // | ██  ██ |
    0xc3, // |██    ██|
    0x81, // |█      █|
    0x81, // |█      █|
    0x81, // |█      █|
    0x00, // |        |
];

static DRAGON_WALKING_IMAGE: [u8; 21] = [
    0x06, // |     ██ |
    0x0f, // |    ████|
    0xf3, // |████  ██|
    0xfe, // |███████ |
    0x0e, // |    ███ |
    0x04, // |     █  |
    0x04, // |     █  |
    0x1e, // |   ████ |
    0x3f, // |  ██████|
    0x7f, // | ███████|
    0xe3, // |███   ██|
    0xc3, // |██    ██|
    0xc3, // |██    ██|
    0xc7, // |██   ███|
    0xff, // |████████|
    0x3c, // |  ████  |
    0x08, // |    █   |
    0x8f, // |█   ████|
    0xe1, // |███    █|
    0x3f, // |  ██████|
    0x00, // |        |
];

static DRAGON_BITING_IMAGE: [u8; 23] = [
    0x80, // |█       |
    0x40, // | █      |
    0x26, // |  █  ██ |
    0x1f, // |   █████|
    0x0b, // |    █ ██|
    0x0e, // |    ███ |
    0x1e, // |   ████ |
    0x24, // |  █  █  |
    0x44, // | █   █  |
    0x8e, // |█   ███ |
    0x1e, // |   ████ |
    0x3f, // |  ██████|
    0x7f, // | ███████|
    0x7f, // | ███████|
    0x7f, // | ███████|
    0x7f, // | ███████|
    0x3e, // |  █████ |
    0x1c, // |   ███  |
    0x08, // |    █   |
    0xf8, // |█████   |
    0x80, // |█       |
    0xe0, // |███     |
    0x00, // |        |
];

static DRAGON_DEAD_IMAGE: [u8; 18] = [
    0x0c, // |    ██  |
    0x0c, // |    ██  |
    0x0c, // |    ██  |
    0x0e, // |    ███ |
    0x1b, // |   ██ ██|
    0x7f, // | ███████|
    0xce, // |██  ███ |
    0x80, // |█       |
    0xfc, // |██████  |
    0xfe, // |███████ |
    0xfe, // |███████ |
    0x7e, // | ██████ |
    0x78, // | ████   |
    0x20, // |  █     |
    0x6e, // | ██ ███ |
    0x42, // | █    █ |
    0x7e, // | ██████ |
    0x00, // |        |
];

static SWORD_IMAGE: [u8; 6] = [
    0x20, // |  █     |
    0x40, // | █      |
    0xff, // |████████|
    0x40, // | █      |
    0x20, // |  █     |
    0x00, // |        |
];

static DOT_IMAGE: [u8; 2] = [
    0x80, // |█       |
    0x00, // |        |
];

static ROBINETT_CREDITS_IMAGE: [u8; 96] = [
    0xf0, // |████    |
    0x80, // |█       |
    0x80, // |█       |
    0x80, // |█       |
    0xf4, // |████ █  |
    0x04, // |     █  |
    0x87, // |█    ███|
    0xe5, // |███  █ █|
    0x87, // |█    ███|
    0x80, // |█       |
    0x05, // |     █ █|
    0xe5, // |███  █ █|
    0xa7, // |█ █  ███|
    0xe1, // |███    █|
    0x87, // |█    ███|
    0xe0, // |███     |
    0x01, // |       █|
    0xe0, // |███     |
    0xa0, // |█ █     |
    0xf0, // |████    |
    0x01, // |       █|
    0x40, // | █      |
    0xe0, // |███     |
    0x40, // | █      |
    0x40, // | █      |
    0x40, // | █      |
    0x01, // |       █|
    0xe0, // |███     |
    0xa0, // |█ █     |
    0xe0, // |███     |
    0x80, // |█       |
    0xe0, // |███     |
    0x01, // |       █|
    0x20, // |  █     |
    0x20, // |  █     |
    0xe0, // |███     |
    0xa0, // |█ █     |
    0xe0, // |███     |
    0x01, // |       █|
    0x01, // |       █|
    0x01, // |       █|
    0x88, // |█   █   |
    0xa8, // |█ █ █   |
    0xa8, // |█ █ █   |
    0xa8, // |█ █ █   |
    0xf8, // |█████   |
    0x01, // |       █|
    0xe0, // |███     |
    0xa0, // |█ █     |
    0xf0, // |████    |
    0x01, // |       █|
    0x80, // |█       |
    0xe0, // |███     |
    0x8f, // |█   ████|
    0x89, // |█   █  █|
    0x0f, // |    ████|
    0x8a, // |█   █ █ |
    0xe9, // |███ █  █|
    0x80, // |█       |
    0x8e, // |█   ███ |
    0x0a, // |    █ █ |
    0xee, // |███ ███ |
    0xa0, // |█ █     |
    0xe8, // |███ █   |
    0x88, // |█   █   |
    0xee, // |███ ███ |
    0x0a, // |    █ █ |
    0x8e, // |█   ███ |
    0xe0, // |███     |
    0xa4, // |█ █  █  |
    0xa4, // |█ █  █  |
    0x04, // |     █  |
    0x80, // |█       |
    0x08, // |    █   |
    0x0e, // |    ███ |
    0x0a, // |    █ █ |
    0x0a, // |    █ █ |
    0x80, // |█       |
    0x0e, // |    ███ |
    0x0a, // |    █ █ |
    0x0e, // |    ███ |
    0x08, // |    █   |
    0x0e, // |    ███ |
    0x80, // |█       |
    0x04, // |     █  |
    0x0e, // |    ███ |
    0x04, // |     █  |
    0x04, // |     █  |
    0x04, // |     █  |
    0x80, // |█       |
    0x04, // |     █  |
    0x0e, // |    ███ |
    0x04, // |     █  |
    0x04, // |     █  |
    0x04, // |     █  |
    0x00, // |        |
];

static CHALICE_IMAGE: [u8; 10] = [
    0x81, // |█      █|
    0x81, // |█      █|
    0xc3, // |██    ██|
    0x7e, // | ██████ |
    0x7e, // | ██████ |
    0x3c, // |  ████  |
    0x18, // |   ██   |
    0x18, // |   ██   |
    0x7e, // | ██████ |
    0x00, // |        |
];

static EMPTY_IMAGE: [u8; 1] = [
    0x00, // |        |
];

static MAGNET_IMAGE: [u8; 9] = [
    0x3c, // |  ████  |
    0x7e, // | ██████ |
    0xe7, // |███  ███|
    0xc3, // |██    ██|
    0xc3, // |██    ██|
    0xc3, // |██    ██|
    0xc3, // |██    ██|
    0xc3, // |██    ██|
    0x00, // |        |
];

#[cfg(test)]
#[path = "tests/adventure.rs"]
mod tests;
