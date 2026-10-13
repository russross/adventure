//! Functional translation of Adventure, with subroutines and static data in ROM order.

use crate::hardware::FrameHost;
use crate::input::{Controls, Difficulty, Direction};
use crate::sound::Sound;
use crate::tia::{Drawable, PlayfieldRow, SpriteStyle, Tia};
use adventure_trace::{Control, Point, Session};
use std::cell::RefCell;

adventure_trace::port_adapter!(ram, tia);

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

// RAM has named records. Temporary room pointers, contacts, reaction indices, and scratch
// bytes become local values or return values rather than sharing an emulated zero-page cell.
pub struct Ram {
    display: PreparedDisplay,
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

impl Default for Ram {
    fn default() -> Self {
        Self {
            display: PreparedDisplay::default(),
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

impl Ram {
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
    style: SpriteStyle,
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
    room_walls: &'static [PlayfieldRow; 7],
    first_image: Bitmap,
    second_image: Bitmap,
    first_position: Position,
    second_position: Position,
}

impl Default for PreparedDisplay {
    fn default() -> Self {
        Self {
            room_walls: &WALLS_ENCLOSED_ROOM,
            first_image: Bitmap::new(&EMPTY_IMAGE),
            second_image: Bitmap::new(&EMPTY_IMAGE),
            first_position: Position::default(),
            second_position: Position::default(),
        }
    }
}

#[derive(Default)]
pub struct Adventure {
    pub ram: Ram,
    pub tia: Tia,
    pub sound: Sound,
    pub controls: Controls,
    pub trace: Option<RefCell<Session>>,
}

impl Adventure {
    // $F000: The entry keeps startup and the repeated game-turn sequence separate.
    pub fn cartridge_entry(&mut self, host: &mut impl FrameHost) -> Result<(), String> {
        if !self.power_on(host)? {
            if let Some(trace) = &self.trace {
                trace.borrow_mut().finish()?;
            }
            return Ok(());
        }
        while self.run_game(host)? {
            if let Some(trace) = &self.trace {
                trace.borrow().check()?;
            }
        }
        if let Some(trace) = &self.trace {
            trace.borrow_mut().finish()?;
        }
        Ok(())
    }

    // $F008: Prepare each row in the original order and paint at horizontal boundaries.
    // Positioning and all row preparation are functional operations; no cycle delays remain.
    fn draw_room(&mut self) {
        self.position_drawable(Drawable::FirstObject, self.ram.display.first_position.x);
        self.position_drawable(Drawable::SecondObject, self.ram.display.second_position.x);
        self.position_drawable(Drawable::Player, self.ram.player.x);
        self.tia.clear_contacts();
        let player_draw_y = self.ram.player.y.wrapping_sub(4);

        let mut first_row = 0;
        let mut second_row = 0;
        let mut wall_band = 0;
        self.tia.second_object.set_row(0);
        self.tia.second_object.delay_rows(true);
        self.tia
            .background_pattern(self.ram.display.room_walls[wall_band]);
        wall_band += 1;
        self.tia.begin_picture();
        self.tia.blank(false);

        // Line A retains the preceding committed rows while the next rows are prepared.
        for row in (8u8..=104).rev() {
            if row != 104 && (row.wrapping_sub(self.ram.display.second_position.y) as i8) < 0 {
                let bits = self.ram.display.second_image.row(second_row);
                self.tia.second_object.set_row(bits);
                if bits != 0 {
                    second_row += 1;
                }
            }
            let mut first_bits = 0;
            if (row.wrapping_sub(self.ram.display.first_position.y) as i8) < 0 {
                first_bits = self.ram.display.first_image.row(first_row);
                if first_bits != 0 {
                    first_row += 1;
                }
            }
            let player_visible = row.wrapping_sub(player_draw_y) & 0xfc == 0;
            let refresh_walls = row & 15 == 0;
            self.tia.horizontal_sync();

            // Line B commits the player and object rows, then advances the wall band.
            self.tia.player.enable(player_visible);
            self.tia.first_object_row(first_bits);
            if refresh_walls {
                self.tia
                    .background_pattern(self.ram.display.room_walls[wall_band]);
                wall_band += 1;
            }
            self.tia.horizontal_sync();
        }
        self.tia.blank(true);
        self.tia.second_object.set_row(0);
        self.tia.first_object_row(0);
    }

    // $F0D2: The coordinate replaces the coarse delay, reset strobe, and fine adjustment.
    fn position_drawable(&mut self, drawable: Drawable, x: u8) {
        match drawable {
            Drawable::FirstObject => self.tia.first_object.reposition(x),
            Drawable::SecondObject => self.tia.second_object.reposition(x),
            Drawable::LeftBarrier => self.tia.left_barrier.reposition(x),
            Drawable::RightBarrier => self.tia.right_barrier.reposition(x),
            Drawable::Player => self.tia.player.reposition(x),
        }
    }

    // $F0EC: Present the accumulated frame and sample controls at the vertical boundary.
    fn begin_frame(&mut self, host: &mut impl FrameHost) -> Result<bool, String> {
        self.tia.blank(true);
        match host.vertical_sync(&self.tia.frame, &self.sound)? {
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

    // $F10F: Read the room, choose its appearance, select objects, then prepare each slot.
    fn prepare_room_display(&mut self) {
        self.trace_at(Point::Display);
        self.trace_input(Control::Color);
        let room = self.find_room_record(self.ram.player.room);
        self.ram.display.room_walls = room.walls;
        let color = if self.controls.color {
            room.color
        } else {
            room.monochrome
        };
        self.tia.foreground(self.resolve_display_color(color));
        self.tia
            .background(self.resolve_display_color(ColorChoice::Fixed(8)));
        self.tia.mirror_walls(room.mirrored);
        self.tia.foreground_priority(room.foreground_priority);
        self.tia.player.set_width(room.player_width);
        self.tia.right_barrier.enable(room.right_barrier);
        self.tia.left_barrier.enable(room.left_barrier);

        self.select_visible_objects();
        if self.ram.first_visible == ObjectId::Light
            || self.ram.first_visible == ObjectId::Bridge
                && self.ram.second_visible != ObjectId::Light
        {
            std::mem::swap(&mut self.ram.first_visible, &mut self.ram.second_visible);
        }
        let first = OBJECT_CATALOG[self.ram.first_visible as usize];
        self.ram.display.first_position = self.find_object_position(self.ram.first_visible);
        let state = self.image_state(first.state);
        self.ram.display.first_image = self.select_object_image(first.images, state);
        let color = if self.controls.color {
            first.color
        } else {
            first.monochrome
        };
        self.tia
            .first_object
            .set_color(self.resolve_display_color(color));
        self.tia.first_object.scale(first.style.scale);
        self.tia.first_object.duplicate(first.style.copies);

        let second = OBJECT_CATALOG[self.ram.second_visible as usize];
        self.ram.display.second_position = self.find_object_position(self.ram.second_visible);
        let state = self.image_state(second.state);
        self.ram.display.second_image = self.select_object_image(second.images, state);
        let color = if self.controls.color {
            second.color
        } else {
            second.monochrome
        };
        self.tia
            .second_object
            .set_color(self.resolve_display_color(color));
        self.tia.second_object.scale(second.style.scale);
        self.tia.second_object.duplicate(second.style.copies);
    }

    // State pointers become a closed selection of named game fields and fixed values.
    fn image_state(&self, source: StateSource) -> u8 {
        match source {
            StateSource::Fixed(value) => value,
            StateSource::Dragon(dragon) => self.ram.dragon(dragon).state.value(),
            StateSource::Gate(gate) => self.ram.gates.get(gate).value(),
            StateSource::GameNumber => (self.ram.game.index() * 2) as u8,
            StateSource::BatAnimation => self.ram.bat.animation,
        }
    }

    // $F235: Rotate over the real inhabitants, retaining at most two in this room.
    fn select_visible_objects(&mut self) {
        self.ram.first_visible = ObjectId::Empty;
        self.ram.second_visible = ObjectId::Empty;
        let mut candidate = self.ram.last_candidate;
        loop {
            candidate = OBJECT_IDS[(candidate as usize + 1) % 18];
            if self.find_object_position(candidate).room == self.ram.player.room {
                if self.ram.first_visible == ObjectId::Empty {
                    self.ram.first_visible = candidate;
                } else {
                    self.ram.second_visible = candidate;
                    break;
                }
            }
            if candidate == self.ram.last_candidate {
                break;
            }
        }
        self.ram.last_candidate = candidate;
    }

    // $F271: Typed indexing replaces the nine-times offset and temporary pointer pair.
    fn find_room_record(&self, room: Room) -> &'static RoomDefinition {
        &ROOMS[room as usize]
    }

    // $F2A1: The first inclusive ceiling selects the object's current bitmap.
    fn select_object_image(&self, choices: &'static [ImageChoice], state: u8) -> Bitmap {
        choices
            .iter()
            .find(|choice| state <= choice.ceiling)
            .expect("image ranges end at 255")
            .bitmap
    }

    // $F2B2: The counter also supplies dynamic colors and Game 3's placement seed.
    fn track_inactivity(&mut self) {
        self.trace_at(Point::Inactivity);
        self.trace_input(Control::LeftDirection);
        self.trace_input(Control::RightDirection);
        if self.controls.left.direction.is_idle() && self.controls.right.direction.is_idle() {
            self.trace_input(Control::Reset);
            self.trace_input(Control::Select);
        }
        self.ram.idle_ticks = self.ram.idle_ticks.wrapping_add(1);
        if self.ram.idle_ticks == 0 {
            self.ram.idle_periods = self.ram.idle_periods.wrapping_add(1);
            if self.ram.idle_periods == 0 {
                self.ram.idle_periods = 0x80;
            }
        }
        if !self.controls.left.direction.is_idle()
            || !self.controls.right.direction.is_idle()
            || self.controls.reset
            || self.controls.select
        {
            self.ram.idle_periods = 0;
        }
    }

    // $F2D3: Resolve literal or counter-driven colors, then apply the idle transform.
    fn resolve_display_color(&self, color: ColorChoice) -> u8 {
        let mut value = match color {
            ColorChoice::Fixed(value) => value >> 1,
            ColorChoice::Cycling => self.ram.idle_ticks,
        };
        if self.ram.idle_periods & 0x80 != 0 {
            value = (value ^ self.ram.idle_periods) & 0xfb;
        }
        value.wrapping_shl(1)
    }

    // $F2E4: Position sources distinguish live records from immutable scenery.
    fn find_object_position(&self, object: ObjectId) -> Position {
        match OBJECT_CATALOG[object as usize].position {
            PositionSource::Record(record) => self.ram.position(record),
            PositionSource::Fixed(position) => position,
        }
    }

    // $F2EF: Clear state once, place the fixed barriers, and initialize the selected world.
    fn power_on(&mut self, host: &mut impl FrameHost) -> Result<bool, String> {
        self.ram = Ram::default();
        self.tia = Tia::default();
        self.sound = Sound::default();
        self.position_side_barriers();
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

    // $F306: The call order is the original game loop; the caller repeats this turn.
    pub fn run_game(&mut self, host: &mut impl FrameHost) -> Result<bool, String> {
        if !self.poll_console_switches(host)? {
            return Ok(false);
        }
        self.play_game_sound();
        self.track_inactivity();
        self.trace_checkpoint();
        if !self.ram.won {
            if self.ram.chalice.room == Room::GoldTreasury {
                self.ram.sound_remaining = 255;
                self.ram.won = true;
                self.ram.sound_effect = SoundEffect::Victory;
            }
            self.trace_checkpoint();

            // First picture finishes with contact handling after drawing.
            self.move_player(MovementPhase::Both);
            self.trace_checkpoint();
            self.trace_at(Point::Held);
            self.move_held_object();
            self.trace_checkpoint();
            if !self.begin_frame(host)? {
                return Ok(false);
            }
            self.prepare_room_display();
            self.draw_room();
            self.handle_pickup_and_drop();
            self.trace_checkpoint();

            // The following calls retain their position around the second frame boundary.
            self.move_player(MovementPhase::Vertical);
            self.trace_checkpoint();
            self.trace_at(Point::Light);
            self.update_dark_room_light();
            self.trace_checkpoint();
            if !self.begin_frame(host)? {
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

            if !self.begin_frame(host)? {
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
        } else {
            // Victory uses the shared picture state, then refreshes its appearance.
            if !self.begin_frame(host)? {
                return Ok(false);
            }
            self.draw_room();
            self.prepare_room_display();
        }
        Ok(true)
    }

    // $F371: Barrier drawables persist at the two fixed room edges.
    fn position_side_barriers(&mut self) {
        self.position_drawable(Drawable::LeftBarrier, 13);
        self.position_drawable(Drawable::RightBarrier, 150);
    }

    // $F384: Button edges preserve RESET's partial revival and SELECT's world reset.
    fn poll_console_switches(&mut self, host: &mut impl FrameHost) -> Result<bool, String> {
        self.trace_at(Point::Console);
        self.trace_input(Control::Reset);
        if self.controls.reset && !self.ram.previous_reset {
            if self.ram.won {
                return self.initialize_world(host);
            }
            self.ram.player = Position::new(Room::GoldCastle, 80, 32);
            self.ram.safe_player = self.ram.player;
            self.ram.rhindle.state = DragonState::Wandering;
            self.ram.yorgle.state = DragonState::Wandering;
            self.ram.grundle.state = DragonState::Wandering;
            self.ram.sound_remaining = 0;
            self.ram.held = ObjectId::Empty;
        }
        self.trace_input(Control::Select);
        if self.controls.select && !self.ram.previous_select {
            if self.ram.player.room == Room::GameSelection {
                self.ram.game = self.ram.game.next();
            }
            return self.initialize_world(host);
        }
        self.remember_console_switches();
        self.trace_checkpoint();
        Ok(true)
    }

    // $F3D3: Only the copied world fields are reset; display caches and counters remain live.
    pub fn initialize_world(&mut self, host: &mut impl FrameHost) -> Result<bool, String> {
        self.trace_initialize();
        self.ram.player.room = Room::GameSelection;
        self.ram.safe_player.room = Room::GameSelection;
        self.ram.player.y = 0;
        self.ram.safe_player.y = 0;
        let world = STARTING_WORLDS[self.ram.game.index()];
        self.ram.dot = world.dot;
        self.ram.rhindle = world.rhindle;
        self.ram.yorgle = world.yorgle;
        self.ram.grundle = world.grundle;
        self.ram.magnet = world.magnet;
        self.ram.sword = world.sword;
        self.ram.chalice = world.chalice;
        self.ram.bridge = world.bridge;
        self.ram.gold_key = world.gold_key;
        self.ram.white_key = world.white_key;
        self.ram.black_key = world.black_key;
        self.ram.gates = world.gates;
        self.ram.bat = world.bat;
        self.trace_checkpoint();

        if self.ram.game == Game::Three {
            self.randomize_game_three();
            self.trace_checkpoint();
            if !self.begin_frame(host)? {
                return Ok(false);
            }
            self.draw_room();
        }
        self.ram.won = false;
        self.ram.held = ObjectId::Empty;
        self.remember_console_switches();
        self.trace_checkpoint();
        Ok(true)
    }

    // $F40C: The shared console-poll tail needs no stack or shared pointer state.
    fn remember_console_switches(&mut self) {
        self.trace_at(Point::History);
        self.trace_input(Control::Reset);
        self.trace_input(Control::Select);
        self.ram.previous_reset = self.controls.reset;
        self.ram.previous_select = self.controls.select;
    }

    // $F412: Work backward through the eleven constraints, retaining the counter as seed.
    fn randomize_game_three(&mut self) {
        for bounds in RANDOM_ROOM_BOUNDS.iter().rev() {
            loop {
                let old = self.ram.idle_ticks;
                self.ram.idle_ticks = (old >> 5).wrapping_add(old).wrapping_add(1);
                let candidate = self.ram.idle_ticks & 31;
                if candidate >= bounds.first as u8 && candidate <= bounds.last as u8 {
                    self.ram.position_mut(bounds.record).room = ROOM_IDS[usize::from(candidate)];
                    break;
                }
            }
        }
    }
}

// $F439: Game 3 constraints retain their original order.
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

// $F45A: The last two game selections share a starting world.
const STARTING_WORLDS: [StartingWorld; 3] = [GAME_ONE_WORLD, GAME_TWO_WORLD, GAME_TWO_WORLD];

// $F460: Fixed placements for Game 1.
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

// $F491: Fixed placements for Game 2, before Game 3 randomization.
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

impl Adventure {
    // $F4C2: Roll back blocked movement except within the bridge's traversable corridor.
    fn move_player(&mut self, phase: MovementPhase) {
        self.trace_at(match phase {
            MovementPhase::Both => Point::PlayerBoth,
            MovementPhase::Vertical => Point::PlayerVertical,
            MovementPhase::Horizontal => Point::PlayerHorizontal,
        });
        let blocked = self.trace_blocked();
        let bridge_x = self.ram.player.x.wrapping_sub(self.ram.bridge.x);
        let bridge_y = self.ram.bridge.y.wrapping_sub(self.ram.player.y);
        let crossing_bridge = phase == MovementPhase::Horizontal
            && self.ram.held != ObjectId::Bridge
            && self.ram.player.room == self.ram.bridge.room
            && (10..23).contains(&bridge_x)
            && !(25..252).contains(&bridge_y);
        if !blocked || crossing_bridge {
            self.ram.joystick_direction = Direction::default();
            self.ram.safe_player = self.ram.player;
        }
        if phase == MovementPhase::Both {
            self.trace_input(Control::LeftDirection);
            self.ram.joystick_direction = self.controls.left.direction;
        }
        self.ram.player = self.ram.safe_player;
        let direction = match phase {
            MovementPhase::Both => self.ram.joystick_direction,
            MovementPhase::Vertical => self.ram.joystick_direction.vertical(),
            MovementPhase::Horizontal => self.ram.joystick_direction.horizontal(),
        };
        self.move_object_through_world(RecordId::Player, direction, 3);
    }

    // $F556: Fire drops an item; the last rendered contacts determine pickup candidates.
    fn handle_pickup_and_drop(&mut self) {
        self.trace_at(Point::Pickup);
        self.trace_input(Control::Fire);
        let newly_pressed = self.controls.left.fire && !self.ram.fire_was_pressed;
        self.ram.fire_was_pressed = self.controls.left.fire;
        if newly_pressed && self.ram.held != ObjectId::Empty {
            self.ram.held = ObjectId::Empty;
            self.ram.sound_effect = SoundEffect::Drop;
            self.ram.sound_remaining = 4;
            self.trace_checkpoint();
        }
        let touched = if self.trace_player_slot(0) {
            self.ram.first_visible
        } else if self.trace_player_slot(1) {
            self.ram.second_visible
        } else {
            return;
        };
        if (touched as u8) < ObjectId::Sword as u8
            || self.find_object_position(touched).room != self.ram.player.room
        {
            return;
        }
        let PositionSource::Record(record) = OBJECT_CATALOG[touched as usize].position else {
            return;
        };
        if touched != self.ram.held {
            self.ram.sound_effect = SoundEffect::Pickup;
            self.ram.sound_remaining = 4;
        }
        self.ram.held = touched;
        self.step_object(record, self.ram.joystick_direction, 6);
        let position = self.ram.position(record);
        self.ram.held_offset_x = position.x.wrapping_sub(self.ram.player.x);
        self.ram.held_offset_y = position.y.wrapping_sub(self.ram.player.y);
    }

    // $F5D4: Keep the carried offset, then resolve boundaries without an extra directional step.
    fn move_held_object(&mut self) {
        if self.ram.held == ObjectId::Empty {
            return;
        }
        let PositionSource::Record(record) = OBJECT_CATALOG[self.ram.held as usize].position else {
            return;
        };
        *self.ram.position_mut(record) = Position::new(
            self.ram.player.room,
            self.ram.player.x.wrapping_add(self.ram.held_offset_x),
            self.ram.player.y.wrapping_add(self.ram.held_offset_y),
        );
        self.move_object_through_world(record, Direction::default(), 0);
    }

    // $F5FF: Open castle exits precede ordinary edges, and the secret edge precedes the right wrap.
    fn move_object_through_world(&mut self, record: RecordId, direction: Direction, steps: u8) {
        self.step_object(record, direction, steps);
        for gate in [GateId::Black, GateId::White, GateId::Gold] {
            if self.ram.gates.get(gate) == GateState::Closed {
                continue;
            }
            let position = self.ram.position(record);
            if position.room != CASTLE_INTERIORS[gate.index()]
                || (position.y.wrapping_sub(13) as i8) >= 0
            {
                continue;
            }
            *self.ram.position_mut(record) = Position::new(CASTLE_OUTSIDES[gate.index()], 80, 44);
            self.ram.gates.set(gate, GateState::Open);
            self.trace_transition(record);
            return;
        }
        let mut position = self.ram.position(record);
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
            if position.room == Room::SecretPassage && self.ram.dot.room != Room::BlackMazeDot {
                position.room = Room::RobinettCredits;
                position.x = 3;
                *self.ram.position_mut(record) = position;
                self.trace_transition(record);
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
        *self.ram.position_mut(record) = position;
        self.trace_transition(record);
    }

    // $F6AC: Each independent direction modifies a byte coordinate with wrapping arithmetic.
    fn step_object(&mut self, record: RecordId, direction: Direction, steps: u8) {
        let position = self.ram.position_mut(record);
        for _ in 0..steps {
            if direction.right {
                position.x = position.x.wrapping_add(1);
            }
            if direction.left {
                position.x = position.x.wrapping_sub(1);
            }
            if direction.up {
                position.y = position.y.wrapping_add(1);
            }
            if direction.down {
                position.y = position.y.wrapping_sub(1);
            }
        }
    }

    // $F6D5: Tagged exits select the current game's member of a three-room group.
    fn resolve_game_exit(&self, exit: Exit) -> Room {
        match exit {
            Exit::Room(room) => room,
            Exit::ByGame(group) => GAME_DEPENDENT_EXITS[group][self.ram.game.index()],
        }
    }

    // $F6E9: Only the displayed slots can report player contact.
    fn is_touching_player(&self, object: ObjectId) -> bool {
        if object == self.ram.first_visible {
            self.trace_player_slot(0)
        } else if object == self.ram.second_visible {
            self.trace_player_slot(1)
        } else {
            false
        }
    }

    // $F6FE: Return the other member of a displayed object pair that touched.
    fn find_touching_object(&self, object: ObjectId) -> ObjectId {
        self.trace_object_contact();
        if !self.tia.contacts.objects_touch {
            return ObjectId::Empty;
        }
        if object == self.ram.first_visible {
            self.ram.second_visible
        } else if object == self.ram.second_visible {
            self.ram.first_visible
        } else {
            ObjectId::Empty
        }
    }

    // $F715: No eligible reaction retains motion; an eligible but aligned pair stops motion.
    fn move_creature(
        &mut self,
        creature: Creature,
        list: &[Reaction],
        ignored: Option<RecordId>,
        speed: u8,
    ) -> Option<ChosenReaction> {
        let chosen = self.choose_reaction(list, ignored);
        let previous = match creature {
            Creature::Dragon(dragon) => self.ram.dragon(dragon).direction,
            Creature::Bat => self.ram.bat.direction,
        };
        let direction = chosen.map_or(previous, |chosen| chosen.direction);
        match creature {
            Creature::Dragon(dragon) => self.ram.dragon_mut(dragon).direction = direction,
            Creature::Bat => self.ram.bat.direction = direction,
        }
        self.move_object_through_world(creature.record(), direction, speed);
        chosen
    }

    // $F728: Ordered pairs use separate local identities and return the chosen pair to the bat.
    fn choose_reaction(
        &self,
        list: &[Reaction],
        ignored: Option<RecordId>,
    ) -> Option<ChosenReaction> {
        for reaction in list {
            if self.ram.position(reaction.from).room != self.ram.position(reaction.to).room {
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

    // $F757: Direction is from the first record toward the second, without register aliases.
    fn direction_between_objects(&self, from: RecordId, to: RecordId) -> Direction {
        let from = self.ram.position(from);
        let to = self.ram.position(to);
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

    // $F795: Rhindle moves three steps using his own priorities.
    fn update_rhindle(&mut self) {
        self.update_dragon(DragonId::Rhindle, &RHINDLE_REACTIONS, 3);
    }
}

// $F7A7: Ordered reactions; a slice replaces the zero terminator.
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

impl Adventure {
    // $F7B0: Yorgle moves two steps and includes the Gold Key among his fears.
    fn update_yorgle(&mut self) {
        self.update_dragon(DragonId::Yorgle, &YORGLE_REACTIONS, 2);
    }
}

// $F7C2: Ordered reactions; a slice replaces the zero terminator.
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

impl Adventure {
    // $F7CB: Grundle moves two steps using his guarding priorities.
    fn update_grundle(&mut self) {
        self.update_dragon(DragonId::Grundle, &GRUNDLE_REACTIONS, 2);
    }
}

// $F7DD: Ordered reactions; a slice replaces the zero terminator.
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

impl Adventure {
    // $F7EA: Preserve wandering, sword contact, swallowing, and delayed-bite branch order.
    fn update_dragon(&mut self, dragon: DragonId, reactions: &[Reaction], speed: u8) {
        let object = dragon.object();
        match self.ram.dragon(dragon).state {
            DragonState::Wandering => {
                self.trace_input(Control::RightDifficulty);
                let ignored = if self.controls.right_difficulty == Difficulty::B {
                    Some(RecordId::Sword)
                } else {
                    None
                };
                self.move_creature(Creature::Dragon(dragon), reactions, ignored, speed);
                if self.is_touching_player(object) {
                    self.trace_input(Control::LeftDifficulty);
                    let difficulty = usize::from(self.controls.left_difficulty == Difficulty::A);
                    self.ram.dragon_mut(dragon).state =
                        DragonState::Biting(DRAGON_BITE_DELAYS[self.ram.game.index()][difficulty]);
                    self.ram.dragon_mut(dragon).position.x = self.ram.safe_player.x;
                    self.ram.dragon_mut(dragon).position.y = self.ram.safe_player.y;
                    self.ram.sound_effect = SoundEffect::Roar;
                    self.ram.sound_remaining = 16;
                }
                if self.find_touching_object(object) == ObjectId::Sword {
                    self.ram.dragon_mut(dragon).state = DragonState::Dead;
                    self.ram.sound_effect = SoundEffect::DragonSlain;
                    self.ram.sound_remaining = 16;
                }
            }
            DragonState::Dead => {}
            DragonState::Swallowed => {
                let position = self.ram.dragon(dragon).position;
                self.ram.player = Position::new(
                    position.room,
                    position.x.wrapping_add(3),
                    position.y.wrapping_sub(10),
                );
                self.ram.safe_player = self.ram.player;
            }
            DragonState::Biting(count) => {
                let count = count.wrapping_add(1);
                self.ram.dragon_mut(dragon).state = DragonState::from_value(count);
                if count < 252 || !self.is_touching_player(object) {
                    return;
                }
                self.ram.dragon_mut(dragon).state = DragonState::Swallowed;
                self.ram.sound_effect = SoundEffect::Swallow;
                self.ram.sound_remaining = 16;
                let position = &mut self.ram.dragon_mut(dragon).position;
                position.x = position.x.min(155);
                position.y = position.y.max(23);
            }
        }
    }
}

// $F89F: Pairs are difficulty B/A for Games 1, 2, and 3.
const DRAGON_BITE_DELAYS: [[u8; 2]; 3] = [[0xd0, 0xe8], [0xf0, 0xf6], [0xf0, 0xf6]];

impl Adventure {
    // $F8A5: The bat reuses its previous heading during its flight commitment.
    fn update_bat(&mut self) {
        self.ram.bat.animation = self.ram.bat.animation.wrapping_add(1);
        if self.ram.bat.animation == 8 {
            self.ram.bat.animation = 0;
        }
        if self.ram.bat.flight_timer != 0 {
            self.ram.bat.flight_timer = self.ram.bat.flight_timer.wrapping_add(1);
            self.move_object_through_world(RecordId::Bat, self.ram.bat.direction, 3);
        } else {
            let chosen = self.move_creature(Creature::Bat, &BAT_REACTIONS, self.ram.bat.carried, 3);
            if let Some(chosen) = chosen {
                let target = self.ram.position(chosen.reaction.to);
                let bat = self.ram.bat.position;
                if target.room == bat.room
                    && target.x.wrapping_sub(bat.x).wrapping_add(4) & 0xf8 == 0
                    && target.y.wrapping_sub(bat.y).wrapping_add(4) & 0xf8 == 0
                {
                    self.ram.bat.carried = Some(chosen.reaction.to);
                    self.ram.bat.flight_timer = 16;
                }
            }
        }
        if let Some(cargo) = self.ram.bat.carried {
            let bat = self.ram.bat.position;
            *self.ram.position_mut(cargo) = Position::new(bat.room, bat.x.wrapping_add(8), bat.y);
            if self.trace_crossed(RecordId::Bat) {
                self.trace_transition(cargo);
            }
            if OBJECT_CATALOG[self.ram.held as usize].position_matches(cargo) {
                self.ram.held = ObjectId::Empty;
            }
        }
    }
}

// $F927: Ordered reactions; a slice replaces the zero terminator.
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

impl Adventure {
    // $F93C: Reverse gate order matches the original loop, including entry before animation.
    fn update_castle_gates(&mut self) {
        for gate in [GateId::Black, GateId::White, GateId::Gold] {
            let index = gate.index();
            let touched = self.find_touching_object(GATE_OBJECTS[index]);
            if touched == GATE_KEYS[index] {
                self.ram.gates.set(
                    gate,
                    GateState::from_value(self.ram.gates.get(gate).value().wrapping_add(1)),
                );
            }
            if self.ram.gates.get(gate) != GateState::Closed {
                let record = if self.is_touching_player(GATE_OBJECTS[index]) {
                    self.ram.gates.set(gate, GateState::Open);
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
                    self.ram.position_mut(record).room = CASTLE_INTERIORS[index];
                    self.ram.position_mut(record).y = 16;
                }
            }
            if let GateState::Moving(value) = self.ram.gates.get(gate) {
                let value = value.wrapping_add(1);
                self.ram.gates.set(
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
}

// $F9A7: Gate objects, keys, inside rooms, and outside facades stay in the same order.
const GATE_OBJECTS: [ObjectId; 3] = [ObjectId::GoldGate, ObjectId::WhiteGate, ObjectId::BlackGate];
const GATE_KEYS: [ObjectId; 3] = [ObjectId::GoldKey, ObjectId::WhiteKey, ObjectId::BlackKey];
const CASTLE_INTERIORS: [Room; 3] = [
    Room::GoldTreasury,
    Room::WhiteCastleInterior,
    Room::BlackCastleInterior,
];
const CASTLE_OUTSIDES: [Room; 3] = [Room::GoldCastle, Room::WhiteCastle, Room::BlackCastle];

impl Adventure {
    // $F9B3: The selected item moves toward the magnet's temporarily adjusted target.
    fn update_magnet(&mut self) {
        self.ram.magnet.y = self.ram.magnet.y.wrapping_sub(8);
        if let Some(chosen) = self.choose_reaction(&MAGNET_REACTIONS, None) {
            self.move_object_through_world(chosen.reaction.from, chosen.direction, 1);
        }
        self.ram.magnet.y = self.ram.magnet.y.wrapping_add(8);
    }
}

// $F9DA: Ordered reactions; a slice replaces the zero terminator.
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

impl Adventure {
    // $F9E7: Light position remains distinct from the room and image-selection scratch values.
    fn update_dark_room_light(&mut self) {
        if !matches!(
            self.find_room_record(self.ram.player.room).color,
            ColorChoice::Fixed(8)
        ) {
            self.ram.light.y = 0;
            return;
        }
        self.ram.light.room = self.ram.player.room;
        self.ram.light.x = self.ram.player.x.wrapping_sub(14);
        self.ram.light.y = self.ram.player.y.wrapping_add(14);
        if self.ram.light.x >= 240 {
            self.ram.light.x = 1;
        } else if self.ram.light.x >= 130 {
            self.ram.light.x = 129;
        }
    }

    // $FA23: Advance each event's duration, then set its channel parameters in order.
    fn play_game_sound(&mut self) {
        if self.ram.sound_remaining == 0 {
            self.sound.volume(0, 0);
            self.sound.volume(1, 0);
            return;
        }
        self.ram.sound_remaining = self.ram.sound_remaining.wrapping_sub(1);
        let remaining = self.ram.sound_remaining;
        match self.ram.sound_effect {
            SoundEffect::Victory => {
                self.tia.foreground(remaining);
                self.sound.waveform(0, remaining);
                self.sound.volume(0, remaining >> 1);
                self.sound.frequency(0, remaining >> 3);
            }
            SoundEffect::Roar => {
                self.sound
                    .waveform(0, if remaining & 1 != 0 { 3 } else { 8 });
                self.sound.volume(0, remaining);
                self.sound.frequency(0, (remaining >> 2).wrapping_add(28));
            }
            SoundEffect::Swallow => {
                self.sound.waveform(0, 6);
                self.sound.frequency(0, remaining ^ 15);
                self.sound.volume(0, (remaining >> 1).wrapping_add(8));
            }
            SoundEffect::DragonSlain => {
                self.sound.waveform(0, 4);
                self.sound.volume(0, remaining);
                self.sound.frequency(0, remaining ^ 31);
            }
            SoundEffect::Drop | SoundEffect::Pickup => {
                self.sound.frequency(
                    0,
                    if self.ram.sound_effect == SoundEffect::Drop {
                        remaining ^ 3
                    } else {
                        remaining
                    },
                );
                self.sound.volume(0, 5);
                self.sound.waveform(0, 6);
            }
        }
    }
}

impl ObjectDescriptor {
    fn position_matches(self, record: RecordId) -> bool {
        matches!(self.position, PositionSource::Record(candidate) if candidate == record)
    }
}

// $FAA0: Static data is located here in the assembly source.
static WALLS_SECRET_PASSAGE: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0xff, 0xff),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0xf0, 0xff, 0x0f),
];

// $FAB2: Static data is located here in the assembly source.
static WALLS_OPEN_HALL: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0xff, 0x0f),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0xf0, 0xff, 0xff),
];

// $FAC7: Static data is located here in the assembly source.
static WALLS_SIDE_PASSAGE: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0xff, 0x0f),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0xf0, 0xff, 0x0f),
];

// $FADC: Static data is located here in the assembly source.
static WALLS_ENCLOSED_ROOM: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0xff, 0xff),
    PlayfieldRow::new(0x30, 0x00, 0x00),
    PlayfieldRow::new(0x30, 0x00, 0x00),
    PlayfieldRow::new(0x30, 0x00, 0x00),
    PlayfieldRow::new(0x30, 0x00, 0x00),
    PlayfieldRow::new(0x30, 0x00, 0x00),
    PlayfieldRow::new(0xf0, 0xff, 0x0f),
];

// $FAF1: Static data is located here in the assembly source.
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

// $FB18: Static data is located here in the assembly source.
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

// $FB29: Static data is located here in the assembly source.
static WALLS_BLACK_CASTLE_INTERIOR: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0xff, 0x0f),
    PlayfieldRow::new(0x30, 0x00, 0x00),
    PlayfieldRow::new(0x30, 0x00, 0x00),
    PlayfieldRow::new(0x30, 0x00, 0x00),
    PlayfieldRow::new(0x30, 0x00, 0x00),
    PlayfieldRow::new(0x30, 0x00, 0x00),
    PlayfieldRow::new(0xf0, 0xff, 0x0f),
];

// $FB3E: Static data is located here in the assembly source.
static WALLS_BLUE_MAZE_NORTH: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0xff, 0x0f),
    PlayfieldRow::new(0x00, 0x0c, 0x0c),
    PlayfieldRow::new(0xf0, 0x0c, 0x3c),
    PlayfieldRow::new(0xf0, 0x0c, 0x00),
    PlayfieldRow::new(0xf0, 0xff, 0x3f),
    PlayfieldRow::new(0x00, 0x30, 0x30),
    PlayfieldRow::new(0xf0, 0x33, 0x3f),
];

// $FB53: Static data is located here in the assembly source.
static WALLS_BLUE_MAZE_05: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0xff, 0xff),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0xf0, 0xfc, 0xff),
    PlayfieldRow::new(0xf0, 0x00, 0xc0),
    PlayfieldRow::new(0xf0, 0x3f, 0xcf),
    PlayfieldRow::new(0x00, 0x30, 0xcc),
    PlayfieldRow::new(0xf0, 0xf3, 0xcc),
];

