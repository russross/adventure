use super::super::model::{
    Bat, Dragon, DragonState, GateState, Gates, Position, RecordId, Room, RoomBounds, StartingWorld,
};
use crate::input::Direction;

pub(in crate::game) static RANDOM_ROOM_BOUNDS: [RoomBounds; 11] = [
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

pub(in crate::game) const STARTING_WORLDS: [StartingWorld; 3] =
    [GAME_ONE_WORLD, GAME_TWO_WORLD, GAME_TWO_WORLD];

pub(in crate::game) const GAME_ONE_WORLD: StartingWorld = StartingWorld {
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

pub(in crate::game) const GAME_TWO_WORLD: StartingWorld = StartingWorld {
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
