//! Room connections, artwork, palettes, and collision-relevant appearance.

use super::artwork::{
    WALLS_BLACK_CASTLE_INTERIOR, WALLS_BLACK_MAZE_13, WALLS_BLACK_MAZE_14, WALLS_BLACK_MAZE_DOT,
    WALLS_BLACK_MAZE_ENTRANCE, WALLS_BLUE_MAZE_05, WALLS_BLUE_MAZE_CENTER,
    WALLS_BLUE_MAZE_ENTRANCE, WALLS_BLUE_MAZE_NORTH, WALLS_BLUE_MAZE_SOUTH, WALLS_CASTLE,
    WALLS_CATACOMBS_CENTER, WALLS_CATACOMBS_ENTRANCE, WALLS_CATACOMBS_SIDE, WALLS_ENCLOSED_ROOM,
    WALLS_OPEN_HALL, WALLS_RED_MAZE_17, WALLS_RED_MAZE_NORTH, WALLS_RED_MAZE_SOUTH,
    WALLS_SECRET_PASSAGE, WALLS_SIDE_PASSAGE, WALLS_TOP_OPENING, WALLS_WHITE_CASTLE_INTERIOR,
};
use super::{Game, PlayfieldRow, Room, RoomEdge};
use Exit::{ByGame, Fixed};

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
pub(super) struct Features {
    pub dark: bool,
    pub mirrored: bool,
    pub barriers: [bool; 2],
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
    pub walls: &'static [PlayfieldRow; 7],
    pub color: u8,
    pub monochrome: u8,
    pub features: Features,
}

impl RoomDefinition {
    pub fn exit(&self, edge: RoomEdge, game: Game) -> Room {
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