// $FB68: Static data is located here in the assembly source.
static WALLS_BLUE_MAZE_SOUTH: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0xf3, 0x0c),
    PlayfieldRow::new(0x00, 0x30, 0x0c),
    PlayfieldRow::new(0xf0, 0x3f, 0x0f),
    PlayfieldRow::new(0xf0, 0x00, 0x00),
    PlayfieldRow::new(0xf0, 0xf0, 0x00),
    PlayfieldRow::new(0x00, 0x30, 0x00),
    PlayfieldRow::new(0xf0, 0xff, 0xff),
];

// $FB7D: Static data is located here in the assembly source.
static WALLS_BLUE_MAZE_CENTER: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0x33, 0x3f),
    PlayfieldRow::new(0x00, 0x30, 0x3c),
    PlayfieldRow::new(0xf0, 0xff, 0x3c),
    PlayfieldRow::new(0x00, 0x03, 0x3c),
    PlayfieldRow::new(0xf0, 0x33, 0x3c),
    PlayfieldRow::new(0x00, 0x33, 0x0c),
    PlayfieldRow::new(0xf0, 0xf3, 0x0c),
];

// $FB92: Static data is located here in the assembly source.
static WALLS_BLUE_MAZE_ENTRANCE: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0xf3, 0xcc),
    PlayfieldRow::new(0x00, 0x33, 0x0c),
    PlayfieldRow::new(0xf0, 0x33, 0xfc),
    PlayfieldRow::new(0x00, 0x33, 0x00),
    PlayfieldRow::new(0xf0, 0xf3, 0xff),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0xf0, 0xff, 0x0f),
];

