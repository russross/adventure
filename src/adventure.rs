//! Game state, startup, and explicit picture scheduling.

use crate::input::{Controls, Direction};
use crate::runtime::{FrameHost, Runtime};
use crate::sound::{Sound, SoundEffect};
use crate::video::{
    Appearance, Contacts, Geometry, HEIGHT, Object, ObjectLine, Picture, PlayfieldRow, SpriteWidth,
    Video,
};
mod assets;
mod original;
mod rules;

use adventure_trace::Point;
use assets::WALLS_ENCLOSED_ROOM;
use original::{ROOM_IDS, object_screen_x, sprite_rows, wall_rows};
use std::num::NonZeroU8;

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

    // Vertical edges use signed wrapping distance; large byte values can lie below an edge.
    fn below_edge(self, edge: u8) -> bool {
        (self.y.wrapping_sub(edge) as i8) < 0
    }

    fn within_bat_reach(self, target: Self) -> bool {
        let near = |from: u8, to: u8| to.wrapping_sub(from).wrapping_add(4) < 8;
        self.room == target.room && near(self.x, target.x) && near(self.y, target.y)
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
    Biting(BiteCountdown),
}

/// Remaining dragon updates before a missed bite returns to wandering.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BiteCountdown(NonZeroU8);

impl BiteCountdown {
    pub fn remaining(self) -> u8 {
        self.0.get()
    }

    fn new(ticks: u8) -> Self {
        Self(NonZeroU8::new(ticks).expect("a bite starts with a positive duration"))
    }

    fn advance(self) -> DragonState {
        match NonZeroU8::new(self.remaining() - 1) {
            Some(ticks) => DragonState::Biting(Self(ticks)),
            None => DragonState::Wandering,
        }
    }

    fn can_capture(self) -> bool {
        self.remaining() <= 4
    }
}

impl DragonState {
    #[cfg(test)]
    fn from_value(value: u8) -> Self {
        match value {
            0 => Self::Wandering,
            1 => Self::Dead,
            2 => Self::Swallowed,
            value => Self::Biting(BiteCountdown::new(value.wrapping_neg())),
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
    pub carried: Option<BatCargo>,
    pub flight_remaining: Option<NonZeroU8>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GateState {
    Open,
    Closed,
    #[default]
    Startup,
    Closing(ClosingProgress),
    Opening(OpeningProgress),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClosingProgress(u8);

impl ClosingProgress {
    pub fn remaining(self) -> u8 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OpeningProgress(u8);

impl OpeningProgress {
    pub fn elapsed(self) -> u8 {
        self.0
    }
}

impl GateState {
    pub fn original_value(self) -> u8 {
        match self {
            Self::Open => 1,
            Self::Closed => 28,
            Self::Startup => 0,
            Self::Closing(progress) => 28 - progress.remaining(),
            Self::Opening(progress) => 28 + progress.elapsed(),
        }
    }
    #[cfg(test)]
    fn from_value(value: u8) -> Self {
        match value {
            1 => Self::Open,
            28 => Self::Closed,
            0 => Self::Startup,
            2..=27 => Self::Closing(ClosingProgress(28 - value)),
            value => Self::Opening(OpeningProgress(value - 28)),
        }
    }

    fn on_key_contact(self) -> Self {
        match self {
            Self::Startup => Self::Open,
            Self::Open => Self::Closing(ClosingProgress(26)),
            Self::Closed => Self::Opening(OpeningProgress(1)),
            Self::Closing(progress) if progress.remaining() == 1 => Self::Closed,
            Self::Closing(progress) => Self::Closing(ClosingProgress(progress.remaining() - 1)),
            Self::Opening(progress) if progress.elapsed() == 227 => Self::Startup,
            Self::Opening(progress) => Self::Opening(OpeningProgress(progress.elapsed() + 1)),
        }
    }

    // Entry can stop animation before its next tick; repeated key contact can skip opening completion.
    fn advance_animation(self) -> Self {
        match self {
            Self::Open | Self::Closed => self,
            Self::Opening(progress) if progress.elapsed() == 27 => Self::Open,
            _ => self.on_key_contact(),
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
    visible: [Option<ObjectId>; 2],
    last_candidate: ObjectId,
    appearance: Appearance,
    objects: [Object; 2],
    walls: [PlayfieldRow; HEIGHT],
}

impl Default for PreparedDisplay {
    fn default() -> Self {
        Self {
            visible: [Some(ObjectId::Light); 2],
            last_candidate: ObjectId::Light,
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
    pub session: Runtime,
    display: PreparedDisplay,
    contacts: Contacts,
    geometry: Geometry,
    drawn: Option<(Picture, Contacts)>,
    scheduler: Scheduler,
}

// These values advance on picture boundaries and survive ordinary world initialization.
#[derive(Default)]
struct Scheduler {
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
            return self.session.finish();
        }
        while self.advance_picture(host)? {
            self.session.check()?;
        }
        self.session.finish()
    }

    // Clear state once, place the fixed barriers, and initialize the selected world.
    fn power_on(&mut self, host: &mut impl FrameHost) -> Result<bool, String> {
        self.world = World::default();
        self.scheduler.phase = PicturePhase::Player;
        self.video = Video::default();
        self.contacts = Contacts::default();
        self.geometry = Geometry::default();
        self.drawn = None;
        self.display = PreparedDisplay::default();
        self.scheduler.player_visible = false;
        self.sound = Sound::default();
        if self.session.stopped_at_boot() {
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
            if self.scheduler.phase == PicturePhase::Player {
                return Ok(true);
            }
        }
    }

    // Each phase makes its update boundary explicit and presents exactly once.
    pub fn advance_picture(&mut self, host: &mut impl FrameHost) -> Result<bool, String> {
        match self.scheduler.phase {
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
                self.scheduler.phase = PicturePhase::BatAndGates;
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
                self.scheduler.phase = PicturePhase::RemainingActors;
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
                self.scheduler.phase = PicturePhase::Player;
            }
        }
        Ok(true)
    }

    // Present the accumulated frame and sample controls at the vertical boundary.
    fn present_picture(&mut self, host: &mut impl FrameHost) -> Result<bool, String> {
        self.session.catch_up()?;
        match host.present(&self.video.frame, &self.sound)? {
            Some(controls) => {
                self.controls = controls;
                self.session.next_frame();
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

#[cfg(test)]
#[path = "tests/adventure.rs"]
mod tests;
