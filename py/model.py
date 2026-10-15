"""Game identities and byte-sized world records."""

from __future__ import annotations

from dataclasses import dataclass, field
from enum import IntEnum, auto


class Room(IntEnum):
    GameSelection = 0
    WestHall = auto()
    BelowGoldCastle = auto()
    SecretPassage = auto()
    BlueMazeNorth = auto()
    BlueMaze05 = auto()
    BlueMazeSouth = auto()
    BlueMazeCenter = auto()
    BlueMazeEntrance = auto()
    CatacombsCenter = auto()
    CatacombsEntrance = auto()
    CatacombsSide = auto()
    SidePassage0c = auto()
    SidePassage0d = auto()
    AboveWhiteCastle = auto()
    WhiteCastle = auto()
    BlackCastle = auto()
    GoldCastle = auto()
    GoldTreasury = auto()
    BlackMaze13 = auto()
    BlackMaze14 = auto()
    BlackMazeDot = auto()
    BlackMazeEntrance = auto()
    RedMaze17 = auto()
    RedMazeNorth = auto()
    RedMazeSouth = auto()
    WhiteCastleInterior = auto()
    BlackCastleInterior = auto()
    PurpleChamber = auto()
    RedHall = auto()
    RobinettCredits = auto()

    def is_dark(self) -> bool:
        return self in DARK_ROOMS


DARK_ROOMS = frozenset(
    (
        Room.CatacombsCenter,
        Room.CatacombsEntrance,
        Room.CatacombsSide,
        Room.BlackMaze13,
        Room.BlackMaze14,
        Room.BlackMazeDot,
        Room.BlackMazeEntrance,
    )
)


class ObjectId(IntEnum):
    Light = 0
    GoldGate = auto()
    WhiteGate = auto()
    BlackGate = auto()
    Credits = auto()
    GameNumber = auto()
    Rhindle = auto()
    Yorgle = auto()
    Grundle = auto()
    Sword = auto()
    Bridge = auto()
    GoldKey = auto()
    WhiteKey = auto()
    BlackKey = auto()
    Bat = auto()
    Dot = auto()
    Chalice = auto()
    Magnet = auto()
    Empty = auto()


class RecordId(IntEnum):
    Player = 0
    Light = auto()
    Dot = auto()
    Rhindle = auto()
    Yorgle = auto()
    Grundle = auto()
    Magnet = auto()
    Sword = auto()
    Chalice = auto()
    Bridge = auto()
    GoldKey = auto()
    WhiteKey = auto()
    BlackKey = auto()
    Bat = auto()


# The empty display slot borrows the bridge position but has blank artwork.
OBJECT_RECORDS = {obj: RecordId[obj.name] for obj in ObjectId if obj.name in RecordId.__members__}
OBJECT_RECORDS[ObjectId.Empty] = RecordId.Bridge
DRAGONS = (RecordId.Rhindle, RecordId.Yorgle, RecordId.Grundle)


class Game(IntEnum):
    One = 0
    Two = 1
    Three = 2


class Difficulty(IntEnum):
    A = 0
    B = 1


class Edge(IntEnum):
    North = 0
    East = 1
    South = 2
    West = 3


class Phase(IntEnum):
    Player = 0
    BatAndGates = 1
    RemainingActors = 2


@dataclass(frozen=True)
class Position:
    room: Room = Room.GameSelection
    x: int = 0
    y: int = 0


@dataclass(frozen=True)
class Direction:
    right: bool = False
    left: bool = False
    up: bool = False
    down: bool = False

    def is_idle(self) -> bool:
        return not (self.right or self.left or self.up or self.down)

    def vertical(self) -> Direction:
        return Direction(up=self.up, down=self.down)

    def horizontal(self) -> Direction:
        return Direction(right=self.right, left=self.left)


@dataclass
class Joystick:
    direction: Direction = field(default_factory=Direction)
    fire: bool = False


@dataclass
class Controls:
    left: Joystick = field(default_factory=Joystick)
    right: Joystick = field(default_factory=Joystick)
    select: bool = False
    reset: bool = False
    color: bool = True
    left_difficulty: Difficulty = Difficulty.B
    right_difficulty: Difficulty = Difficulty.B


# Countdown states keep their cartridge values because overflow restarts wandering.
class DragonState(IntEnum):
    Wandering = 0
    Dead = 1
    Swallowed = 2
    Biting = 3


class GateState(IntEnum):
    Open = 1
    Closed = 28


@dataclass
class Dragon:
    direction: Direction = field(default_factory=Direction)
    state: DragonState = DragonState.Wandering
    bite_timer: int = 0


@dataclass
class Bat:
    direction: Direction = field(default_factory=Direction)
    animation: int = 0
    carried: RecordId | None = None
    flight_timer: int = 0


@dataclass
class World:
    positions: dict[RecordId, Position] = field(
        default_factory=lambda: {record: Position() for record in RecordId}
    )
    dragons: dict[RecordId, Dragon] = field(
        default_factory=lambda: {record: Dragon() for record in DRAGONS}
    )
    bat: Bat = field(default_factory=Bat)
    gates: dict[ObjectId, int] = field(
        default_factory=lambda: {
            gate: 0 for gate in (ObjectId.GoldGate, ObjectId.WhiteGate, ObjectId.BlackGate)
        }
    )
    safe_player: Position = field(default_factory=Position)
    game: Game = Game.One
    won: bool = False
    held: ObjectId = ObjectId.Light
    held_offset_x: int = 0
    held_offset_y: int = 0
    first_visible: ObjectId = ObjectId.Light
    second_visible: ObjectId = ObjectId.Light
    last_candidate: ObjectId = ObjectId.Light
    joystick_direction: Direction = field(default_factory=Direction)
    idle_ticks: int = 0
    idle_periods: int = 0
    previous_reset: bool = True
    previous_select: bool = True
    fire_was_pressed: bool = True

    @property
    def player(self) -> Position:
        return self.positions[RecordId.Player]

    @player.setter
    def player(self, position: Position) -> None:
        self.positions[RecordId.Player] = position


# Gate order is also interaction priority: black, white, then gold.
CASTLES = (
    (ObjectId.BlackGate, ObjectId.BlackKey, Room.BlackCastleInterior, Room.BlackCastle),
    (ObjectId.WhiteGate, ObjectId.WhiteKey, Room.WhiteCastleInterior, Room.WhiteCastle),
    (ObjectId.GoldGate, ObjectId.GoldKey, Room.GoldTreasury, Room.GoldCastle),
)


def signed(value: int) -> int:
    value &= 255
    return value if value < 128 else value - 256