// $FBA7: Static data is located here in the assembly source.
static WALLS_CATACOMBS_CENTER: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0xff, 0xcc),
    PlayfieldRow::new(0x00, 0x00, 0xcc),
    PlayfieldRow::new(0xf0, 0x03, 0xcf),
    PlayfieldRow::new(0x00, 0x03, 0x00),
    PlayfieldRow::new(0xf0, 0xf3, 0xfc),
    PlayfieldRow::new(0x00, 0x33, 0x0c),
    PlayfieldRow::new(0xf0, 0x33, 0xcc),
];

// $FBB9: Static data is located here in the assembly source.
static WALLS_CATACOMBS_SIDE: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0x33, 0xcc),
    PlayfieldRow::new(0x00, 0x30, 0xcc),
    PlayfieldRow::new(0x00, 0x3f, 0xcf),
    PlayfieldRow::new(0x00, 0x00, 0xc0),
    PlayfieldRow::new(0x00, 0x3f, 0xc3),
    PlayfieldRow::new(0x00, 0x30, 0xc0),
    PlayfieldRow::new(0xf0, 0xff, 0xff),
];

// $FBCE: Static data is located here in the assembly source.
static WALLS_CATACOMBS_ENTRANCE: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0xff, 0x0f),
    PlayfieldRow::new(0x00, 0x30, 0x00),
    PlayfieldRow::new(0xf0, 0x30, 0xff),
    PlayfieldRow::new(0x00, 0x30, 0xc0),
    PlayfieldRow::new(0xf0, 0xf3, 0xc0),
    PlayfieldRow::new(0x00, 0x03, 0xc0),
    PlayfieldRow::new(0xf0, 0xff, 0xcc),
];

