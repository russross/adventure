use super::super::model::{ColorChoice, Exit, Room, RoomDefinition};
use crate::video::PlayfieldRow;

pub(in crate::game) static WALLS_SECRET_PASSAGE: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0xff, 0xff),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0xf0, 0xff, 0x0f),
];

pub(in crate::game) static WALLS_OPEN_HALL: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0xff, 0x0f),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0xf0, 0xff, 0xff),
];

pub(in crate::game) static WALLS_SIDE_PASSAGE: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0xff, 0x0f),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0xf0, 0xff, 0x0f),
];

pub(in crate::game) static WALLS_ENCLOSED_ROOM: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0xff, 0xff),
    PlayfieldRow::new(0x30, 0x00, 0x00),
    PlayfieldRow::new(0x30, 0x00, 0x00),
    PlayfieldRow::new(0x30, 0x00, 0x00),
    PlayfieldRow::new(0x30, 0x00, 0x00),
    PlayfieldRow::new(0x30, 0x00, 0x00),
    PlayfieldRow::new(0xf0, 0xff, 0x0f),
];

pub(in crate::game) static WALLS_BLACK_CASTLE_INTERIOR: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0xff, 0x0f),
    PlayfieldRow::new(0x30, 0x00, 0x00),
    PlayfieldRow::new(0x30, 0x00, 0x00),
    PlayfieldRow::new(0x30, 0x00, 0x00),
    PlayfieldRow::new(0x30, 0x00, 0x00),
    PlayfieldRow::new(0x30, 0x00, 0x00),
    PlayfieldRow::new(0xf0, 0xff, 0x0f),
];

pub(in crate::game) static WALLS_BLUE_MAZE_NORTH: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0xff, 0x0f),
    PlayfieldRow::new(0x00, 0x0c, 0x0c),
    PlayfieldRow::new(0xf0, 0x0c, 0x3c),
    PlayfieldRow::new(0xf0, 0x0c, 0x00),
    PlayfieldRow::new(0xf0, 0xff, 0x3f),
    PlayfieldRow::new(0x00, 0x30, 0x30),
    PlayfieldRow::new(0xf0, 0x33, 0x3f),
];

pub(in crate::game) static WALLS_BLUE_MAZE_05: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0xff, 0xff),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0xf0, 0xfc, 0xff),
    PlayfieldRow::new(0xf0, 0x00, 0xc0),
    PlayfieldRow::new(0xf0, 0x3f, 0xcf),
    PlayfieldRow::new(0x00, 0x30, 0xcc),
    PlayfieldRow::new(0xf0, 0xf3, 0xcc),
];

pub(in crate::game) static WALLS_BLUE_MAZE_SOUTH: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0xf3, 0x0c),
    PlayfieldRow::new(0x00, 0x30, 0x0c),
    PlayfieldRow::new(0xf0, 0x3f, 0x0f),
    PlayfieldRow::new(0xf0, 0x00, 0x00),
    PlayfieldRow::new(0xf0, 0xf0, 0x00),
    PlayfieldRow::new(0x00, 0x30, 0x00),
    PlayfieldRow::new(0xf0, 0xff, 0xff),
];

pub(in crate::game) static WALLS_BLUE_MAZE_CENTER: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0x33, 0x3f),
    PlayfieldRow::new(0x00, 0x30, 0x3c),
    PlayfieldRow::new(0xf0, 0xff, 0x3c),
    PlayfieldRow::new(0x00, 0x03, 0x3c),
    PlayfieldRow::new(0xf0, 0x33, 0x3c),
    PlayfieldRow::new(0x00, 0x33, 0x0c),
    PlayfieldRow::new(0xf0, 0xf3, 0x0c),
];

pub(in crate::game) static WALLS_BLUE_MAZE_ENTRANCE: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0xf3, 0xcc),
    PlayfieldRow::new(0x00, 0x33, 0x0c),
    PlayfieldRow::new(0xf0, 0x33, 0xfc),
    PlayfieldRow::new(0x00, 0x33, 0x00),
    PlayfieldRow::new(0xf0, 0xf3, 0xff),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0xf0, 0xff, 0x0f),
];

pub(in crate::game) static WALLS_CATACOMBS_CENTER: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0xff, 0xcc),
    PlayfieldRow::new(0x00, 0x00, 0xcc),
    PlayfieldRow::new(0xf0, 0x03, 0xcf),
    PlayfieldRow::new(0x00, 0x03, 0x00),
    PlayfieldRow::new(0xf0, 0xf3, 0xfc),
    PlayfieldRow::new(0x00, 0x33, 0x0c),
    PlayfieldRow::new(0xf0, 0x33, 0xcc),
];

pub(in crate::game) static WALLS_CATACOMBS_SIDE: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0x33, 0xcc),
    PlayfieldRow::new(0x00, 0x30, 0xcc),
    PlayfieldRow::new(0x00, 0x3f, 0xcf),
    PlayfieldRow::new(0x00, 0x00, 0xc0),
    PlayfieldRow::new(0x00, 0x3f, 0xc3),
    PlayfieldRow::new(0x00, 0x30, 0xc0),
    PlayfieldRow::new(0xf0, 0xff, 0xff),
];

pub(in crate::game) static WALLS_CATACOMBS_ENTRANCE: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0xff, 0x0f),
    PlayfieldRow::new(0x00, 0x30, 0x00),
    PlayfieldRow::new(0xf0, 0x30, 0xff),
    PlayfieldRow::new(0x00, 0x30, 0xc0),
    PlayfieldRow::new(0xf0, 0xf3, 0xc0),
    PlayfieldRow::new(0x00, 0x03, 0xc0),
    PlayfieldRow::new(0xf0, 0xff, 0xcc),
];

