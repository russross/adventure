"""Room connections and shared wall layouts."""

import artwork as art
from model import Edge, Game, Room

# Exits run north, east, south, west. Game-dependent exits are resolved below.
EXITS = {
    Room.GameSelection: (
        Room.GameSelection,
        Room.GameSelection,
        Room.GameSelection,
        Room.GameSelection,
    ),
    Room.WestHall: (
        Room.BlueMazeEntrance,
        Room.BelowGoldCastle,
        Room.BlackCastle,
        Room.SecretPassage,
    ),
    Room.BelowGoldCastle: (Room.GoldCastle, Room.SecretPassage, Room.BlueMaze05, Room.WestHall),
    Room.SecretPassage: (Room.BlueMazeSouth, Room.WestHall, Room.RedHall, Room.BelowGoldCastle),
    Room.BlueMazeNorth: (
        Room.BlackCastle,
        Room.BlueMaze05,
        Room.BlueMazeCenter,
        Room.BlueMazeSouth,
    ),
    Room.BlueMaze05: (Room.RedHall, Room.BlueMazeSouth, Room.BlueMazeEntrance, Room.BlueMazeNorth),
    Room.BlueMazeSouth: (
        Room.BlueMazeCenter,
        Room.BlueMazeNorth,
        Room.SecretPassage,
        Room.BlueMaze05,
    ),
    Room.BlueMazeCenter: (
        Room.BlueMazeNorth,
        Room.BlueMazeEntrance,
        Room.BlueMazeSouth,
        Room.BlueMazeEntrance,
    ),
    Room.BlueMazeEntrance: (
        Room.BlueMaze05,
        Room.BlueMazeCenter,
        Room.WestHall,
        Room.BlueMazeCenter,
    ),
    Room.CatacombsCenter: (
        Room.CatacombsEntrance,
        Room.CatacombsEntrance,
        Room.CatacombsSide,
        Room.CatacombsEntrance,
    ),
    Room.CatacombsEntrance: (
        Room.SecretPassage,
        Room.CatacombsCenter,
        Room.CatacombsCenter,
        Room.CatacombsCenter,
    ),
    Room.CatacombsSide: (
        Room.CatacombsCenter,
        Room.SidePassage0c,
        Room.PurpleChamber,
        Room.SidePassage0d,
    ),
    Room.SidePassage0c: (Room.PurpleChamber, Room.SidePassage0d, Room.RedHall, Room.CatacombsSide),
    Room.SidePassage0d: (
        Room.WhiteCastle,
        Room.CatacombsSide,
        Room.AboveWhiteCastle,
        Room.SidePassage0c,
    ),
    Room.AboveWhiteCastle: (
        Room.SidePassage0d,
        Room.BlackCastle,
        Room.WhiteCastle,
        Room.BlackCastle,
    ),
    Room.WhiteCastle: (
        Room.AboveWhiteCastle,
        Room.WhiteCastle,
        Room.SidePassage0d,
        Room.WhiteCastle,
    ),
    Room.BlackCastle: (Room.WestHall, Room.PurpleChamber, Room.BlueMazeNorth, Room.PurpleChamber),
    Room.GoldCastle: (Room.BlueMazeSouth, Room.SecretPassage, Room.BelowGoldCastle, Room.WestHall),
    Room.GoldTreasury: (Room.GoldTreasury, Room.GoldTreasury, Room.GoldTreasury, Room.GoldTreasury),
    Room.BlackMaze13: (
        Room.BlackMazeDot,
        Room.BlackMaze14,
        Room.BlackMazeDot,
        Room.BlackMazeEntrance,
    ),
    Room.BlackMaze14: (
        Room.BlackMazeEntrance,
        Room.BlackMazeDot,
        Room.BlackMazeEntrance,
        Room.BlackMaze13,
    ),
    Room.BlackMazeDot: (
        Room.BlackMaze13,
        Room.BlackMazeEntrance,
        Room.BlackMaze13,
        Room.BlackMaze14,
    ),
    Room.BlackMazeEntrance: (
        Room.BlackMaze14,
        Room.BlackMaze13,
        Room.BlackCastleInterior,
        Room.BlackMazeDot,
    ),
    Room.RedMaze17: (Room.RedMazeSouth, Room.RedMazeNorth, Room.RedMazeSouth, Room.RedMazeNorth),
    Room.RedMazeNorth: (
        Room.WhiteCastleInterior,
        Room.RedMaze17,
        Room.WhiteCastleInterior,
        Room.RedMaze17,
    ),
    Room.RedMazeSouth: (
        Room.RedMaze17,
        Room.WhiteCastleInterior,
        Room.RedMaze17,
        Room.WhiteCastleInterior,
    ),
    Room.WhiteCastleInterior: (
        Room.RedMazeNorth,
        Room.RedMazeSouth,
        Room.RedMazeNorth,
        Room.RedMazeSouth,
    ),
    Room.BlackCastleInterior: (
        Room.PurpleChamber,
        Room.PurpleChamber,
        Room.PurpleChamber,
        Room.PurpleChamber,
    ),
    Room.PurpleChamber: (
        Room.RedHall,
        Room.BlueMazeCenter,
        Room.BlackCastleInterior,
        Room.BlueMazeEntrance,
    ),
    Room.RedHall: (Room.SecretPassage, Room.WestHall, Room.BlackCastle, Room.SecretPassage),
    Room.RobinettCredits: (
        Room.BlueMazeSouth,
        Room.WestHall,
        Room.BlueMazeSouth,
        Room.SecretPassage,
    ),
}