// $FBE3: Static data is located here in the assembly source.
static WALLS_CASTLE: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0xfe, 0x15),
    PlayfieldRow::new(0x30, 0x03, 0x1f),
    PlayfieldRow::new(0x30, 0x03, 0xff),
    PlayfieldRow::new(0x30, 0x00, 0xff),
    PlayfieldRow::new(0x30, 0x00, 0x3f),
    PlayfieldRow::new(0x30, 0x00, 0x00),
    PlayfieldRow::new(0xf0, 0xff, 0x0f),
];

// $FBF8: Static data is located here in the assembly source.
const GOLD_GATE_POSITION: Position = Position::new(Room::GoldCastle, 77, 49);

// $FBFB: Static data is located here in the assembly source.
const WHITE_GATE_POSITION: Position = Position::new(Room::WhiteCastle, 77, 49);

// $FBFE: Static data is located here in the assembly source.
const BLACK_GATE_POSITION: Position = Position::new(Room::BlackCastle, 77, 49);

// $FC02: Static data is located here in the assembly source.
static LIGHT_IMAGES: [ImageChoice; 1] = [ImageChoice {
    ceiling: 0xff,
    bitmap: Bitmap::new(&LIGHT_IMAGE),
}];

// $FC05: Static data is located here in the assembly source.
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

