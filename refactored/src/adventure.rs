//! World simulation, picture scheduling, rendering, and immutable game data.

use crate::hardware::FrameHost;
use crate::input::{Controls, Difficulty, Direction};
use crate::sound::Sound;
use crate::video::{Appearance, ObjectLine, PlayfieldRow, Scanline, SpriteWidth, Video, WIDTH};

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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Game {
    #[default]
    One,
    Two,
    Three,
}

impl Game {
    fn index(self) -> usize {
        match self {
            Self::One => 0,
            Self::Two => 1,
            Self::Three => 2,
        }
    }
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
    fn value(self) -> u8 {
        match self {
            Self::Wandering => 0,
            Self::Dead => 1,
            Self::Swallowed => 2,
            Self::Biting(value) => value,
        }
    }
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
    fn index(self) -> usize {
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

#[derive(Clone, Copy, Debug)]
enum Creature {
    Dragon(DragonId),
    Bat,
}

impl Creature {
    fn record(self) -> RecordId {
        match self {
            Self::Dragon(dragon) => dragon.record(),
            Self::Bat => RecordId::Bat,
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
    start: usize,
}

impl Bitmap {
    const fn new(rows: &'static [u8]) -> Self {
        Self { rows, start: 0 }
    }
    fn row(self, index: usize) -> u8 {
        self.rows.get(self.start + index).copied().unwrap_or(0)
    }
}

#[derive(Clone, Copy, Debug)]
struct ImageChoice {
    ceiling: u8,
    bitmap: Bitmap,
}

#[derive(Clone, Copy, Debug)]
enum ColorChoice {
    Fixed(u8),
    Cycling,
}

#[derive(Clone, Copy, Debug)]
enum PositionSource {
    Record(RecordId),
    Fixed(Position),
}

#[derive(Clone, Copy, Debug)]
enum StateSource {
    Fixed(u8),
    Dragon(DragonId),
    Gate(GateId),
    GameNumber,
    BatAnimation,
}

#[derive(Clone, Copy, Debug)]
struct ObjectDescriptor {
    position: PositionSource,
    state: StateSource,
    images: &'static [ImageChoice],
    color: ColorChoice,
    monochrome: ColorChoice,
    style: SpriteWidth,
}

#[derive(Clone, Copy, Debug)]
enum Exit {
    Room(Room),
    ByGame(usize),
}

#[derive(Clone, Copy, Debug)]
struct RoomDefinition {
    walls: &'static [PlayfieldRow; 7],
    color: ColorChoice,
    monochrome: ColorChoice,
    mirrored: bool,
    foreground_priority: bool,
    player_width: u8,
    left_barrier: bool,
    right_barrier: bool,
    exits: [Exit; 4],
}

#[derive(Clone, Copy, Debug)]
struct Reaction {
    from: RecordId,
    to: RecordId,
}

#[derive(Clone, Copy, Debug)]
struct ChosenReaction {
    reaction: Reaction,
    direction: Direction,
}

#[derive(Clone, Copy)]
struct StartingWorld {
    dot: Position,
    rhindle: Dragon,
    yorgle: Dragon,
    grundle: Dragon,
    magnet: Position,
    sword: Position,
    chalice: Position,
    bridge: Position,
    gold_key: Position,
    white_key: Position,
    black_key: Position,
    gates: Gates,
    bat: Bat,
}

#[derive(Clone, Copy)]
struct RoomBounds {
    record: RecordId,
    first: Room,
    last: Room,
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

impl ObjectDescriptor {
    fn position_matches(self, record: RecordId) -> bool {
        matches!(self.position, PositionSource::Record(candidate) if candidate == record)
    }
}

// Startup and picture scheduling.

#[derive(Default)]
pub struct Adventure {
    pub world: World,
    pub video: Video,
    pub sound: Sound,
    pub controls: Controls,
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
            return Ok(());
        }
        while self.advance_picture(host)? {}
        Ok(())
    }

    // Clear state once, place the fixed barriers, and initialize the selected world.
    fn power_on(&mut self, host: &mut impl FrameHost) -> Result<bool, String> {
        self.world = World::default();
        self.phase = PicturePhase::Player;
        self.video = Video::default();
        self.display = PreparedDisplay::default();
        self.sound = Sound::default();
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

                // Player movement precedes presentation; pickup uses the newly drawn contacts.
                self.move_player(MovementPhase::Both);
                self.move_held_object();
                if !self.present_picture(host)? {
                    return Ok(false);
                }
                self.prepare_room_display();
                self.draw_room();
                self.handle_pickup_and_drop();
                self.phase = PicturePhase::BatAndGates;
            }
            PicturePhase::BatAndGates => {
                self.move_player(MovementPhase::Vertical);
                self.update_dark_room_light();
                if !self.present_picture(host)? {
                    return Ok(false);
                }
                self.update_bat();
                self.update_castle_gates();
                self.draw_room();
                self.update_grundle();
                self.update_yorgle();
                self.phase = PicturePhase::RemainingActors;
            }
            PicturePhase::RemainingActors => {
                if !self.present_picture(host)? {
                    return Ok(false);
                }
                self.move_player(MovementPhase::Horizontal);
                self.update_rhindle();
                self.update_magnet();
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
                Ok(true)
            }
            None => Ok(false),
        }
    }

    // Button edges preserve RESET's partial revival and SELECT's world reset.
    fn poll_console_switches(&mut self, host: &mut impl FrameHost) -> Result<bool, String> {
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
        if self.controls.select && !self.world.previous_select {
            if self.world.player.room == Room::GameSelection {
                self.world.game = self.world.game.next();
            }
            return self.initialize_world(host);
        }
        self.remember_console_switches();
        Ok(true)
    }

    // Only the copied world fields are reset; display caches and counters remain live.
    pub fn initialize_world(&mut self, host: &mut impl FrameHost) -> Result<bool, String> {
        self.world.player.room = Room::GameSelection;
        self.world.safe_player.room = Room::GameSelection;
        self.world.player.y = 0;
        self.world.safe_player.y = 0;
        let world = STARTING_WORLDS[self.world.game.index()];
        self.world.dot = world.dot;
        self.world.rhindle = world.rhindle;
        self.world.yorgle = world.yorgle;
        self.world.grundle = world.grundle;
        self.world.magnet = world.magnet;
        self.world.sword = world.sword;
        self.world.chalice = world.chalice;
        self.world.bridge = world.bridge;
        self.world.gold_key = world.gold_key;
        self.world.white_key = world.white_key;
        self.world.black_key = world.black_key;
        self.world.gates = world.gates;
        self.world.bat = world.bat;

        if self.world.game == Game::Three {
            self.randomize_game_three();
            if !self.present_picture(host)? {
                return Ok(false);
            }
            self.draw_room();
        }
        self.world.won = false;
        self.world.held = ObjectId::Empty;
        self.remember_console_switches();
        Ok(true)
    }

    // The shared console-poll tail needs no stack or shared pointer state.
    fn remember_console_switches(&mut self) {
        self.world.previous_reset = self.controls.reset;
        self.world.previous_select = self.controls.select;
    }

    // Work backward through the eleven constraints, retaining the counter as seed.
    fn randomize_game_three(&mut self) {
        for bounds in RANDOM_ROOM_BOUNDS.iter().rev() {
            loop {
                let old = self.world.idle_ticks;
                self.world.idle_ticks = (old >> 5).wrapping_add(old).wrapping_add(1);
                let candidate = self.world.idle_ticks & 31;
                if candidate >= bounds.first as u8 && candidate <= bounds.last as u8 {
                    self.world.position_mut(bounds.record).room = ROOM_IDS[usize::from(candidate)];
                    break;
                }
            }
        }
    }

    // The counter also supplies dynamic colors and Game 3's placement seed.
    fn track_inactivity(&mut self) {
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
    fn move_object_through_world(&mut self, record: RecordId, direction: Direction, steps: u8) {
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

    // Tagged exits select the current game's member of a three-room group.
    fn resolve_game_exit(&self, exit: Exit) -> Room {
        match exit {
            Exit::Room(room) => room,
            Exit::ByGame(group) => GAME_DEPENDENT_EXITS[group][self.world.game.index()],
        }
    }
}

// Pickup, gates, magnet, and light.

impl Adventure {
    // Fire drops an item; the last rendered contacts determine pickup candidates.
    fn handle_pickup_and_drop(&mut self) {
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
    fn move_held_object(&mut self) {
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
    fn is_touching_player(&self, object: ObjectId) -> bool {
        if object == self.world.first_visible {
            self.video.contacts.player_objects[0]
        } else if object == self.world.second_visible {
            self.video.contacts.player_objects[1]
        } else {
            false
        }
    }

    // Return the other member of a displayed object pair that touched.
    fn find_touching_object(&self, object: ObjectId) -> ObjectId {
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
    fn update_magnet(&mut self) {
        self.world.magnet.y = self.world.magnet.y.wrapping_sub(8);
        if let Some(chosen) = self.choose_reaction(&MAGNET_REACTIONS, None) {
            self.move_object_through_world(chosen.reaction.from, chosen.direction, 1);
        }
        self.world.magnet.y = self.world.magnet.y.wrapping_add(8);
    }

    // Dark rooms center the light on the player and clamp its horizontal reach.
    fn update_dark_room_light(&mut self) {
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

// Dragon and bat behavior.

impl Adventure {
    // No eligible reaction retains motion; an eligible but aligned pair stops motion.
    fn move_creature(
        &mut self,
        creature: Creature,
        list: &[Reaction],
        ignored: Option<RecordId>,
        speed: u8,
    ) -> Option<ChosenReaction> {
        let chosen = self.choose_reaction(list, ignored);
        let previous = match creature {
            Creature::Dragon(dragon) => self.world.dragon(dragon).direction,
            Creature::Bat => self.world.bat.direction,
        };
        let direction = chosen.map_or(previous, |chosen| chosen.direction);
        match creature {
            Creature::Dragon(dragon) => self.world.dragon_mut(dragon).direction = direction,
            Creature::Bat => self.world.bat.direction = direction,
        }
        self.move_object_through_world(creature.record(), direction, speed);
        chosen
    }

    // Ordered pairs use separate local identities and return the chosen pair to the bat.
    fn choose_reaction(
        &self,
        list: &[Reaction],
        ignored: Option<RecordId>,
    ) -> Option<ChosenReaction> {
        for reaction in list {
            if self.world.position(reaction.from).room != self.world.position(reaction.to).room {
                continue;
            }
            if ignored == Some(reaction.from) || ignored == Some(reaction.to) {
                continue;
            }
            return Some(ChosenReaction {
                reaction: *reaction,
                direction: self.direction_between_objects(reaction.from, reaction.to),
            });
        }
        None
    }

    // Each reaction points from its first record toward its second record.
    fn direction_between_objects(&self, from: RecordId, to: RecordId) -> Direction {
        let from = self.world.position(from);
        let to = self.world.position(to);
        if from.room != to.room {
            return Direction::default();
        }
        Direction {
            right: to.x > from.x,
            left: to.x < from.x,
            up: to.y > from.y,
            down: to.y < from.y,
        }
    }

    // Rhindle moves three steps using his own priorities.
    fn update_rhindle(&mut self) {
        self.update_dragon(DragonId::Rhindle, &RHINDLE_REACTIONS, 3);
    }

    // Yorgle moves two steps and includes the Gold Key among his fears.
    fn update_yorgle(&mut self) {
        self.update_dragon(DragonId::Yorgle, &YORGLE_REACTIONS, 2);
    }

    // Grundle moves two steps using his guarding priorities.
    fn update_grundle(&mut self) {
        self.update_dragon(DragonId::Grundle, &GRUNDLE_REACTIONS, 2);
    }

    // Preserve wandering, sword contact, swallowing, and delayed-bite branch order.
    fn update_dragon(&mut self, dragon: DragonId, reactions: &[Reaction], speed: u8) {
        let object = dragon.object();
        match self.world.dragon(dragon).state {
            DragonState::Wandering => {
                let ignored = if self.controls.right_difficulty == Difficulty::B {
                    Some(RecordId::Sword)
                } else {
                    None
                };
                self.move_creature(Creature::Dragon(dragon), reactions, ignored, speed);
                if self.is_touching_player(object) {
                    let difficulty = usize::from(self.controls.left_difficulty == Difficulty::A);
                    self.world.dragon_mut(dragon).state = DragonState::Biting(
                        DRAGON_BITE_DELAYS[self.world.game.index()][difficulty],
                    );
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
            let chosen =
                self.move_creature(Creature::Bat, &BAT_REACTIONS, self.world.bat.carried, 3);
            if let Some(chosen) = chosen {
                let target = self.world.position(chosen.reaction.to);
                let bat = self.world.bat.position;
                if target.room == bat.room
                    && target.x.wrapping_sub(bat.x).wrapping_add(4) & 0xf8 == 0
                    && target.y.wrapping_sub(bat.y).wrapping_add(4) & 0xf8 == 0
                {
                    self.world.bat.carried = Some(chosen.reaction.to);
                    self.world.bat.flight_timer = 16;
                }
            }
        }
        if let Some(cargo) = self.world.bat.carried {
            let bat = self.world.bat.position;
            *self.world.position_mut(cargo) = Position::new(bat.room, bat.x.wrapping_add(8), bat.y);
            if OBJECT_CATALOG[self.world.held as usize].position_matches(cargo) {
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
        let room = self.find_room_record(self.world.player.room);
        self.display.room_walls = room.walls;
        self.display.appearance = Appearance {
            foreground: self.resolve_display_color(if self.controls.color {
                room.color
            } else {
                room.monochrome
            }),
            background: self.resolve_display_color(ColorChoice::Fixed(8)),
            mirrored: room.mirrored,
            walls_in_front: room.foreground_priority,
            player_width: room.player_width,
            barriers: [room.left_barrier, room.right_barrier],
        };
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

        // Both slots use the same catalog resolution; priority follows their selected order.
        for (slot, object) in [self.world.first_visible, self.world.second_visible]
            .into_iter()
            .enumerate()
        {
            let descriptor = OBJECT_CATALOG[object as usize];
            let position = self.find_object_position(object);
            let image =
                self.select_object_image(descriptor.images, self.image_state(descriptor.state));
            self.display.objects[slot] = ObjectLine {
                x: object_screen_x(position.x, descriptor.style),
                color: self.resolve_display_color(if self.controls.color {
                    descriptor.color
                } else {
                    descriptor.monochrome
                }),
                width: descriptor.style,
                bits: 0,
            };
            if slot == 0 {
                self.display.first_position = position;
                self.display.first_image = image;
            } else {
                self.display.second_position = position;
                self.display.second_image = image;
            }
        }
    }

    // State pointers become a closed selection of named game fields and fixed values.
    fn image_state(&self, source: StateSource) -> u8 {
        match source {
            StateSource::Fixed(value) => value,
            StateSource::Dragon(dragon) => self.world.dragon(dragon).state.value(),
            StateSource::Gate(gate) => self.world.gates.get(gate).value(),
            StateSource::GameNumber => (self.world.game.index() * 2) as u8,
            StateSource::BatAnimation => self.world.bat.animation,
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

    // Room identities index the immutable map catalog.
    fn find_room_record(&self, room: Room) -> &'static RoomDefinition {
        &ROOMS[room as usize]
    }

    // The first inclusive ceiling selects the object's current bitmap.
    fn select_object_image(&self, choices: &'static [ImageChoice], state: u8) -> Bitmap {
        choices
            .iter()
            .find(|choice| state <= choice.ceiling)
            .expect("image ranges end at 255")
            .bitmap
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

    // Position sources distinguish live records from immutable scenery.
    fn find_object_position(&self, object: ObjectId) -> Position {
        match OBJECT_CATALOG[object as usize].position {
            PositionSource::Record(record) => self.world.position(record),
            PositionSource::Fixed(position) => position,
        }
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

// Starting worlds and randomization constraints.

static RANDOM_ROOM_BOUNDS: [RoomBounds; 11] = [
    RoomBounds {
        record: RecordId::Chalice,
        first: Room::BlackMaze13,
        last: Room::WhiteCastleInterior,
    },
    RoomBounds {
        record: RecordId::Rhindle,
        first: Room::WestHall,
        last: Room::RedHall,
    },
    RoomBounds {
        record: RecordId::Yorgle,
        first: Room::WestHall,
        last: Room::RedHall,
    },
    RoomBounds {
        record: RecordId::Grundle,
        first: Room::WestHall,
        last: Room::RedHall,
    },
    RoomBounds {
        record: RecordId::Sword,
        first: Room::WestHall,
        last: Room::RedHall,
    },
    RoomBounds {
        record: RecordId::Bridge,
        first: Room::WestHall,
        last: Room::RedHall,
    },
    RoomBounds {
        record: RecordId::GoldKey,
        first: Room::WestHall,
        last: Room::RedHall,
    },
    RoomBounds {
        record: RecordId::WhiteKey,
        first: Room::WestHall,
        last: Room::BlackMazeEntrance,
    },
    RoomBounds {
        record: RecordId::BlackKey,
        first: Room::WestHall,
        last: Room::GoldTreasury,
    },
    RoomBounds {
        record: RecordId::Bat,
        first: Room::WestHall,
        last: Room::RedHall,
    },
    RoomBounds {
        record: RecordId::Magnet,
        first: Room::WestHall,
        last: Room::RedHall,
    },
];

const STARTING_WORLDS: [StartingWorld; 3] = [GAME_ONE_WORLD, GAME_TWO_WORLD, GAME_TWO_WORLD];

const GAME_ONE_WORLD: StartingWorld = StartingWorld {
    dot: Position::new(Room::BlackMazeDot, 81, 18),
    rhindle: Dragon {
        position: Position::new(Room::AboveWhiteCastle, 80, 32),
        direction: Direction {
            right: true,
            left: true,
            up: true,
            down: true,
        },
        state: DragonState::Wandering,
    },
    yorgle: Dragon {
        position: Position::new(Room::WestHall, 80, 32),
        direction: Direction {
            right: true,
            left: true,
            up: true,
            down: true,
        },
        state: DragonState::Wandering,
    },
    grundle: Dragon {
        position: Position::new(Room::RedHall, 80, 32),
        direction: Direction {
            right: true,
            left: true,
            up: true,
            down: true,
        },
        state: DragonState::Wandering,
    },
    magnet: Position::new(Room::BlackCastleInterior, 128, 32),
    sword: Position::new(Room::GoldTreasury, 32, 32),
    chalice: Position::new(Room::PurpleChamber, 48, 32),
    bridge: Position::new(Room::BlueMazeNorth, 41, 55),
    gold_key: Position::new(Room::GoldCastle, 32, 64),
    white_key: Position::new(Room::AboveWhiteCastle, 32, 64),
    black_key: Position::new(Room::RedHall, 32, 64),
    gates: Gates {
        gold: GateState::Closed,
        white: GateState::Closed,
        black: GateState::Closed,
    },
    bat: Bat {
        position: Position::new(Room::WhiteCastleInterior, 32, 32),
        direction: Direction {
            right: true,
            left: true,
            up: true,
            down: true,
        },
        animation: 0,
        carried: None,
        flight_timer: 0,
    },
};

const GAME_TWO_WORLD: StartingWorld = StartingWorld {
    dot: Position::new(Room::BlackMazeDot, 81, 18),
    rhindle: Dragon {
        position: Position::new(Room::BlackMaze14, 80, 32),
        direction: Direction {
            right: false,
            left: true,
            up: true,
            down: false,
        },
        state: DragonState::Wandering,
    },
    yorgle: Dragon {
        position: Position::new(Room::RedMazeSouth, 80, 32),
        direction: Direction {
            right: false,
            left: true,
            up: true,
            down: false,
        },
        state: DragonState::Wandering,
    },
    grundle: Dragon {
        position: Position::new(Room::BlueMazeNorth, 80, 32),
        direction: Direction {
            right: false,
            left: true,
            up: true,
            down: false,
        },
        state: DragonState::Wandering,
    },
    magnet: Position::new(Room::AboveWhiteCastle, 128, 32),
    sword: Position::new(Room::GoldCastle, 32, 32),
    chalice: Position::new(Room::BlackMaze14, 48, 32),
    bridge: Position::new(Room::CatacombsSide, 64, 64),
    gold_key: Position::new(Room::CatacombsCenter, 32, 64),
    white_key: Position::new(Room::BlueMazeSouth, 32, 64),
    black_key: Position::new(Room::RedMazeSouth, 32, 64),
    gates: Gates {
        gold: GateState::Closed,
        white: GateState::Closed,
        black: GateState::Closed,
    },
    bat: Bat {
        position: Position::new(Room::BelowGoldCastle, 32, 32),
        direction: Direction {
            right: false,
            left: true,
            up: false,
            down: true,
        },
        animation: 0,
        carried: None,
        flight_timer: 0,
    },
};

// Ordered reactions and castle associations.

static RHINDLE_REACTIONS: [Reaction; 4] = [
    Reaction {
        from: RecordId::Sword,
        to: RecordId::Rhindle,
    },
    Reaction {
        from: RecordId::Rhindle,
        to: RecordId::Player,
    },
    Reaction {
        from: RecordId::Rhindle,
        to: RecordId::Chalice,
    },
    Reaction {
        from: RecordId::Rhindle,
        to: RecordId::WhiteKey,
    },
];

static YORGLE_REACTIONS: [Reaction; 4] = [
    Reaction {
        from: RecordId::Sword,
        to: RecordId::Yorgle,
    },
    Reaction {
        from: RecordId::GoldKey,
        to: RecordId::Yorgle,
    },
    Reaction {
        from: RecordId::Yorgle,
        to: RecordId::Player,
    },
    Reaction {
        from: RecordId::Yorgle,
        to: RecordId::Chalice,
    },
];

static GRUNDLE_REACTIONS: [Reaction; 6] = [
    Reaction {
        from: RecordId::Sword,
        to: RecordId::Grundle,
    },
    Reaction {
        from: RecordId::Grundle,
        to: RecordId::Player,
    },
    Reaction {
        from: RecordId::Grundle,
        to: RecordId::Chalice,
    },
    Reaction {
        from: RecordId::Grundle,
        to: RecordId::Bridge,
    },
    Reaction {
        from: RecordId::Grundle,
        to: RecordId::Magnet,
    },
    Reaction {
        from: RecordId::Grundle,
        to: RecordId::BlackKey,
    },
];

const DRAGON_BITE_DELAYS: [[u8; 2]; 3] = [[0xd0, 0xe8], [0xf0, 0xf6], [0xf0, 0xf6]];

static BAT_REACTIONS: [Reaction; 10] = [
    Reaction {
        from: RecordId::Bat,
        to: RecordId::Chalice,
    },
    Reaction {
        from: RecordId::Bat,
        to: RecordId::Sword,
    },
    Reaction {
        from: RecordId::Bat,
        to: RecordId::Bridge,
    },
    Reaction {
        from: RecordId::Bat,
        to: RecordId::GoldKey,
    },
    Reaction {
        from: RecordId::Bat,
        to: RecordId::WhiteKey,
    },
    Reaction {
        from: RecordId::Bat,
        to: RecordId::BlackKey,
    },
    Reaction {
        from: RecordId::Bat,
        to: RecordId::Rhindle,
    },
    Reaction {
        from: RecordId::Bat,
        to: RecordId::Yorgle,
    },
    Reaction {
        from: RecordId::Bat,
        to: RecordId::Grundle,
    },
    Reaction {
        from: RecordId::Bat,
        to: RecordId::Magnet,
    },
];

const GATE_OBJECTS: [ObjectId; 3] = [ObjectId::GoldGate, ObjectId::WhiteGate, ObjectId::BlackGate];

const GATE_KEYS: [ObjectId; 3] = [ObjectId::GoldKey, ObjectId::WhiteKey, ObjectId::BlackKey];

const CASTLE_INTERIORS: [Room; 3] = [
    Room::GoldTreasury,
    Room::WhiteCastleInterior,
    Room::BlackCastleInterior,
];

const CASTLE_OUTSIDES: [Room; 3] = [Room::GoldCastle, Room::WhiteCastle, Room::BlackCastle];

static MAGNET_REACTIONS: [Reaction; 6] = [
    Reaction {
        from: RecordId::GoldKey,
        to: RecordId::Magnet,
    },
    Reaction {
        from: RecordId::WhiteKey,
        to: RecordId::Magnet,
    },
    Reaction {
        from: RecordId::BlackKey,
        to: RecordId::Magnet,
    },
    Reaction {
        from: RecordId::Sword,
        to: RecordId::Magnet,
    },
    Reaction {
        from: RecordId::Bridge,
        to: RecordId::Magnet,
    },
    Reaction {
        from: RecordId::Chalice,
        to: RecordId::Magnet,
    },
];

// Wall layouts and room map.

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

static ROOMS: [RoomDefinition; 31] = [
    // $00: GameSelection.
    RoomDefinition {
        walls: &WALLS_ENCLOSED_ROOM,
        color: ColorChoice::Fixed(0x66),
        monochrome: ColorChoice::Fixed(0x0a),
        mirrored: true,
        foreground_priority: false,
        player_width: 4,
        left_barrier: false,
        right_barrier: false,
        exits: [
            Exit::Room(Room::GameSelection),
            Exit::Room(Room::GameSelection),
            Exit::Room(Room::GameSelection),
            Exit::Room(Room::GameSelection),
        ],
    },
    // $01: WestHall.
    RoomDefinition {
        walls: &WALLS_OPEN_HALL,
        color: ColorChoice::Fixed(0xd8),
        monochrome: ColorChoice::Fixed(0x0a),
        mirrored: true,
        foreground_priority: false,
        player_width: 4,
        left_barrier: true,
        right_barrier: false,
        exits: [
            Exit::Room(Room::BlueMazeEntrance),
            Exit::Room(Room::BelowGoldCastle),
            Exit::ByGame(0),
            Exit::Room(Room::SecretPassage),
        ],
    },
    // $02: BelowGoldCastle.
    RoomDefinition {
        walls: &WALLS_OPEN_HALL,
        color: ColorChoice::Fixed(0xc8),
        monochrome: ColorChoice::Fixed(0x0a),
        mirrored: true,
        foreground_priority: false,
        player_width: 4,
        left_barrier: false,
        right_barrier: false,
        exits: [
            Exit::Room(Room::GoldCastle),
            Exit::Room(Room::SecretPassage),
            Exit::ByGame(1),
            Exit::Room(Room::WestHall),
        ],
    },
    // $03: SecretPassage.
    RoomDefinition {
        walls: &WALLS_SECRET_PASSAGE,
        color: ColorChoice::Fixed(0xe8),
        monochrome: ColorChoice::Fixed(0x0a),
        mirrored: true,
        foreground_priority: false,
        player_width: 4,
        left_barrier: false,
        right_barrier: true,
        exits: [
            Exit::Room(Room::BlueMazeSouth),
            Exit::Room(Room::WestHall),
            Exit::ByGame(2),
            Exit::Room(Room::BelowGoldCastle),
        ],
    },
    // $04: BlueMazeNorth.
    RoomDefinition {
        walls: &WALLS_BLUE_MAZE_NORTH,
        color: ColorChoice::Fixed(0x86),
        monochrome: ColorChoice::Fixed(0x0a),
        mirrored: true,
        foreground_priority: false,
        player_width: 4,
        left_barrier: false,
        right_barrier: false,
        exits: [
            Exit::Room(Room::BlackCastle),
            Exit::Room(Room::BlueMaze05),
            Exit::Room(Room::BlueMazeCenter),
            Exit::Room(Room::BlueMazeSouth),
        ],
    },
    // $05: BlueMaze05.
    RoomDefinition {
        walls: &WALLS_BLUE_MAZE_05,
        color: ColorChoice::Fixed(0x86),
        monochrome: ColorChoice::Fixed(0x0a),
        mirrored: true,
        foreground_priority: false,
        player_width: 4,
        left_barrier: false,
        right_barrier: false,
        exits: [
            Exit::Room(Room::RedHall),
            Exit::Room(Room::BlueMazeSouth),
            Exit::Room(Room::BlueMazeEntrance),
            Exit::Room(Room::BlueMazeNorth),
        ],
    },
    // $06: BlueMazeSouth.
    RoomDefinition {
        walls: &WALLS_BLUE_MAZE_SOUTH,
        color: ColorChoice::Fixed(0x86),
        monochrome: ColorChoice::Fixed(0x0a),
        mirrored: true,
        foreground_priority: false,
        player_width: 4,
        left_barrier: false,
        right_barrier: false,
        exits: [
            Exit::Room(Room::BlueMazeCenter),
            Exit::Room(Room::BlueMazeNorth),
            Exit::Room(Room::SecretPassage),
            Exit::Room(Room::BlueMaze05),
        ],
    },
    // $07: BlueMazeCenter.
    RoomDefinition {
        walls: &WALLS_BLUE_MAZE_CENTER,
        color: ColorChoice::Fixed(0x86),
        monochrome: ColorChoice::Fixed(0x0a),
        mirrored: true,
        foreground_priority: false,
        player_width: 4,
        left_barrier: false,
        right_barrier: false,
        exits: [
            Exit::Room(Room::BlueMazeNorth),
            Exit::Room(Room::BlueMazeEntrance),
            Exit::Room(Room::BlueMazeSouth),
            Exit::Room(Room::BlueMazeEntrance),
        ],
    },
    // $08: BlueMazeEntrance.
    RoomDefinition {
        walls: &WALLS_BLUE_MAZE_ENTRANCE,
        color: ColorChoice::Fixed(0x86),
        monochrome: ColorChoice::Fixed(0x0a),
        mirrored: true,
        foreground_priority: false,
        player_width: 4,
        left_barrier: false,
        right_barrier: false,
        exits: [
            Exit::Room(Room::BlueMaze05),
            Exit::Room(Room::BlueMazeCenter),
            Exit::Room(Room::WestHall),
            Exit::Room(Room::BlueMazeCenter),
        ],
    },
    // $09: CatacombsCenter.
    RoomDefinition {
        walls: &WALLS_CATACOMBS_CENTER,
        color: ColorChoice::Fixed(0x08),
        monochrome: ColorChoice::Fixed(0x08),
        mirrored: true,
        foreground_priority: true,
        player_width: 4,
        left_barrier: false,
        right_barrier: false,
        exits: [
            Exit::Room(Room::CatacombsEntrance),
            Exit::Room(Room::CatacombsEntrance),
            Exit::Room(Room::CatacombsSide),
            Exit::Room(Room::CatacombsEntrance),
        ],
    },
    // $0A: CatacombsEntrance.
    RoomDefinition {
        walls: &WALLS_CATACOMBS_ENTRANCE,
        color: ColorChoice::Fixed(0x08),
        monochrome: ColorChoice::Fixed(0x08),
        mirrored: true,
        foreground_priority: true,
        player_width: 4,
        left_barrier: false,
        right_barrier: false,
        exits: [
            Exit::Room(Room::SecretPassage),
            Exit::Room(Room::CatacombsCenter),
            Exit::Room(Room::CatacombsCenter),
            Exit::Room(Room::CatacombsCenter),
        ],
    },
    // $0B: CatacombsSide.
    RoomDefinition {
        walls: &WALLS_CATACOMBS_SIDE,
        color: ColorChoice::Fixed(0x08),
        monochrome: ColorChoice::Fixed(0x08),
        mirrored: true,
        foreground_priority: true,
        player_width: 4,
        left_barrier: false,
        right_barrier: false,
        exits: [
            Exit::Room(Room::CatacombsCenter),
            Exit::Room(Room::SidePassage0c),
            Exit::Room(Room::PurpleChamber),
            Exit::Room(Room::SidePassage0d),
        ],
    },
    // $0C: SidePassage0c.
    RoomDefinition {
        walls: &WALLS_SIDE_PASSAGE,
        color: ColorChoice::Fixed(0x98),
        monochrome: ColorChoice::Fixed(0x0a),
        mirrored: true,
        foreground_priority: false,
        player_width: 4,
        left_barrier: false,
        right_barrier: true,
        exits: [
            Exit::Room(Room::PurpleChamber),
            Exit::Room(Room::SidePassage0d),
            Exit::Room(Room::RedHall),
            Exit::Room(Room::CatacombsSide),
        ],
    },
    // $0D: SidePassage0d.
    RoomDefinition {
        walls: &WALLS_SIDE_PASSAGE,
        color: ColorChoice::Fixed(0xb8),
        monochrome: ColorChoice::Fixed(0x0a),
        mirrored: true,
        foreground_priority: false,
        player_width: 4,
        left_barrier: true,
        right_barrier: false,
        exits: [
            Exit::Room(Room::WhiteCastle),
            Exit::Room(Room::CatacombsSide),
            Exit::Room(Room::AboveWhiteCastle),
            Exit::Room(Room::SidePassage0c),
        ],
    },
    // $0E: AboveWhiteCastle.
    RoomDefinition {
        walls: &WALLS_TOP_OPENING,
        color: ColorChoice::Fixed(0xa8),
        monochrome: ColorChoice::Fixed(0x0a),
        mirrored: true,
        foreground_priority: false,
        player_width: 4,
        left_barrier: false,
        right_barrier: false,
        exits: [
            Exit::Room(Room::SidePassage0d),
            Exit::Room(Room::BlackCastle),
            Exit::Room(Room::WhiteCastle),
            Exit::Room(Room::BlackCastle),
        ],
    },
    // $0F: WhiteCastle.
    RoomDefinition {
        walls: &WALLS_CASTLE,
        color: ColorChoice::Fixed(0x0c),
        monochrome: ColorChoice::Fixed(0x0c),
        mirrored: true,
        foreground_priority: false,
        player_width: 4,
        left_barrier: false,
        right_barrier: false,
        exits: [
            Exit::Room(Room::AboveWhiteCastle),
            Exit::Room(Room::WhiteCastle),
            Exit::Room(Room::SidePassage0d),
            Exit::Room(Room::WhiteCastle),
        ],
    },
    // $10: BlackCastle.
    RoomDefinition {
        walls: &WALLS_CASTLE,
        color: ColorChoice::Fixed(0x00),
        monochrome: ColorChoice::Fixed(0x02),
        mirrored: true,
        foreground_priority: false,
        player_width: 4,
        left_barrier: false,
        right_barrier: false,
        exits: [
            Exit::Room(Room::WestHall),
            Exit::Room(Room::PurpleChamber),
            Exit::Room(Room::BlueMazeNorth),
            Exit::Room(Room::PurpleChamber),
        ],
    },
    // $11: GoldCastle.
    RoomDefinition {
        walls: &WALLS_CASTLE,
        color: ColorChoice::Fixed(0x1a),
        monochrome: ColorChoice::Fixed(0x0a),
        mirrored: true,
        foreground_priority: false,
        player_width: 4,
        left_barrier: false,
        right_barrier: false,
        exits: [
            Exit::Room(Room::BlueMazeSouth),
            Exit::Room(Room::SecretPassage),
            Exit::Room(Room::BelowGoldCastle),
            Exit::Room(Room::WestHall),
        ],
    },
    // $12: GoldTreasury.
    RoomDefinition {
        walls: &WALLS_ENCLOSED_ROOM,
        color: ColorChoice::Fixed(0x1a),
        monochrome: ColorChoice::Fixed(0x0a),
        mirrored: true,
        foreground_priority: false,
        player_width: 4,
        left_barrier: false,
        right_barrier: false,
        exits: [
            Exit::Room(Room::GoldTreasury),
            Exit::Room(Room::GoldTreasury),
            Exit::Room(Room::GoldTreasury),
            Exit::Room(Room::GoldTreasury),
        ],
    },
    // $13: BlackMaze13.
    RoomDefinition {
        walls: &WALLS_BLACK_MAZE_13,
        color: ColorChoice::Fixed(0x08),
        monochrome: ColorChoice::Fixed(0x08),
        mirrored: true,
        foreground_priority: true,
        player_width: 4,
        left_barrier: false,
        right_barrier: false,
        exits: [
            Exit::Room(Room::BlackMazeDot),
            Exit::Room(Room::BlackMaze14),
            Exit::Room(Room::BlackMazeDot),
            Exit::Room(Room::BlackMazeEntrance),
        ],
    },
    // $14: BlackMaze14.
    RoomDefinition {
        walls: &WALLS_BLACK_MAZE_14,
        color: ColorChoice::Fixed(0x08),
        monochrome: ColorChoice::Fixed(0x08),
        mirrored: false,
        foreground_priority: true,
        player_width: 4,
        left_barrier: false,
        right_barrier: false,
        exits: [
            Exit::Room(Room::BlackMazeEntrance),
            Exit::Room(Room::BlackMazeDot),
            Exit::Room(Room::BlackMazeEntrance),
            Exit::Room(Room::BlackMaze13),
        ],
    },
    // $15: BlackMazeDot.
    RoomDefinition {
        walls: &WALLS_BLACK_MAZE_DOT,
        color: ColorChoice::Fixed(0x08),
        monochrome: ColorChoice::Fixed(0x08),
        mirrored: false,
        foreground_priority: true,
        player_width: 4,
        left_barrier: false,
        right_barrier: false,
        exits: [
            Exit::Room(Room::BlackMaze13),
            Exit::Room(Room::BlackMazeEntrance),
            Exit::Room(Room::BlackMaze13),
            Exit::Room(Room::BlackMaze14),
        ],
    },
    // $16: BlackMazeEntrance.
    RoomDefinition {
        walls: &WALLS_BLACK_MAZE_ENTRANCE,
        color: ColorChoice::Fixed(0x08),
        monochrome: ColorChoice::Fixed(0x08),
        mirrored: true,
        foreground_priority: true,
        player_width: 4,
        left_barrier: false,
        right_barrier: false,
        exits: [
            Exit::Room(Room::BlackMaze14),
            Exit::Room(Room::BlackMaze13),
            Exit::Room(Room::BlackCastleInterior),
            Exit::Room(Room::BlackMazeDot),
        ],
    },
    // $17: RedMaze17.
    RoomDefinition {
        walls: &WALLS_RED_MAZE_17,
        color: ColorChoice::Fixed(0x36),
        monochrome: ColorChoice::Fixed(0x0a),
        mirrored: true,
        foreground_priority: false,
        player_width: 4,
        left_barrier: false,
        right_barrier: false,
        exits: [
            Exit::Room(Room::RedMazeSouth),
            Exit::Room(Room::RedMazeNorth),
            Exit::Room(Room::RedMazeSouth),
            Exit::Room(Room::RedMazeNorth),
        ],
    },
    // $18: RedMazeNorth.
    RoomDefinition {
        walls: &WALLS_RED_MAZE_NORTH,
        color: ColorChoice::Fixed(0x36),
        monochrome: ColorChoice::Fixed(0x0a),
        mirrored: true,
        foreground_priority: false,
        player_width: 4,
        left_barrier: false,
        right_barrier: false,
        exits: [
            Exit::Room(Room::WhiteCastleInterior),
            Exit::Room(Room::RedMaze17),
            Exit::Room(Room::WhiteCastleInterior),
            Exit::Room(Room::RedMaze17),
        ],
    },
    // $19: RedMazeSouth.
    RoomDefinition {
        walls: &WALLS_RED_MAZE_SOUTH,
        color: ColorChoice::Fixed(0x36),
        monochrome: ColorChoice::Fixed(0x0a),
        mirrored: true,
        foreground_priority: false,
        player_width: 4,
        left_barrier: false,
        right_barrier: false,
        exits: [
            Exit::Room(Room::RedMaze17),
            Exit::Room(Room::WhiteCastleInterior),
            Exit::Room(Room::RedMaze17),
            Exit::Room(Room::WhiteCastleInterior),
        ],
    },
    // $1A: WhiteCastleInterior.
    RoomDefinition {
        walls: &WALLS_WHITE_CASTLE_INTERIOR,
        color: ColorChoice::Fixed(0x36),
        monochrome: ColorChoice::Fixed(0x0a),
        mirrored: true,
        foreground_priority: false,
        player_width: 4,
        left_barrier: false,
        right_barrier: false,
        exits: [
            Exit::Room(Room::RedMazeNorth),
            Exit::Room(Room::RedMazeSouth),
            Exit::Room(Room::RedMazeNorth),
            Exit::Room(Room::RedMazeSouth),
        ],
    },
    // $1B: BlackCastleInterior.
    RoomDefinition {
        walls: &WALLS_BLACK_CASTLE_INTERIOR,
        color: ColorChoice::Fixed(0x36),
        monochrome: ColorChoice::Fixed(0x0a),
        mirrored: true,
        foreground_priority: false,
        player_width: 4,
        left_barrier: false,
        right_barrier: false,
        exits: [
            Exit::ByGame(3),
            Exit::ByGame(3),
            Exit::ByGame(3),
            Exit::ByGame(3),
        ],
    },
    // $1C: PurpleChamber.
    RoomDefinition {
        walls: &WALLS_ENCLOSED_ROOM,
        color: ColorChoice::Fixed(0x66),
        monochrome: ColorChoice::Fixed(0x0a),
        mirrored: true,
        foreground_priority: false,
        player_width: 4,
        left_barrier: false,
        right_barrier: false,
        exits: [
            Exit::Room(Room::RedHall),
            Exit::Room(Room::BlueMazeCenter),
            Exit::ByGame(4),
            Exit::Room(Room::BlueMazeEntrance),
        ],
    },
    // $1D: RedHall.
    RoomDefinition {
        walls: &WALLS_TOP_OPENING,
        color: ColorChoice::Fixed(0x36),
        monochrome: ColorChoice::Fixed(0x0a),
        mirrored: true,
        foreground_priority: false,
        player_width: 4,
        left_barrier: false,
        right_barrier: false,
        exits: [
            Exit::ByGame(5),
            Exit::Room(Room::WestHall),
            Exit::Room(Room::BlackCastle),
            Exit::Room(Room::SecretPassage),
        ],
    },
    // $1E: RobinettCredits.
    RoomDefinition {
        walls: &WALLS_OPEN_HALL,
        color: ColorChoice::Fixed(0x66),
        monochrome: ColorChoice::Fixed(0x0a),
        mirrored: true,
        foreground_priority: false,
        player_width: 4,
        left_barrier: false,
        right_barrier: false,
        exits: [
            Exit::Room(Room::BlueMazeSouth),
            Exit::Room(Room::WestHall),
            Exit::Room(Room::BlueMazeSouth),
            Exit::Room(Room::SecretPassage),
        ],
    },
];

static GAME_DEPENDENT_EXITS: [[Room; 3]; 6] = [
    [Room::BlackCastle, Room::WhiteCastle, Room::WhiteCastle],
    [Room::BlueMaze05, Room::GoldCastle, Room::GoldCastle],
    [
        Room::RedHall,
        Room::CatacombsEntrance,
        Room::CatacombsEntrance,
    ],
    [
        Room::PurpleChamber,
        Room::BlackMazeEntrance,
        Room::BlackMazeEntrance,
    ],
    [
        Room::BlackCastleInterior,
        Room::SidePassage0c,
        Room::SidePassage0c,
    ],
    [
        Room::SecretPassage,
        Room::SidePassage0c,
        Room::SidePassage0c,
    ],
];

// Sprite artwork and image ranges.

static GATE_IMAGES: [ImageChoice; 13] = [
    ImageChoice {
        ceiling: 0x04,
        bitmap: Bitmap {
            rows: &GATE_IMAGE,
            start: 12,
        },
    },
    ImageChoice {
        ceiling: 0x08,
        bitmap: Bitmap {
            rows: &GATE_IMAGE,
            start: 10,
        },
    },
    ImageChoice {
        ceiling: 0x0c,
        bitmap: Bitmap {
            rows: &GATE_IMAGE,
            start: 8,
        },
    },
    ImageChoice {
        ceiling: 0x10,
        bitmap: Bitmap {
            rows: &GATE_IMAGE,
            start: 6,
        },
    },
    ImageChoice {
        ceiling: 0x14,
        bitmap: Bitmap {
            rows: &GATE_IMAGE,
            start: 4,
        },
    },
    ImageChoice {
        ceiling: 0x18,
        bitmap: Bitmap {
            rows: &GATE_IMAGE,
            start: 2,
        },
    },
    ImageChoice {
        ceiling: 0x1c,
        bitmap: Bitmap::new(&GATE_IMAGE),
    },
    ImageChoice {
        ceiling: 0x20,
        bitmap: Bitmap {
            rows: &GATE_IMAGE,
            start: 2,
        },
    },
    ImageChoice {
        ceiling: 0x24,
        bitmap: Bitmap {
            rows: &GATE_IMAGE,
            start: 4,
        },
    },
    ImageChoice {
        ceiling: 0x28,
        bitmap: Bitmap {
            rows: &GATE_IMAGE,
            start: 6,
        },
    },
    ImageChoice {
        ceiling: 0x2c,
        bitmap: Bitmap {
            rows: &GATE_IMAGE,
            start: 8,
        },
    },
    ImageChoice {
        ceiling: 0x30,
        bitmap: Bitmap {
            rows: &GATE_IMAGE,
            start: 10,
        },
    },
    ImageChoice {
        ceiling: 0xff,
        bitmap: Bitmap {
            rows: &GATE_IMAGE,
            start: 12,
        },
    },
];

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

const GOLD_GATE_POSITION: Position = Position::new(Room::GoldCastle, 77, 49);

const WHITE_GATE_POSITION: Position = Position::new(Room::WhiteCastle, 77, 49);

const BLACK_GATE_POSITION: Position = Position::new(Room::BlackCastle, 77, 49);

static LIGHT_IMAGES: [ImageChoice; 1] = [ImageChoice {
    ceiling: 0xff,
    bitmap: Bitmap::new(&LIGHT_IMAGE),
}];

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

static BRIDGE_IMAGES: [ImageChoice; 1] = [ImageChoice {
    ceiling: 0xff,
    bitmap: Bitmap::new(&BRIDGE_IMAGE),
}];

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

static KEY_IMAGES: [ImageChoice; 1] = [ImageChoice {
    ceiling: 0xff,
    bitmap: Bitmap::new(&KEY_IMAGE),
}];

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

static BAT_IMAGES: [ImageChoice; 2] = [
    ImageChoice {
        ceiling: 0x03,
        bitmap: Bitmap::new(&BAT_FRAME_ONE),
    },
    ImageChoice {
        ceiling: 0xff,
        bitmap: Bitmap::new(&BAT_FRAME_TWO),
    },
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

static DRAGON_IMAGES: [ImageChoice; 4] = [
    ImageChoice {
        ceiling: 0x00,
        bitmap: Bitmap::new(&DRAGON_WALKING_IMAGE),
    },
    ImageChoice {
        ceiling: 0x01,
        bitmap: Bitmap::new(&DRAGON_DEAD_IMAGE),
    },
    ImageChoice {
        ceiling: 0x02,
        bitmap: Bitmap::new(&DRAGON_WALKING_IMAGE),
    },
    ImageChoice {
        ceiling: 0xff,
        bitmap: Bitmap::new(&DRAGON_BITING_IMAGE),
    },
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

static SWORD_IMAGES: [ImageChoice; 1] = [ImageChoice {
    ceiling: 0xff,
    bitmap: Bitmap::new(&SWORD_IMAGE),
}];

static SWORD_IMAGE: [u8; 6] = [
    0x20, // |  █     |
    0x40, // | █      |
    0xff, // |████████|
    0x40, // | █      |
    0x20, // |  █     |
    0x00, // |        |
];

static DOT_IMAGES: [ImageChoice; 1] = [ImageChoice {
    ceiling: 0xff,
    bitmap: Bitmap::new(&DOT_IMAGE),
}];

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

const ROBINETT_CREDITS_POSITION: Position = Position::new(Room::RobinettCredits, 80, 105);

static ROBINETT_CREDITS_IMAGES: [ImageChoice; 1] = [ImageChoice {
    ceiling: 0xff,
    bitmap: Bitmap::new(&ROBINETT_CREDITS_IMAGE),
}];

static CHALICE_IMAGES: [ImageChoice; 1] = [ImageChoice {
    ceiling: 0xff,
    bitmap: Bitmap::new(&CHALICE_IMAGE),
}];

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

static EMPTY_IMAGES: [ImageChoice; 1] = [ImageChoice {
    ceiling: 0xff,
    bitmap: Bitmap::new(&EMPTY_IMAGE),
}];

static EMPTY_IMAGE: [u8; 1] = [
    0x00, // |        |
];

const GAME_NUMBER_POSITION: Position = Position::new(Room::GameSelection, 80, 64);

static GAME_NUMBER_IMAGES: [ImageChoice; 3] = [
    ImageChoice {
        ceiling: 0x01,
        bitmap: Bitmap::new(&GAME_ONE_IMAGE),
    },
    ImageChoice {
        ceiling: 0x03,
        bitmap: Bitmap::new(&GAME_TWO_IMAGE),
    },
    ImageChoice {
        ceiling: 0xff,
        bitmap: Bitmap::new(&GAME_THREE_IMAGE),
    },
];

static MAGNET_IMAGES: [ImageChoice; 1] = [ImageChoice {
    ceiling: 0xff,
    bitmap: Bitmap::new(&MAGNET_IMAGE),
}];

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

// Object catalog.

static OBJECT_CATALOG: [ObjectDescriptor; 19] = [
    // Light.
    ObjectDescriptor {
        position: PositionSource::Record(RecordId::Light),
        state: StateSource::Fixed(0),
        images: &LIGHT_IMAGES,
        color: ColorChoice::Fixed(0x28),
        monochrome: ColorChoice::Fixed(0x0c),
        style: SpriteWidth::Wide,
    },
    // GoldGate.
    ObjectDescriptor {
        position: PositionSource::Fixed(GOLD_GATE_POSITION),
        state: StateSource::Gate(GateId::Gold),
        images: &GATE_IMAGES,
        color: ColorChoice::Fixed(0x00),
        monochrome: ColorChoice::Fixed(0x00),
        style: SpriteWidth::Normal,
    },
    // WhiteGate.
    ObjectDescriptor {
        position: PositionSource::Fixed(WHITE_GATE_POSITION),
        state: StateSource::Gate(GateId::White),
        images: &GATE_IMAGES,
        color: ColorChoice::Fixed(0x00),
        monochrome: ColorChoice::Fixed(0x00),
        style: SpriteWidth::Normal,
    },
    // BlackGate.
    ObjectDescriptor {
        position: PositionSource::Fixed(BLACK_GATE_POSITION),
        state: StateSource::Gate(GateId::Black),
        images: &GATE_IMAGES,
        color: ColorChoice::Fixed(0x00),
        monochrome: ColorChoice::Fixed(0x00),
        style: SpriteWidth::Normal,
    },
    // Credits.
    ObjectDescriptor {
        position: PositionSource::Fixed(ROBINETT_CREDITS_POSITION),
        state: StateSource::Fixed(0),
        images: &ROBINETT_CREDITS_IMAGES,
        color: ColorChoice::Cycling,
        monochrome: ColorChoice::Fixed(0x00),
        style: SpriteWidth::Normal,
    },
    // GameNumber.
    ObjectDescriptor {
        position: PositionSource::Fixed(GAME_NUMBER_POSITION),
        state: StateSource::GameNumber,
        images: &GAME_NUMBER_IMAGES,
        color: ColorChoice::Fixed(0xc8),
        monochrome: ColorChoice::Fixed(0x00),
        style: SpriteWidth::Normal,
    },
    // Rhindle.
    ObjectDescriptor {
        position: PositionSource::Record(RecordId::Rhindle),
        state: StateSource::Dragon(DragonId::Rhindle),
        images: &DRAGON_IMAGES,
        color: ColorChoice::Fixed(0x36),
        monochrome: ColorChoice::Fixed(0x0e),
        style: SpriteWidth::Normal,
    },
    // Yorgle.
    ObjectDescriptor {
        position: PositionSource::Record(RecordId::Yorgle),
        state: StateSource::Dragon(DragonId::Yorgle),
        images: &DRAGON_IMAGES,
        color: ColorChoice::Fixed(0x1a),
        monochrome: ColorChoice::Fixed(0x06),
        style: SpriteWidth::Normal,
    },
    // Grundle.
    ObjectDescriptor {
        position: PositionSource::Record(RecordId::Grundle),
        state: StateSource::Dragon(DragonId::Grundle),
        images: &DRAGON_IMAGES,
        color: ColorChoice::Fixed(0xc8),
        monochrome: ColorChoice::Fixed(0x00),
        style: SpriteWidth::Normal,
    },
    // Sword.
    ObjectDescriptor {
        position: PositionSource::Record(RecordId::Sword),
        state: StateSource::Fixed(0),
        images: &SWORD_IMAGES,
        color: ColorChoice::Fixed(0x1a),
        monochrome: ColorChoice::Fixed(0x06),
        style: SpriteWidth::Normal,
    },
    // Bridge.
    ObjectDescriptor {
        position: PositionSource::Record(RecordId::Bridge),
        state: StateSource::Fixed(0),
        images: &BRIDGE_IMAGES,
        color: ColorChoice::Fixed(0x66),
        monochrome: ColorChoice::Fixed(0x02),
        style: SpriteWidth::Wide,
    },
    // GoldKey.
    ObjectDescriptor {
        position: PositionSource::Record(RecordId::GoldKey),
        state: StateSource::Fixed(0),
        images: &KEY_IMAGES,
        color: ColorChoice::Fixed(0x1a),
        monochrome: ColorChoice::Fixed(0x06),
        style: SpriteWidth::Normal,
    },
    // WhiteKey.
    ObjectDescriptor {
        position: PositionSource::Record(RecordId::WhiteKey),
        state: StateSource::Fixed(0),
        images: &KEY_IMAGES,
        color: ColorChoice::Fixed(0x0e),
        monochrome: ColorChoice::Fixed(0x0e),
        style: SpriteWidth::Normal,
    },
    // BlackKey.
    ObjectDescriptor {
        position: PositionSource::Record(RecordId::BlackKey),
        state: StateSource::Fixed(0),
        images: &KEY_IMAGES,
        color: ColorChoice::Fixed(0x00),
        monochrome: ColorChoice::Fixed(0x00),
        style: SpriteWidth::Normal,
    },
    // Bat.
    ObjectDescriptor {
        position: PositionSource::Record(RecordId::Bat),
        state: StateSource::BatAnimation,
        images: &BAT_IMAGES,
        color: ColorChoice::Fixed(0x00),
        monochrome: ColorChoice::Fixed(0x00),
        style: SpriteWidth::Normal,
    },
    // Dot.
    ObjectDescriptor {
        position: PositionSource::Record(RecordId::Dot),
        state: StateSource::Fixed(0),
        images: &DOT_IMAGES,
        color: ColorChoice::Fixed(0x08),
        monochrome: ColorChoice::Fixed(0x08),
        style: SpriteWidth::Normal,
    },
    // Chalice.
    ObjectDescriptor {
        position: PositionSource::Record(RecordId::Chalice),
        state: StateSource::Fixed(0),
        images: &CHALICE_IMAGES,
        color: ColorChoice::Cycling,
        monochrome: ColorChoice::Fixed(0x06),
        style: SpriteWidth::Normal,
    },
    // Magnet.
    ObjectDescriptor {
        position: PositionSource::Record(RecordId::Magnet),
        state: StateSource::Fixed(0),
        images: &MAGNET_IMAGES,
        color: ColorChoice::Fixed(0x00),
        monochrome: ColorChoice::Fixed(0x06),
        style: SpriteWidth::Normal,
    },
    // Empty.
    ObjectDescriptor {
        position: PositionSource::Record(RecordId::Bridge),
        state: StateSource::Fixed(0),
        images: &EMPTY_IMAGES,
        color: ColorChoice::Fixed(0x00),
        monochrome: ColorChoice::Fixed(0x00),
        style: SpriteWidth::Normal,
    },
];

#[cfg(test)]
#[path = "tests/adventure.rs"]
mod tests;