def room_exit(room: Room, edge: Edge, game: Game) -> Room:
    if game != Game.One:
        special = {
            (Room.WestHall, Edge.South): Room.WhiteCastle,
            (Room.BelowGoldCastle, Edge.South): Room.GoldCastle,
            (Room.SecretPassage, Edge.South): Room.CatacombsEntrance,
            (Room.PurpleChamber, Edge.South): Room.SidePassage0c,
            (Room.RedHall, Edge.North): Room.SidePassage0c,
        }
        if room == Room.BlackCastleInterior:
            return Room.BlackMazeEntrance
        if (room, edge) in special:
            return special[room, edge]
    return Room(EXITS[room][edge])


# Several rooms share artwork while retaining different colors and exits.
WALLS = {
    Room.GameSelection: art.WALLS_ENCLOSED_ROOM,
    Room.WestHall: art.WALLS_OPEN_HALL,
    Room.BelowGoldCastle: art.WALLS_OPEN_HALL,
    Room.SecretPassage: art.WALLS_SECRET_PASSAGE,
    Room.BlueMazeNorth: art.WALLS_BLUE_MAZE_NORTH,
    Room.BlueMaze05: art.WALLS_BLUE_MAZE_05,
    Room.BlueMazeSouth: art.WALLS_BLUE_MAZE_SOUTH,
    Room.BlueMazeCenter: art.WALLS_BLUE_MAZE_CENTER,
    Room.BlueMazeEntrance: art.WALLS_BLUE_MAZE_ENTRANCE,
    Room.CatacombsCenter: art.WALLS_CATACOMBS_CENTER,
    Room.CatacombsEntrance: art.WALLS_CATACOMBS_ENTRANCE,
    Room.CatacombsSide: art.WALLS_CATACOMBS_SIDE,
    Room.SidePassage0c: art.WALLS_SIDE_PASSAGE,
    Room.SidePassage0d: art.WALLS_SIDE_PASSAGE,
    Room.AboveWhiteCastle: art.WALLS_TOP_OPENING,
    Room.WhiteCastle: art.WALLS_CASTLE,
    Room.BlackCastle: art.WALLS_CASTLE,
    Room.GoldCastle: art.WALLS_CASTLE,
    Room.GoldTreasury: art.WALLS_ENCLOSED_ROOM,
    Room.BlackMaze13: art.WALLS_BLACK_MAZE_13,
    Room.BlackMaze14: art.WALLS_BLACK_MAZE_14,
    Room.BlackMazeDot: art.WALLS_BLACK_MAZE_DOT,
    Room.BlackMazeEntrance: art.WALLS_BLACK_MAZE_ENTRANCE,
    Room.RedMaze17: art.WALLS_RED_MAZE_17,
    Room.RedMazeNorth: art.WALLS_RED_MAZE_NORTH,
    Room.RedMazeSouth: art.WALLS_RED_MAZE_SOUTH,
    Room.WhiteCastleInterior: art.WALLS_WHITE_CASTLE_INTERIOR,
    Room.BlackCastleInterior: art.WALLS_BLACK_CASTLE_INTERIOR,
    Room.PurpleChamber: art.WALLS_ENCLOSED_ROOM,
    Room.RedHall: art.WALLS_TOP_OPENING,
    Room.RobinettCredits: art.WALLS_OPEN_HALL,
}

ROOM_COLORS = {
    Room.GameSelection: 0x66,
    Room.WestHall: 0xD8,
    Room.BelowGoldCastle: 0xC8,
    Room.SecretPassage: 0xE8,
    Room.BlueMazeNorth: 0x86,
    Room.BlueMaze05: 0x86,
    Room.BlueMazeSouth: 0x86,
    Room.BlueMazeCenter: 0x86,
    Room.BlueMazeEntrance: 0x86,
    Room.CatacombsCenter: 0x08,
    Room.CatacombsEntrance: 0x08,
    Room.CatacombsSide: 0x08,
    Room.SidePassage0c: 0x98,
    Room.SidePassage0d: 0xB8,
    Room.AboveWhiteCastle: 0xA8,
    Room.WhiteCastle: 0x0C,
    Room.BlackCastle: 0,
    Room.GoldCastle: 0x1A,
    Room.GoldTreasury: 0x1A,
    Room.BlackMaze13: 0x08,
    Room.BlackMaze14: 0x08,
    Room.BlackMazeDot: 0x08,
    Room.BlackMazeEntrance: 0x08,
    Room.RedMaze17: 0x36,
    Room.RedMazeNorth: 0x36,
    Room.RedMazeSouth: 0x36,
    Room.WhiteCastleInterior: 0x36,
    Room.BlackCastleInterior: 0x36,
    Room.PurpleChamber: 0x66,
    Room.RedHall: 0x36,
    Room.RobinettCredits: 0x66,
}