// $FC26: Static data is located here in the assembly source.
static WALLS_RED_MAZE_17: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0xff, 0xff),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0xf0, 0xff, 0x0f),
    PlayfieldRow::new(0x00, 0x00, 0x0c),
    PlayfieldRow::new(0xf0, 0xff, 0x0c),
    PlayfieldRow::new(0xf0, 0x03, 0xcc),
    PlayfieldRow::new(0xf0, 0x33, 0xcf),
];

// $FC38: Static data is located here in the assembly source.
static WALLS_RED_MAZE_SOUTH: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0x33, 0xcf),
    PlayfieldRow::new(0xf0, 0x30, 0x00),
    PlayfieldRow::new(0xf0, 0x33, 0xff),
    PlayfieldRow::new(0x00, 0x33, 0x00),
    PlayfieldRow::new(0xf0, 0xff, 0x00),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0xf0, 0xff, 0x0f),
];

// $FC4D: Static data is located here in the assembly source.
static WALLS_RED_MAZE_NORTH: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0xff, 0xff),
    PlayfieldRow::new(0x00, 0x00, 0xc0),
    PlayfieldRow::new(0xf0, 0xff, 0xcf),
    PlayfieldRow::new(0x00, 0x00, 0xcc),
    PlayfieldRow::new(0xf0, 0x33, 0xff),
    PlayfieldRow::new(0xf0, 0x33, 0x00),
    PlayfieldRow::new(0xf0, 0x3f, 0x0c),
];

