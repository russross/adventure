//! World simulation, picture scheduling, rendering, and immutable game data.

use crate::hardware::FrameHost;
use crate::input::{Controls, Difficulty, Direction};
use crate::sound::{Sound, SoundEffect};
use crate::video::{Appearance, ObjectLine, PlayfieldRow, Scanline, SpriteWidth, Video, WIDTH};
use adventure_trace::{Control, Point, Session};
use std::cell::RefCell;

adventure_trace::port_adapter!(world, video);

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
        matches!(
            self,
            Self::CatacombsCenter
                | Self::CatacombsEntrance
                | Self::CatacombsSide
                | Self::BlackMaze13
                | Self::BlackMaze14
                | Self::BlackMazeDot
                | Self::BlackMazeEntrance
        )
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
    Empty,
}

const OBJECT_IDS: [ObjectId; 19] = [
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
    const fn new(room: Room, x: u8, y: u8) -> Self {
        Self { room, x, y }
    }
}

impl ObjectId {
    // Scenery has no movable record. Empty retains the bridge position for display selection.
    fn record(self) -> Option<RecordId> {
        match self {
            Self::Light => Some(RecordId::Light),
            Self::Rhindle => Some(RecordId::Rhindle),
            Self::Yorgle => Some(RecordId::Yorgle),
            Self::Grundle => Some(RecordId::Grundle),
            Self::Sword => Some(RecordId::Sword),
            Self::Bridge | Self::Empty => Some(RecordId::Bridge),
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MovementPhase {
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

struct PreparedDisplay {
    appearance: Appearance,
    objects: [ObjectLine; 2],
    player_visible: bool,
    room_walls: &'static [PlayfieldRow; 7],
    first_image: Bitmap,
    second_image: Bitmap,
    first_position: Position,
    second_position: Position,
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

// Startup and picture scheduling.

#[derive(Default)]
pub struct Adventure {
    pub world: World,
    pub video: Video,
    pub sound: Sound,
    pub controls: Controls,
    pub trace: Option<RefCell<Session>>,
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
                    self.world.sound_remaining = 255;
                    self.world.won = true;
                    self.world.sound_effect = SoundEffect::Victory;
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
                self.update_grundle();
                self.trace_checkpoint();
                self.trace_at(Point::Yorgle);
                self.update_yorgle();
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
                self.update_rhindle();
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
        self.trace_input(Control::Reset);
        if self.controls.reset && !self.world.previous_reset {
            if self.world.won {
                return self.initialize_world(host);
            }
            self.world.player = Position::new(Room::GoldCastle, 80, 32);
            self.world.safe_player = self.world.player;
            self.world.rhindle.state = DragonState::Wandering;
            self.world.yorgle.state = DragonState::Wandering;
            self.world.grundle.state = DragonState::Wandering;
            self.world.sound_remaining = 0;
            self.world.held = ObjectId::Empty;
        }
        self.trace_input(Control::Select);
        if self.controls.select && !self.world.previous_select {
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
        self.world.held = ObjectId::Empty;
        self.remember_console_switches();
        self.trace_checkpoint();
        Ok(true)
    }

    // The shared console-poll tail needs no stack or shared pointer state.
    fn remember_console_switches(&mut self) {
        self.trace_at(Point::History);
        self.trace_input(Control::Reset);
        self.trace_input(Control::Select);
        self.world.previous_reset = self.controls.reset;
        self.world.previous_select = self.controls.select;
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
        self.trace_input(Control::LeftDirection);
        self.trace_input(Control::RightDirection);
        if self.controls.left.direction.is_idle() && self.controls.right.direction.is_idle() {
            self.trace_input(Control::Reset);
            self.trace_input(Control::Select);
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
    // Roll back blocked movement except within the bridge's traversable corridor.
    fn move_player(&mut self, phase: MovementPhase) {
        self.trace_at(match phase {
            MovementPhase::Both => Point::PlayerBoth,
            MovementPhase::Vertical => Point::PlayerVertical,
            MovementPhase::Horizontal => Point::PlayerHorizontal,
        });
        let blocked = self.trace_blocked();
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
            self.trace_input(Control::LeftDirection);
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
        let first_game = self.world.game == Game::One;
        match (room, edge) {
            (Room::GameSelection | Room::GoldTreasury, _) => room,
            (Room::WestHall, RoomEdge::North) => Room::BlueMazeEntrance,
            (Room::WestHall, RoomEdge::East) => Room::BelowGoldCastle,
            (Room::WestHall, RoomEdge::South) => {
                if first_game {
                    Room::BlackCastle
                } else {
                    Room::WhiteCastle
                }
            }
            (Room::WestHall, RoomEdge::West) => Room::SecretPassage,
            (Room::BelowGoldCastle, RoomEdge::North) => Room::GoldCastle,
            (Room::BelowGoldCastle, RoomEdge::East) => Room::SecretPassage,
            (Room::BelowGoldCastle, RoomEdge::South) => {
                if first_game {
                    Room::BlueMaze05
                } else {
                    Room::GoldCastle
                }
            }
            (Room::BelowGoldCastle, RoomEdge::West) => Room::WestHall,
            (Room::SecretPassage, RoomEdge::North) => Room::BlueMazeSouth,
            (Room::SecretPassage, RoomEdge::East) => Room::WestHall,
            (Room::SecretPassage, RoomEdge::South) => {
                if first_game {
                    Room::RedHall
                } else {
                    Room::CatacombsEntrance
                }
            }
            (Room::SecretPassage, RoomEdge::West) => Room::BelowGoldCastle,

            // The blue maze connects the three outdoor halls.
            (Room::BlueMazeNorth, RoomEdge::North) => Room::BlackCastle,
            (Room::BlueMazeNorth, RoomEdge::East) => Room::BlueMaze05,
            (Room::BlueMazeNorth, RoomEdge::South) => Room::BlueMazeCenter,
            (Room::BlueMazeNorth, RoomEdge::West) => Room::BlueMazeSouth,
            (Room::BlueMaze05, RoomEdge::North) => Room::RedHall,
            (Room::BlueMaze05, RoomEdge::East) => Room::BlueMazeSouth,
            (Room::BlueMaze05, RoomEdge::South) => Room::BlueMazeEntrance,
            (Room::BlueMaze05, RoomEdge::West) => Room::BlueMazeNorth,
            (Room::BlueMazeSouth, RoomEdge::North) => Room::BlueMazeCenter,
            (Room::BlueMazeSouth, RoomEdge::East) => Room::BlueMazeNorth,
            (Room::BlueMazeSouth, RoomEdge::South) => Room::SecretPassage,
            (Room::BlueMazeSouth, RoomEdge::West) => Room::BlueMaze05,
            (Room::BlueMazeCenter, RoomEdge::North) => Room::BlueMazeNorth,
            (Room::BlueMazeCenter, RoomEdge::East | RoomEdge::West) => Room::BlueMazeEntrance,
            (Room::BlueMazeCenter, RoomEdge::South) => Room::BlueMazeSouth,
            (Room::BlueMazeEntrance, RoomEdge::North) => Room::BlueMaze05,
            (Room::BlueMazeEntrance, RoomEdge::East | RoomEdge::West) => Room::BlueMazeCenter,
            (Room::BlueMazeEntrance, RoomEdge::South) => Room::WestHall,

            // The catacombs open onto the side passages and the purple chamber.
            (Room::CatacombsCenter, RoomEdge::North | RoomEdge::East | RoomEdge::West) => {
                Room::CatacombsEntrance
            }
            (Room::CatacombsCenter, RoomEdge::South) => Room::CatacombsSide,
            (Room::CatacombsEntrance, RoomEdge::North) => Room::SecretPassage,
            (Room::CatacombsEntrance, RoomEdge::East | RoomEdge::South | RoomEdge::West) => {
                Room::CatacombsCenter
            }
            (Room::CatacombsSide, RoomEdge::North) => Room::CatacombsCenter,
            (Room::CatacombsSide, RoomEdge::East) => Room::SidePassage0c,
            (Room::CatacombsSide, RoomEdge::South) => Room::PurpleChamber,
            (Room::CatacombsSide, RoomEdge::West) => Room::SidePassage0d,
            (Room::SidePassage0c, RoomEdge::North) => Room::PurpleChamber,
            (Room::SidePassage0c, RoomEdge::East) => Room::SidePassage0d,
            (Room::SidePassage0c, RoomEdge::South) => Room::RedHall,
            (Room::SidePassage0c, RoomEdge::West) => Room::CatacombsSide,
            (Room::SidePassage0d, RoomEdge::North) => Room::WhiteCastle,
            (Room::SidePassage0d, RoomEdge::East) => Room::CatacombsSide,
            (Room::SidePassage0d, RoomEdge::South) => Room::AboveWhiteCastle,
            (Room::SidePassage0d, RoomEdge::West) => Room::SidePassage0c,

            // Exterior castle rooms use ordinary map edges outside the gate opening.
            (Room::AboveWhiteCastle, RoomEdge::North) => Room::SidePassage0d,
            (Room::AboveWhiteCastle, RoomEdge::East | RoomEdge::West) => Room::BlackCastle,
            (Room::AboveWhiteCastle, RoomEdge::South) => Room::WhiteCastle,
            (Room::WhiteCastle, RoomEdge::North) => Room::AboveWhiteCastle,
            (Room::WhiteCastle, RoomEdge::East | RoomEdge::West) => Room::WhiteCastle,
            (Room::WhiteCastle, RoomEdge::South) => Room::SidePassage0d,
            (Room::BlackCastle, RoomEdge::North) => Room::WestHall,
            (Room::BlackCastle, RoomEdge::East | RoomEdge::West) => Room::PurpleChamber,
            (Room::BlackCastle, RoomEdge::South) => Room::BlueMazeNorth,
            (Room::GoldCastle, RoomEdge::North) => Room::BlueMazeSouth,
            (Room::GoldCastle, RoomEdge::East) => Room::SecretPassage,
            (Room::GoldCastle, RoomEdge::South) => Room::BelowGoldCastle,
            (Room::GoldCastle, RoomEdge::West) => Room::WestHall,

            // Black Castle contains the dark maze; White Castle contains the red maze.
            (Room::BlackMaze13, RoomEdge::North | RoomEdge::South) => Room::BlackMazeDot,
            (Room::BlackMaze13, RoomEdge::East) => Room::BlackMaze14,
            (Room::BlackMaze13, RoomEdge::West) => Room::BlackMazeEntrance,
            (Room::BlackMaze14, RoomEdge::North | RoomEdge::South) => Room::BlackMazeEntrance,
            (Room::BlackMaze14, RoomEdge::East) => Room::BlackMazeDot,
            (Room::BlackMaze14, RoomEdge::West) => Room::BlackMaze13,
            (Room::BlackMazeDot, RoomEdge::North | RoomEdge::South) => Room::BlackMaze13,
            (Room::BlackMazeDot, RoomEdge::East) => Room::BlackMazeEntrance,
            (Room::BlackMazeDot, RoomEdge::West) => Room::BlackMaze14,
            (Room::BlackMazeEntrance, RoomEdge::North) => Room::BlackMaze14,
            (Room::BlackMazeEntrance, RoomEdge::East) => Room::BlackMaze13,
            (Room::BlackMazeEntrance, RoomEdge::South) => Room::BlackCastleInterior,
            (Room::BlackMazeEntrance, RoomEdge::West) => Room::BlackMazeDot,
            (Room::RedMaze17, RoomEdge::North | RoomEdge::South) => Room::RedMazeSouth,
            (Room::RedMaze17, RoomEdge::East | RoomEdge::West) => Room::RedMazeNorth,
            (Room::RedMazeNorth, RoomEdge::North | RoomEdge::South) => Room::WhiteCastleInterior,
            (Room::RedMazeNorth, RoomEdge::East | RoomEdge::West) => Room::RedMaze17,
            (Room::RedMazeSouth, RoomEdge::North | RoomEdge::South) => Room::RedMaze17,
            (Room::RedMazeSouth, RoomEdge::East | RoomEdge::West) => Room::WhiteCastleInterior,
            (Room::WhiteCastleInterior, RoomEdge::North | RoomEdge::South) => Room::RedMazeNorth,
            (Room::WhiteCastleInterior, RoomEdge::East | RoomEdge::West) => Room::RedMazeSouth,
            (Room::BlackCastleInterior, _) => {
                if first_game {
                    Room::PurpleChamber
                } else {
                    Room::BlackMazeEntrance
                }
            }

            // The remaining chambers include the dot's separate, conditional credits entrance.
            (Room::PurpleChamber, RoomEdge::North) => Room::RedHall,
            (Room::PurpleChamber, RoomEdge::East) => Room::BlueMazeCenter,
            (Room::PurpleChamber, RoomEdge::South) => {
                if first_game {
                    Room::BlackCastleInterior
                } else {
                    Room::SidePassage0c
                }
            }
            (Room::PurpleChamber, RoomEdge::West) => Room::BlueMazeEntrance,
            (Room::RedHall, RoomEdge::North) => {
                if first_game {
                    Room::SecretPassage
                } else {
                    Room::SidePassage0c
                }
            }
            (Room::RedHall, RoomEdge::East) => Room::WestHall,
            (Room::RedHall, RoomEdge::South) => Room::BlackCastle,
            (Room::RedHall, RoomEdge::West) => Room::SecretPassage,
            (Room::RobinettCredits, RoomEdge::North | RoomEdge::South) => Room::BlueMazeSouth,
            (Room::RobinettCredits, RoomEdge::East) => Room::WestHall,
            (Room::RobinettCredits, RoomEdge::West) => Room::SecretPassage,
        }
    }
}

// Pickup, gates, magnet, and light.

impl Adventure {
    // Fire drops an item; the last rendered contacts determine pickup candidates.
    fn handle_pickup_and_drop(&mut self) {
        self.trace_at(Point::Pickup);
        self.trace_input(Control::Fire);
        let newly_pressed = self.controls.left.fire && !self.world.fire_was_pressed;
        self.world.fire_was_pressed = self.controls.left.fire;
        if newly_pressed && self.world.held != ObjectId::Empty {
            self.world.held = ObjectId::Empty;
            self.world.sound_effect = SoundEffect::Drop;
            self.world.sound_remaining = 4;
            self.trace_checkpoint();
        }
        let touched = if self.trace_player_slot(0) {
            self.world.first_visible
        } else if self.trace_player_slot(1) {
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
        let Some(record) = touched.record() else {
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
    fn move_held_object(&mut self) {
        if self.world.held == ObjectId::Empty {
            return;
        }
        let Some(record) = self.world.held.record() else {
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
    fn is_touching_player(&self, object: ObjectId) -> bool {
        if object == self.world.first_visible {
            self.trace_player_slot(0)
        } else if object == self.world.second_visible {
            self.trace_player_slot(1)
        } else {
            false
        }
    }

    // Return the other member of a displayed object pair that touched.
    fn find_touching_object(&self, object: ObjectId) -> ObjectId {
        self.trace_object_contact();
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
    fn update_castle_gates(&mut self) {
        for gate in [GateId::Black, GateId::White, GateId::Gold] {
            let (object, key, inside, _) = gate.castle();
            let touched = self.find_touching_object(object);
            if touched == key {
                self.world.gates.set(
                    gate,
                    GateState::from_value(self.world.gates.get(gate).value().wrapping_add(1)),
                );
            }
            if self.world.gates.get(gate) != GateState::Closed {
                let record = if self.is_touching_player(object) {
                    self.world.gates.set(gate, GateState::Open);
                    Some(RecordId::Player)
                } else if touched != ObjectId::Empty {
                    touched.record()
                } else {
                    None
                };
                if let Some(record) = record {
                    self.world.position_mut(record).room = inside;
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
    fn update_magnet(&mut self) {
        self.world.magnet.y = self.world.magnet.y.wrapping_sub(8);
        // The first metal object in the room wins, even if it is already at the target.
        for item in [
            RecordId::GoldKey,
            RecordId::WhiteKey,
            RecordId::BlackKey,
            RecordId::Sword,
            RecordId::Bridge,
            RecordId::Chalice,
        ] {
            if let Some(direction) = self.direction_toward(item, RecordId::Magnet) {
                self.move_object_through_world(item, direction, 1);
                break;
            }
        }
        self.world.magnet.y = self.world.magnet.y.wrapping_add(8);
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
    // An absent target retains motion; a target at the same position supplies idle motion.
    fn direction_toward(&self, from: RecordId, to: RecordId) -> Option<Direction> {
        let from = self.world.position(from);
        let to = self.world.position(to);
        if from.room != to.room {
            return None;
        }
        Some(Direction {
            right: to.x > from.x,
            left: to.x < from.x,
            up: to.y > from.y,
            down: to.y < from.y,
        })
    }

    // Dragons flee their fears before pursuing the player or their guarded treasures.
    fn dragon_direction(&self, dragon: DragonId) -> Direction {
        let record = dragon.record();
        if self.controls.right_difficulty == Difficulty::A
            && let Some(direction) = self.direction_toward(RecordId::Sword, record)
        {
            return direction;
        }
        if dragon == DragonId::Yorgle
            && let Some(direction) = self.direction_toward(RecordId::GoldKey, record)
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
            .find_map(|&target| self.direction_toward(record, target))
            .unwrap_or(self.world.dragon(dragon).direction)
    }

    // Rhindle moves three steps using his own priorities.
    fn update_rhindle(&mut self) {
        self.update_dragon(DragonId::Rhindle);
    }

    // Yorgle moves two steps and includes the Gold Key among his fears.
    fn update_yorgle(&mut self) {
        self.update_dragon(DragonId::Yorgle);
    }

    // Grundle moves two steps using his guarding priorities.
    fn update_grundle(&mut self) {
        self.update_dragon(DragonId::Grundle);
    }

    // Preserve wandering, sword contact, swallowing, and delayed-bite branch order.
    fn update_dragon(&mut self, dragon: DragonId) {
        let object = dragon.object();
        match self.world.dragon(dragon).state {
            DragonState::Wandering => {
                self.trace_input(Control::RightDifficulty);
                let speed = match dragon {
                    DragonId::Rhindle => 3,
                    DragonId::Yorgle | DragonId::Grundle => 2,
                };
                let direction = self.dragon_direction(dragon);
                self.world.dragon_mut(dragon).direction = direction;
                self.move_object_through_world(dragon.record(), direction, speed);
                if self.is_touching_player(object) {
                    self.trace_input(Control::LeftDifficulty);
                    let delay = match (self.world.game, self.controls.left_difficulty) {
                        (Game::One, Difficulty::B) => 0xd0,
                        (Game::One, Difficulty::A) => 0xe8,
                        (Game::Two | Game::Three, Difficulty::B) => 0xf0,
                        (Game::Two | Game::Three, Difficulty::A) => 0xf6,
                    };
                    self.world.dragon_mut(dragon).state = DragonState::Biting(delay);
                    self.world.dragon_mut(dragon).position.x = self.world.safe_player.x;
                    self.world.dragon_mut(dragon).position.y = self.world.safe_player.y;
                    self.world.sound_effect = SoundEffect::Roar;
                    self.world.sound_remaining = 16;
                }
                if self.find_touching_object(object) == ObjectId::Sword {
                    self.world.dragon_mut(dragon).state = DragonState::Dead;
                    self.world.sound_effect = SoundEffect::DragonSlain;
                    self.world.sound_remaining = 16;
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
                let count = count.wrapping_add(1);
                self.world.dragon_mut(dragon).state = DragonState::from_value(count);
                if count < 252 || !self.is_touching_player(object) {
                    return;
                }
                self.world.dragon_mut(dragon).state = DragonState::Swallowed;
                self.world.sound_effect = SoundEffect::Swallow;
                self.world.sound_remaining = 16;
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
                RecordId::Chalice,
                RecordId::Sword,
                RecordId::Bridge,
                RecordId::GoldKey,
                RecordId::WhiteKey,
                RecordId::BlackKey,
                RecordId::Rhindle,
                RecordId::Yorgle,
                RecordId::Grundle,
                RecordId::Magnet,
            ]
            .into_iter()
            .find(|&target| {
                self.world.bat.carried != Some(RecordId::Bat)
                    && self.world.bat.carried != Some(target)
                    && self.world.position(target).room == self.world.bat.position.room
            });
            if let Some(target) = target {
                self.world.bat.direction = self
                    .direction_toward(RecordId::Bat, target)
                    .expect("the bat target is in its room");
            }
            self.move_object_through_world(RecordId::Bat, self.world.bat.direction, 3);
            if let Some(record) = target {
                let target = self.world.position(record);
                let bat = self.world.bat.position;
                if target.room == bat.room
                    && target.x.wrapping_sub(bat.x).wrapping_add(4) & 0xf8 == 0
                    && target.y.wrapping_sub(bat.y).wrapping_add(4) & 0xf8 == 0
                {
                    self.world.bat.carried = Some(record);
                    self.world.bat.flight_timer = 16;
                }
            }
        }
        if let Some(cargo) = self.world.bat.carried {
            let bat = self.world.bat.position;
            *self.world.position_mut(cargo) = Position::new(bat.room, bat.x.wrapping_add(8), bat.y);
            if self.trace_crossed(RecordId::Bat) {
                self.trace_transition(cargo);
            }
            if self.world.held.record() == Some(cargo) {
                self.world.held = ObjectId::Empty;
            }
        }
    }
}

// Game sound effects.

impl Adventure {
    // Duration advances before voice selection; victory also animates the retained room color.
    fn play_game_sound(&mut self) {
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

// Display preparation and scanline construction.

impl Adventure {
    // Each logical row emits the retained line followed by the newly prepared line.
    fn draw_room(&mut self) {
        let mut line = Scanline {
            appearance: self.display.appearance,
            walls: self.display.room_walls[0],
            objects: self.display.objects,
            player_x: screen_x(positioned_object_x(self.world.player.x) - 1),
            player_visible: self.display.player_visible,
        };
        line.objects[0].x = object_screen_x(self.display.first_position.x, line.objects[0].width);
        line.objects[1].x = object_screen_x(self.display.second_position.x, line.objects[1].width);
        line.objects[0].bits = 0;
        line.objects[1].bits = 0;
        self.video.begin_picture();
        let player_y = self.world.player.y.wrapping_sub(4);
        let mut cursors = [0; 2];
        let mut staged_second = 0;
        let mut band = 1;

        // The first retained line skips the second image fetch. Zero terminators stop cursors.
        for row in (8u8..=104).rev() {
            if row != 104 && below(row, self.display.second_position.y) {
                staged_second = self.display.second_image.row(cursors[1]);
                cursors[1] += usize::from(staged_second != 0);
            }
            let first = if below(row, self.display.first_position.y) {
                let bits = self.display.first_image.row(cursors[0]);
                cursors[0] += usize::from(bits != 0);
                bits
            } else {
                0
            };
            self.video.paint(&line);

            // Commit both object rows together; wall changes affect the second line only.
            line.objects[0].bits = first;
            line.objects[1].bits = staged_second;
            line.player_visible = row.wrapping_sub(player_y) & 0xfc == 0;
            if row & 15 == 0 {
                line.walls = self.display.room_walls[band];
                band += 1;
            }
            self.video.paint(&line);
        }
        self.display.player_visible = line.player_visible;
    }

    // Appearance and images are a snapshot, reused by the following two pictures.
    fn prepare_room_display(&mut self) {
        self.trace_at(Point::Display);
        self.trace_input(Control::Color);
        let (walls, appearance) = self.room_appearance(self.world.player.room);
        self.display.room_walls = walls;
        self.display.appearance = appearance;
        self.select_visible_objects();
        if self.world.first_visible == ObjectId::Light
            || self.world.first_visible == ObjectId::Bridge
                && self.world.second_visible != ObjectId::Light
        {
            std::mem::swap(
                &mut self.world.first_visible,
                &mut self.world.second_visible,
            );
        }

        // Resolve the live object's appearance once; following pictures reuse this snapshot.
        for (slot, object) in [self.world.first_visible, self.world.second_visible]
            .into_iter()
            .enumerate()
        {
            let display = self.object_display(object);
            self.display.objects[slot] = ObjectLine {
                x: object_screen_x(display.position.x, display.width),
                color: display.color,
                width: display.width,
                bits: 0,
            };
            if slot == 0 {
                self.display.first_position = display.position;
                self.display.first_image = display.bitmap;
            } else {
                self.display.second_position = display.position;
                self.display.second_image = display.bitmap;
            }
        }
    }

    // Rotate over the real inhabitants, retaining at most two in this room.
    fn select_visible_objects(&mut self) {
        self.world.first_visible = ObjectId::Empty;
        self.world.second_visible = ObjectId::Empty;
        let mut candidate = self.world.last_candidate;
        loop {
            candidate = OBJECT_IDS[(candidate as usize + 1) % 18];
            if self.find_object_position(candidate).room == self.world.player.room {
                if self.world.first_visible == ObjectId::Empty {
                    self.world.first_visible = candidate;
                } else {
                    self.world.second_visible = candidate;
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
    fn room_appearance(&self, room: Room) -> (&'static [PlayfieldRow; 7], Appearance) {
        let walls = match room {
            Room::GameSelection | Room::GoldTreasury | Room::PurpleChamber => &WALLS_ENCLOSED_ROOM,
            Room::WestHall | Room::BelowGoldCastle | Room::RobinettCredits => &WALLS_OPEN_HALL,
            Room::SecretPassage => &WALLS_SECRET_PASSAGE,
            Room::BlueMazeNorth => &WALLS_BLUE_MAZE_NORTH,
            Room::BlueMaze05 => &WALLS_BLUE_MAZE_05,
            Room::BlueMazeSouth => &WALLS_BLUE_MAZE_SOUTH,
            Room::BlueMazeCenter => &WALLS_BLUE_MAZE_CENTER,
            Room::BlueMazeEntrance => &WALLS_BLUE_MAZE_ENTRANCE,
            Room::CatacombsCenter => &WALLS_CATACOMBS_CENTER,
            Room::CatacombsEntrance => &WALLS_CATACOMBS_ENTRANCE,
            Room::CatacombsSide => &WALLS_CATACOMBS_SIDE,
            Room::SidePassage0c | Room::SidePassage0d => &WALLS_SIDE_PASSAGE,
            Room::AboveWhiteCastle | Room::RedHall => &WALLS_TOP_OPENING,
            Room::WhiteCastle | Room::BlackCastle | Room::GoldCastle => &WALLS_CASTLE,
            Room::BlackMaze13 => &WALLS_BLACK_MAZE_13,
            Room::BlackMaze14 => &WALLS_BLACK_MAZE_14,
            Room::BlackMazeDot => &WALLS_BLACK_MAZE_DOT,
            Room::BlackMazeEntrance => &WALLS_BLACK_MAZE_ENTRANCE,
            Room::RedMaze17 => &WALLS_RED_MAZE_17,
            Room::RedMazeNorth => &WALLS_RED_MAZE_NORTH,
            Room::RedMazeSouth => &WALLS_RED_MAZE_SOUTH,
            Room::WhiteCastleInterior => &WALLS_WHITE_CASTLE_INTERIOR,
            Room::BlackCastleInterior => &WALLS_BLACK_CASTLE_INTERIOR,
        };

        // Dark rooms share a palette; outdoor rooms and colored mazes retain their own hues.
        let color = if room.is_dark() {
            0x08
        } else if !self.controls.color {
            match room {
                Room::WhiteCastle => 0x0c,
                Room::BlackCastle => 0x02,
                _ => 0x0a,
            }
        } else {
            match room {
                Room::GameSelection | Room::PurpleChamber | Room::RobinettCredits => 0x66,
                Room::WestHall => 0xd8,
                Room::BelowGoldCastle => 0xc8,
                Room::SecretPassage => 0xe8,
                Room::BlueMazeNorth
                | Room::BlueMaze05
                | Room::BlueMazeSouth
                | Room::BlueMazeCenter
                | Room::BlueMazeEntrance => 0x86,
                Room::SidePassage0c => 0x98,
                Room::SidePassage0d => 0xb8,
                Room::AboveWhiteCastle => 0xa8,
                Room::WhiteCastle => 0x0c,
                Room::BlackCastle => 0x00,
                Room::GoldCastle | Room::GoldTreasury => 0x1a,
                _ => 0x36,
            }
        };
        (
            walls,
            Appearance {
                foreground: self.resolve_display_color(ColorChoice::Fixed(color)),
                background: self.resolve_display_color(ColorChoice::Fixed(8)),
                mirrored: !matches!(room, Room::BlackMaze14 | Room::BlackMazeDot),
                walls_in_front: room.is_dark(),
                player_width: 4,
                barriers: [
                    matches!(room, Room::WestHall | Room::SidePassage0d),
                    matches!(room, Room::SecretPassage | Room::SidePassage0c),
                ],
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
            ObjectId::Empty => &EMPTY_IMAGE,
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

// Wall artwork and room identities.

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
