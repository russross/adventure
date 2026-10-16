//! Original visual profile: snapshot preparation, coordinate calibration, and room definitions.

use super::assets::{
    BAT_FRAME_ONE, BAT_FRAME_TWO, BRIDGE_IMAGE, CHALICE_IMAGE, DOT_IMAGE, DRAGON_BITING_IMAGE,
    DRAGON_DEAD_IMAGE, DRAGON_WALKING_IMAGE, EMPTY_IMAGE, GAME_ONE_IMAGE, GAME_THREE_IMAGE,
    GAME_TWO_IMAGE, GATE_IMAGE, KEY_IMAGE, LIGHT_IMAGE, MAGNET_IMAGE, ROBINETT_CREDITS_IMAGE,
    SWORD_IMAGE, WALLS_BLACK_CASTLE_INTERIOR, WALLS_BLACK_MAZE_13, WALLS_BLACK_MAZE_14,
    WALLS_BLACK_MAZE_DOT, WALLS_BLACK_MAZE_ENTRANCE, WALLS_BLUE_MAZE_05, WALLS_BLUE_MAZE_CENTER,
    WALLS_BLUE_MAZE_ENTRANCE, WALLS_BLUE_MAZE_NORTH, WALLS_BLUE_MAZE_SOUTH, WALLS_CASTLE,
    WALLS_CATACOMBS_CENTER, WALLS_CATACOMBS_ENTRANCE, WALLS_CATACOMBS_SIDE, WALLS_ENCLOSED_ROOM,
    WALLS_OPEN_HALL, WALLS_RED_MAZE_17, WALLS_RED_MAZE_NORTH, WALLS_RED_MAZE_SOUTH,
    WALLS_SECRET_PASSAGE, WALLS_SIDE_PASSAGE, WALLS_TOP_OPENING, WALLS_WHITE_CASTLE_INTERIOR,
};
use super::{
    Adventure, Bitmap, ColorChoice, DragonState, Game, GateId, OBJECT_IDS, ObjectDisplay, ObjectId,
    Position, Room, RoomEdge,
};
use crate::video::{Appearance, Geometry, HEIGHT, Picture, PlayfieldRow, SpriteWidth, WIDTH};
use Exit::{ByGame, Fixed};
use adventure_trace::Point;

// Display preparation and complete-picture construction.

impl Adventure {
    // Room layers remain frozen between display refreshes; the player is sampled for every picture.
    pub(super) fn draw_room(&mut self) {
        let player_rows = player_rows(self.world.player.y, self.scheduler.player_visible);
        let picture = Picture {
            appearance: self.display.appearance,
            objects: self.display.objects,
            walls: self.display.walls,
            player_x: screen_x(positioned_object_x(self.world.player.x) - 1),
            player_visible: player_rows,
        };
        self.scheduler.player_visible = player_rows[HEIGHT - 1];
        let geometry = Geometry::prepare(&picture);
        self.contacts = geometry.contacts();
        self.video.compose(&picture, &geometry);
    }

    // Appearance and images are a snapshot, reused by the following two pictures.
    pub(super) fn prepare_room_display(&mut self) {
        self.trace_at(Point::Display);
        let color = self.read_color();
        let (walls, appearance) = self.room_appearance(self.world.player.room, color);
        self.display.walls = wall_rows(walls);
        self.display.appearance = appearance;
        self.select_visible_objects();
        if self.display.visible[0] == Some(ObjectId::Light)
            || self.display.visible[0] == Some(ObjectId::Bridge)
                && self.display.visible[1] != Some(ObjectId::Light)
        {
            self.display.visible.swap(0, 1);
        }

        // Resolve the live object's appearance once; following pictures reuse this snapshot.
        for (slot, object) in self.display.visible.into_iter().enumerate() {
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
    pub(super) fn select_visible_objects(&mut self) {
        self.display.visible = [None; 2];
        let mut candidate = self.display.last_candidate;
        loop {
            candidate = OBJECT_IDS[(candidate as usize + 1) % OBJECT_IDS.len()];
            if self.find_object_position(candidate).room == self.world.player.room {
                if self.display.visible[0].is_none() {
                    self.display.visible[0] = Some(candidate);
                } else {
                    self.display.visible[1] = Some(candidate);
                    break;
                }
            }
            if candidate == self.display.last_candidate {
                break;
            }
        }
        self.display.last_candidate = candidate;
    }

    // Wall artwork varies by room; darkness controls both the light and foreground priority.
    pub(super) fn room_appearance(
        &self,
        room: Room,
        color: bool,
    ) -> (&'static [PlayfieldRow; 7], Appearance) {
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
    pub(super) fn resolve_display_color(&self, color: ColorChoice) -> u8 {
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
    pub(super) fn find_object_position(&self, object: ObjectId) -> Position {
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
    pub(super) fn object_display(&self, object: ObjectId) -> ObjectDisplay {
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
    pub(super) fn gate_image(&self, gate: GateId) -> &'static [u8] {
        let value = self.world.gates.get(gate).original_value();
        let stage = usize::from(value.saturating_sub(1) / 4).min(12);
        let start = stage.abs_diff(6) * 2;
        &GATE_IMAGE[start..]
    }
}

pub(super) fn dragon_image(state: DragonState) -> &'static [u8] {
    match state {
        DragonState::Wandering | DragonState::Swallowed => &DRAGON_WALKING_IMAGE,
        DragonState::Dead => &DRAGON_DEAD_IMAGE,
        DragonState::Biting(_) => &DRAGON_BITING_IMAGE,
    }
}

// Wall changes begin on the second row of a logical band boundary.
pub(super) fn wall_rows(bands: &[PlayfieldRow; 7]) -> [PlayfieldRow; HEIGHT] {
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
pub(super) fn sprite_rows(bitmap: Bitmap, y: u8, slot: usize) -> [u8; HEIGHT] {
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
pub(super) fn player_rows(y: u8, retained: bool) -> [bool; HEIGHT] {
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
pub(super) fn below(row: u8, y: u8) -> bool {
    (row.wrapping_sub(y) as i8) < 0
}

// Game coordinates encode the cartridge's calibrated placement, including line crossings.
pub(super) fn positioned_object_x(x: u8) -> i16 {
    match x {
        0..=164 => i16::from(x) - 1,
        165..=239 => i16::from(x % 15) - 4,
        240..=255 => i16::from(x) - 229,
    }
}

pub(super) fn object_screen_x(x: u8, width: SpriteWidth) -> u8 {
    let expansion_shift = i16::from(matches!(width, SpriteWidth::Wide));
    screen_x(positioned_object_x(x) + expansion_shift)
}

// Normalize after the signed correction so the left edge wraps through pixel 159.
pub(super) fn screen_x(x: i16) -> u8 {
    x.rem_euclid(WIDTH as i16) as u8
}

pub(super) const ROOM_IDS: [Room; 31] = [
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
    pub(super) fn destination(self, game: Game) -> Room {
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
    pub(super) fn new(north: Exit, east: Exit, south: Exit, west: Exit) -> Self {
        Self {
            north,
            east,
            south,
            west,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct Features {
    pub(super) dark: bool,
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

pub(super) struct RoomDefinition {
    exits: Exits,
    walls: &'static [PlayfieldRow; 7],
    color: u8,
    monochrome: u8,
    pub(super) features: Features,
}

impl RoomDefinition {
    pub(super) fn exit(&self, edge: RoomEdge, game: Game) -> Room {
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
    pub(super) fn definition(self) -> RoomDefinition {
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