// $FC5F: Static data is located here in the assembly source.
static WALLS_WHITE_CASTLE_INTERIOR: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0x3f, 0x0c),
    PlayfieldRow::new(0xf0, 0x00, 0x0c),
    PlayfieldRow::new(0xf0, 0xff, 0x0f),
    PlayfieldRow::new(0x00, 0x30, 0x00),
    PlayfieldRow::new(0xf0, 0x30, 0x00),
    PlayfieldRow::new(0x00, 0x30, 0x00),
    PlayfieldRow::new(0xf0, 0xff, 0x0f),
];

// $FC74: Static data is located here in the assembly source.
static WALLS_TOP_OPENING: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0xff, 0x0f),
    PlayfieldRow::new(0x30, 0x00, 0x00),
    PlayfieldRow::new(0x30, 0x00, 0x00),
    PlayfieldRow::new(0x30, 0x00, 0x00),
    PlayfieldRow::new(0x30, 0x00, 0x00),
    PlayfieldRow::new(0x30, 0x00, 0x00),
    PlayfieldRow::new(0xf0, 0xff, 0xff),
];

// $FC89: Static data is located here in the assembly source.
static WALLS_BLACK_MAZE_13: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0xf0, 0xff),
    PlayfieldRow::new(0x00, 0x00, 0x03),
    PlayfieldRow::new(0xf0, 0xff, 0x03),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0x30, 0x3f, 0xff),
    PlayfieldRow::new(0x00, 0x30, 0x00),
    PlayfieldRow::new(0xf0, 0xf0, 0xff),
];

