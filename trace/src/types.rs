use std::fmt;
use std::str::FromStr;

// Closed names keep text parsing and adapters on the same vocabulary.
macro_rules! names {
    ($name:ident { $($variant:ident => $text:literal),+ $(,)? }) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum $name { $($variant),+ }
        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(match self { $(Self::$variant => $text),+ })
            }
        }
        impl FromStr for $name {
            type Err = String;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s { $($text => Ok(Self::$variant),)+ _ => Err(format!("unknown {} {s}", stringify!($name))) }
            }
        }
    };
}

names!(Object {
    World => "world", Player => "player", SafePlayer => "safe-player",
    Light => "light", GoldGate => "gold-gate", WhiteGate => "white-gate",
    BlackGate => "black-gate", Credits => "credits", GameNumber => "game-number",
    Rhindle => "rhindle", Yorgle => "yorgle", Grundle => "grundle", Sword => "sword",
    Bridge => "bridge", GoldKey => "gold-key", WhiteKey => "white-key",
    BlackKey => "black-key", Bat => "bat", Dot => "dot", Chalice => "chalice", Magnet => "magnet"
});
names!(Room {
    GameSelection => "game-selection", WestHall => "west-hall",
    BelowGoldCastle => "below-gold-castle", SecretPassage => "secret-passage",
    BlueMazeNorth => "blue-maze-north", BlueMaze05 => "blue-maze-05",
    BlueMazeSouth => "blue-maze-south", BlueMazeCenter => "blue-maze-center",
    BlueMazeEntrance => "blue-maze-entrance", CatacombsCenter => "catacombs-center",
    CatacombsEntrance => "catacombs-entrance", CatacombsSide => "catacombs-side",
    SidePassage0c => "side-passage-0c", SidePassage0d => "side-passage-0d",
    AboveWhiteCastle => "above-white-castle", WhiteCastle => "white-castle",
    BlackCastle => "black-castle", GoldCastle => "gold-castle", GoldTreasury => "gold-treasury",
    BlackMaze13 => "black-maze-13", BlackMaze14 => "black-maze-14",
    BlackMazeDot => "black-maze-dot", BlackMazeEntrance => "black-maze-entrance",
    RedMaze17 => "red-maze-17", RedMazeNorth => "red-maze-north", RedMazeSouth => "red-maze-south",
    WhiteCastleInterior => "white-castle-interior", BlackCastleInterior => "black-castle-interior",
    PurpleChamber => "purple-chamber", RedHall => "red-hall", RobinettCredits => "robinett-credits"
});
names!(Point {
    Boot => "boot", Init => "init", Console => "console", History => "history",
    Effects => "effects", Inactivity => "inactivity", PlayerBoth => "player-both",
    PlayerVertical => "player-vertical", PlayerHorizontal => "player-horizontal",
    Held => "held", Display => "display", Pickup => "pickup", Bat => "bat", Gates => "gates",
    Grundle => "grundle", Yorgle => "yorgle", Rhindle => "rhindle", Magnet => "magnet",
    Light => "light", Frame => "frame", End => "end"
});
names!(Control {
    Reset => "reset", Select => "select", Color => "color", LeftDirection => "left-direction",
    RightDirection => "right-direction", Fire => "fire", LeftDifficulty => "left-difficulty",
    RightDifficulty => "right-difficulty"
});
names!(Game { One => "1", Two => "2", Three => "3" });
impl Control {
    pub fn validate(self, value: u8) -> Result<(), String> {
        let limit = match self {
            Self::LeftDirection | Self::RightDirection => 15,
            _ => 1,
        };
        if value > limit {
            return Err(format!("invalid {self} value {value}"));
        }
        Ok(())
    }
}

// An ordinal distinguishes repeated reads and committed events at the same point.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Moment {
    pub frame: u64,
    pub point: Point,
    pub ordinal: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Position {
    pub room: Room,
    pub x: u8,
    pub y: u8,
}

names!(Counter {
    Animation => "animation", Flight => "flight", Bite => "bite", Gate => "gate",
    IdleTicks => "idle-ticks", IdlePeriods => "idle-periods"
});
names!(Mode {
    Wandering => "wandering", Dead => "dead", Swallowed => "swallowed", Biting => "biting",
    Closed => "closed", Opening => "opening", Closing => "closing", Open => "open"
});
names!(Target { Walls => "walls", LeftBarrier => "left-barrier", RightBarrier => "right-barrier" });
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContactTarget {
    Object(Object),
    Surface(Target),
}

// States separate discrete transitions from counters whose expected increments can be omitted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Mode(Mode),
    Counter(Counter, u8),
    Carry { item: Option<Object>, x: u8, y: u8 },
    Direction(u8),
    Game(Game),
    Won(bool),
}
impl State {
    pub(crate) fn key(self) -> u8 {
        match self {
            Self::Mode(_) => 0,
            Self::Counter(c, _) => 1 + c as u8,
            Self::Carry { .. } => 7,
            Self::Direction(_) => 8,
            Self::Game(_) => 9,
            Self::Won(_) => 10,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    Begin,
    Init(Game),
    Input(Control, u8),
    Place(Object, Position),
    Enter(Object, Position),
    Motion {
        object: Object,
        position: Position,
        dx: i16,
        dy: i16,
        period: u64,
    },
    Stop(Object, Position),
    State(Object, State),
    Contact(Object, ContactTarget),
    End,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Record {
    pub at: Moment,
    pub event: Event,
}

// Raw positions are committed observations, never partial coordinate stores.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Observation {
    Position(Object, Position),
    Transition(Object, Position),
    State(Object, State),
    Contact(Object, ContactTarget),
}