pub(in crate::game) static WALLS_CASTLE: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0xfe, 0x15),
    PlayfieldRow::new(0x30, 0x03, 0x1f),
    PlayfieldRow::new(0x30, 0x03, 0xff),
    PlayfieldRow::new(0x30, 0x00, 0xff),
    PlayfieldRow::new(0x30, 0x00, 0x3f),
    PlayfieldRow::new(0x30, 0x00, 0x00),
    PlayfieldRow::new(0xf0, 0xff, 0x0f),
];

pub(in crate::game) static WALLS_RED_MAZE_17: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0xff, 0xff),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0xf0, 0xff, 0x0f),
    PlayfieldRow::new(0x00, 0x00, 0x0c),
    PlayfieldRow::new(0xf0, 0xff, 0x0c),
    PlayfieldRow::new(0xf0, 0x03, 0xcc),
    PlayfieldRow::new(0xf0, 0x33, 0xcf),
];

pub(in crate::game) static WALLS_RED_MAZE_SOUTH: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0x33, 0xcf),
    PlayfieldRow::new(0xf0, 0x30, 0x00),
    PlayfieldRow::new(0xf0, 0x33, 0xff),
    PlayfieldRow::new(0x00, 0x33, 0x00),
    PlayfieldRow::new(0xf0, 0xff, 0x00),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0xf0, 0xff, 0x0f),
];

pub(in crate::game) static WALLS_RED_MAZE_NORTH: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0xff, 0xff),
    PlayfieldRow::new(0x00, 0x00, 0xc0),
    PlayfieldRow::new(0xf0, 0xff, 0xcf),
    PlayfieldRow::new(0x00, 0x00, 0xcc),
    PlayfieldRow::new(0xf0, 0x33, 0xff),
    PlayfieldRow::new(0xf0, 0x33, 0x00),
    PlayfieldRow::new(0xf0, 0x3f, 0x0c),
];

pub(in crate::game) static WALLS_WHITE_CASTLE_INTERIOR: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0x3f, 0x0c),
    PlayfieldRow::new(0xf0, 0x00, 0x0c),
    PlayfieldRow::new(0xf0, 0xff, 0x0f),
    PlayfieldRow::new(0x00, 0x30, 0x00),
    PlayfieldRow::new(0xf0, 0x30, 0x00),
    PlayfieldRow::new(0x00, 0x30, 0x00),
    PlayfieldRow::new(0xf0, 0xff, 0x0f),
];

pub(in crate::game) static WALLS_TOP_OPENING: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0xff, 0x0f),
    PlayfieldRow::new(0x30, 0x00, 0x00),
    PlayfieldRow::new(0x30, 0x00, 0x00),
    PlayfieldRow::new(0x30, 0x00, 0x00),
    PlayfieldRow::new(0x30, 0x00, 0x00),
    PlayfieldRow::new(0x30, 0x00, 0x00),
    PlayfieldRow::new(0xf0, 0xff, 0xff),
];

pub(in crate::game) static WALLS_BLACK_MAZE_13: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0xf0, 0xff),
    PlayfieldRow::new(0x00, 0x00, 0x03),
    PlayfieldRow::new(0xf0, 0xff, 0x03),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0x30, 0x3f, 0xff),
    PlayfieldRow::new(0x00, 0x30, 0x00),
    PlayfieldRow::new(0xf0, 0xf0, 0xff),
];

pub(in crate::game) static WALLS_BLACK_MAZE_DOT: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0xf0, 0xff),
    PlayfieldRow::new(0x30, 0x00, 0x00),
    PlayfieldRow::new(0x30, 0x3f, 0xff),
    PlayfieldRow::new(0x00, 0x30, 0x00),
    PlayfieldRow::new(0xf0, 0xf0, 0xff),
    PlayfieldRow::new(0x30, 0x00, 0x03),
    PlayfieldRow::new(0xf0, 0xf0, 0xff),
];

pub(in crate::game) static WALLS_BLACK_MAZE_14: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0xf0, 0xff, 0xff),
    PlayfieldRow::new(0x00, 0x00, 0xc0),
    PlayfieldRow::new(0xf0, 0xff, 0xcf),
    PlayfieldRow::new(0x00, 0x00, 0x0c),
    PlayfieldRow::new(0xf0, 0x0f, 0xff),
    PlayfieldRow::new(0x00, 0x0f, 0xc0),
    PlayfieldRow::new(0x30, 0xcf, 0xcc),
];

pub(in crate::game) static WALLS_BLACK_MAZE_ENTRANCE: [PlayfieldRow; 7] = [
    PlayfieldRow::new(0x30, 0xcf, 0xcc),
    PlayfieldRow::new(0x00, 0xc0, 0xcc),
    PlayfieldRow::new(0xf0, 0xff, 0x0f),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0xf0, 0xff, 0x0f),
    PlayfieldRow::new(0x00, 0x00, 0x00),
    PlayfieldRow::new(0xf0, 0xff, 0x0f),
];

pub(in crate::game) const ROOM_IDS: [Room; 31] = [
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

pub(in crate::game) static ROOMS: [RoomDefinition; 31] = [
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

pub(in crate::game) static GAME_DEPENDENT_EXITS: [[Room; 3]; 6] = [
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