// $FC9B: Static data is located here in the assembly source.
static WALLS_BLACK_MAZE_DOT: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0xf0, 0xff),
    PlayfieldRow::new(0x30, 0x00, 0x00),
    PlayfieldRow::new(0x30, 0x3f, 0xff),
    PlayfieldRow::new(0x00, 0x30, 0x00),
    PlayfieldRow::new(0xf0, 0xf0, 0xff),
    PlayfieldRow::new(0x30, 0x00, 0x03),
    PlayfieldRow::new(0xf0, 0xf0, 0xff),
];

// $FCB0: Static data is located here in the assembly source.
static WALLS_BLACK_MAZE_14: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0xff, 0xff),
    PlayfieldRow::new(0x00, 0x00, 0xc0),
    PlayfieldRow::new(0xf0, 0xff, 0xcf),
    PlayfieldRow::new(0x00, 0x00, 0x0c),
    PlayfieldRow::new(0xf0, 0x0f, 0xff),
    PlayfieldRow::new(0x00, 0x0f, 0xc0),
    PlayfieldRow::new(0x30, 0xcf, 0xcc),
];

// $FCC2: Static data is located here in the assembly source.
static WALLS_BLACK_MAZE_ENTRANCE: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0x30, 0xcf, 0xcc),
    PlayfieldRow::new(0x00, 0xc0, 0xcc),
    PlayfieldRow::new(0xf0, 0xff, 0x0f),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0xf0, 0xff, 0x0f),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0xf0, 0xff, 0x0f),
];

// $FCD8: Static data is located here in the assembly source.
static BRIDGE_IMAGES: [ImageChoice; 1] = [ImageChoice {
    ceiling: 0xff,
    bitmap: Bitmap::new(&BRIDGE_IMAGE),
}];

// $FCDB: Static data is located here in the assembly source.
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

// $FCF4: Static data is located here in the assembly source.
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

// $FCFD: Static data is located here in the assembly source.
static KEY_IMAGES: [ImageChoice; 1] = [ImageChoice {
    ceiling: 0xff,
    bitmap: Bitmap::new(&KEY_IMAGE),
}];

// $FD00: Static data is located here in the assembly source.
static KEY_IMAGE: [u8; 4] = [
    0x07, // |     ███|
    0xfd, // |██████ █|
    0xa7, // |█ █  ███|
    0x00, // |        |
];

// $FD04: Static data is located here in the assembly source.
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

// $FD0C: Static data is located here in the assembly source.
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

// $FD14: Static data is located here in the assembly source.
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

// $FD1A: Static data is located here in the assembly source.
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

// $FD22: Static data is located here in the assembly source.
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

// $FD2E: Static data is located here in the assembly source.
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

// $FD3A: Static data is located here in the assembly source.
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

// $FD4F: Static data is located here in the assembly source.
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

// $FD66: Static data is located here in the assembly source.
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

// $FD79: Static data is located here in the assembly source.
static SWORD_IMAGES: [ImageChoice; 1] = [ImageChoice {
    ceiling: 0xff,
    bitmap: Bitmap::new(&SWORD_IMAGE),
}];

// $FD7C: Static data is located here in the assembly source.
static SWORD_IMAGE: [u8; 6] = [
    0x20, // |  █     |
    0x40, // | █      |
    0xff, // |████████|
    0x40, // | █      |
    0x20, // |  █     |
    0x00, // |        |
];

// $FD83: Static data is located here in the assembly source.
static DOT_IMAGES: [ImageChoice; 1] = [ImageChoice {
    ceiling: 0xff,
    bitmap: Bitmap::new(&DOT_IMAGE),
}];

// $FD86: Static data is located here in the assembly source.
static DOT_IMAGE: [u8; 2] = [
    0x80, // |█       |
    0x00, // |        |
];

// $FD88: Static data is located here in the assembly source.
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

// $FDE8: Static data is located here in the assembly source.
const ROBINETT_CREDITS_POSITION: Position = Position::new(Room::RobinettCredits, 80, 105);

// $FDEC: Static data is located here in the assembly source.
static ROBINETT_CREDITS_IMAGES: [ImageChoice; 1] = [ImageChoice {
    ceiling: 0xff,
    bitmap: Bitmap::new(&ROBINETT_CREDITS_IMAGE),
}];

// $FDF0: Static data is located here in the assembly source.
static CHALICE_IMAGES: [ImageChoice; 1] = [ImageChoice {
    ceiling: 0xff,
    bitmap: Bitmap::new(&CHALICE_IMAGE),
}];

// $FDF3: Static data is located here in the assembly source.
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

// $FDFE: Static data is located here in the assembly source.
static EMPTY_IMAGES: [ImageChoice; 1] = [ImageChoice {
    ceiling: 0xff,
    bitmap: Bitmap::new(&EMPTY_IMAGE),
}];

// $FE01: Static data is located here in the assembly source.
static EMPTY_IMAGE: [u8; 1] = [
    0x00, // |        |
];

// $FE02: Static data is located here in the assembly source.
const GAME_NUMBER_POSITION: Position = Position::new(Room::GameSelection, 80, 64);

// $FE05: Static data is located here in the assembly source.
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

// $FE0F: Static data is located here in the assembly source.
static MAGNET_IMAGES: [ImageChoice; 1] = [ImageChoice {
    ceiling: 0xff,
    bitmap: Bitmap::new(&MAGNET_IMAGE),
}];

// $FE12: Static data is located here in the assembly source.
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

// $FE1B: Room identifiers and room records retain the original map order.
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

// $FF32: Three destinations per tagged exit, in game order.
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

// $FF44: Position, state, image choices, and appearance for all nineteen catalog entries.
static OBJECT_CATALOG: [ObjectDescriptor; 19] = [
    // Light.
    ObjectDescriptor {
        position: PositionSource::Record(RecordId::Light),
        state: StateSource::Fixed(0),
        images: &LIGHT_IMAGES,
        color: ColorChoice::Fixed(0x28),
        monochrome: ColorChoice::Fixed(0x0c),
        style: SpriteStyle::WIDE,
    },
    // GoldGate.
    ObjectDescriptor {
        position: PositionSource::Fixed(GOLD_GATE_POSITION),
        state: StateSource::Gate(GateId::Gold),
        images: &GATE_IMAGES,
        color: ColorChoice::Fixed(0x00),
        monochrome: ColorChoice::Fixed(0x00),
        style: SpriteStyle::NORMAL,
    },
    // WhiteGate.
    ObjectDescriptor {
        position: PositionSource::Fixed(WHITE_GATE_POSITION),
        state: StateSource::Gate(GateId::White),
        images: &GATE_IMAGES,
        color: ColorChoice::Fixed(0x00),
        monochrome: ColorChoice::Fixed(0x00),
        style: SpriteStyle::NORMAL,
    },
    // BlackGate.
    ObjectDescriptor {
        position: PositionSource::Fixed(BLACK_GATE_POSITION),
        state: StateSource::Gate(GateId::Black),
        images: &GATE_IMAGES,
        color: ColorChoice::Fixed(0x00),
        monochrome: ColorChoice::Fixed(0x00),
        style: SpriteStyle::NORMAL,
    },
    // Credits.
    ObjectDescriptor {
        position: PositionSource::Fixed(ROBINETT_CREDITS_POSITION),
        state: StateSource::Fixed(0),
        images: &ROBINETT_CREDITS_IMAGES,
        color: ColorChoice::Cycling,
        monochrome: ColorChoice::Fixed(0x00),
        style: SpriteStyle::NORMAL,
    },
    // GameNumber.
    ObjectDescriptor {
        position: PositionSource::Fixed(GAME_NUMBER_POSITION),
        state: StateSource::GameNumber,
        images: &GAME_NUMBER_IMAGES,
        color: ColorChoice::Fixed(0xc8),
        monochrome: ColorChoice::Fixed(0x00),
        style: SpriteStyle::NORMAL,
    },
    // Rhindle.
    ObjectDescriptor {
        position: PositionSource::Record(RecordId::Rhindle),
        state: StateSource::Dragon(DragonId::Rhindle),
        images: &DRAGON_IMAGES,
        color: ColorChoice::Fixed(0x36),
        monochrome: ColorChoice::Fixed(0x0e),
        style: SpriteStyle::NORMAL,
    },
    // Yorgle.
    ObjectDescriptor {
        position: PositionSource::Record(RecordId::Yorgle),
        state: StateSource::Dragon(DragonId::Yorgle),
        images: &DRAGON_IMAGES,
        color: ColorChoice::Fixed(0x1a),
        monochrome: ColorChoice::Fixed(0x06),
        style: SpriteStyle::NORMAL,
    },
    // Grundle.
    ObjectDescriptor {
        position: PositionSource::Record(RecordId::Grundle),
        state: StateSource::Dragon(DragonId::Grundle),
        images: &DRAGON_IMAGES,
        color: ColorChoice::Fixed(0xc8),
        monochrome: ColorChoice::Fixed(0x00),
        style: SpriteStyle::NORMAL,
    },
    // Sword.
    ObjectDescriptor {
        position: PositionSource::Record(RecordId::Sword),
        state: StateSource::Fixed(0),
        images: &SWORD_IMAGES,
        color: ColorChoice::Fixed(0x1a),
        monochrome: ColorChoice::Fixed(0x06),
        style: SpriteStyle::NORMAL,
    },
    // Bridge.
    ObjectDescriptor {
        position: PositionSource::Record(RecordId::Bridge),
        state: StateSource::Fixed(0),
        images: &BRIDGE_IMAGES,
        color: ColorChoice::Fixed(0x66),
        monochrome: ColorChoice::Fixed(0x02),
        style: SpriteStyle::WIDE,
    },
    // GoldKey.
    ObjectDescriptor {
        position: PositionSource::Record(RecordId::GoldKey),
        state: StateSource::Fixed(0),
        images: &KEY_IMAGES,
        color: ColorChoice::Fixed(0x1a),
        monochrome: ColorChoice::Fixed(0x06),
        style: SpriteStyle::NORMAL,
    },
    // WhiteKey.
    ObjectDescriptor {
        position: PositionSource::Record(RecordId::WhiteKey),
        state: StateSource::Fixed(0),
        images: &KEY_IMAGES,
        color: ColorChoice::Fixed(0x0e),
        monochrome: ColorChoice::Fixed(0x0e),
        style: SpriteStyle::NORMAL,
    },
    // BlackKey.
    ObjectDescriptor {
        position: PositionSource::Record(RecordId::BlackKey),
        state: StateSource::Fixed(0),
        images: &KEY_IMAGES,
        color: ColorChoice::Fixed(0x00),
        monochrome: ColorChoice::Fixed(0x00),
        style: SpriteStyle::NORMAL,
    },
    // Bat.
    ObjectDescriptor {
        position: PositionSource::Record(RecordId::Bat),
        state: StateSource::BatAnimation,
        images: &BAT_IMAGES,
        color: ColorChoice::Fixed(0x00),
        monochrome: ColorChoice::Fixed(0x00),
        style: SpriteStyle::NORMAL,
    },
    // Dot.
    ObjectDescriptor {
        position: PositionSource::Record(RecordId::Dot),
        state: StateSource::Fixed(0),
        images: &DOT_IMAGES,
        color: ColorChoice::Fixed(0x08),
        monochrome: ColorChoice::Fixed(0x08),
        style: SpriteStyle::NORMAL,
    },
    // Chalice.
    ObjectDescriptor {
        position: PositionSource::Record(RecordId::Chalice),
        state: StateSource::Fixed(0),
        images: &CHALICE_IMAGES,
        color: ColorChoice::Cycling,
        monochrome: ColorChoice::Fixed(0x06),
        style: SpriteStyle::NORMAL,
    },
    // Magnet.
    ObjectDescriptor {
        position: PositionSource::Record(RecordId::Magnet),
        state: StateSource::Fixed(0),
        images: &MAGNET_IMAGES,
        color: ColorChoice::Fixed(0x00),
        monochrome: ColorChoice::Fixed(0x06),
        style: SpriteStyle::NORMAL,
    },
    // Empty.
    ObjectDescriptor {
        position: PositionSource::Record(RecordId::Bridge),
        state: StateSource::Fixed(0),
        images: &EMPTY_IMAGES,
        color: ColorChoice::Fixed(0x00),
        monochrome: ColorChoice::Fixed(0x00),
        style: SpriteStyle::NORMAL,
    },
];

#[cfg(test)]
#[path = "adventure_tests.rs"]
mod tests;
