//! Optional trace observations and recorded input decoding for validation.

use adventure_trace::{
    ContactTarget, Control, Counter, Game as TraceGame, Mode, Object, Observation, Point, State,
    Target,
};

use super::Direction;
use super::{
    Adventure, DragonState, Game as PortGame, GateState, ObjectId, Position, RecordId, Room,
    Surface,
};

fn game(value: PortGame) -> TraceGame {
    match value {
        PortGame::One => TraceGame::One,
        PortGame::Two => TraceGame::Two,
        PortGame::Three => TraceGame::Three,
    }
}

fn direction(value: Direction) -> u8 {
    u8::from(value.right)
        | u8::from(value.left) << 1
        | u8::from(value.up) << 2
        | u8::from(value.down) << 3
}
impl Adventure {
    pub(super) fn trace_initialize(&self) {
        if let Some(trace) = &self.trace {
            trace.borrow_mut().initialize(game(self.world.game));
        }
    }
    // Attachment is observed as a transition only when the bat crossed a room boundary this operation.
    pub(super) fn trace_cargo_attachment(&self, cargo: RecordId) {
        if self
            .trace
            .as_ref()
            .is_some_and(|trace| trace.borrow().crossed(Object::Bat))
        {
            self.trace_transition(cargo);
        }
    }
    pub(super) fn trace_transition(&self, id: RecordId) {
        self.trace_observe(Observation::Transition(
            record(id),
            self.world.position(id).into(),
        ));
    }

    pub(super) fn trace_object_contact(&self) {
        if self.contacts.objects_touch
            && let (Some(first), Some(second)) = (
                self.display.visible[0].map(object),
                self.display.visible[1].map(object),
            )
        {
            self.trace_observe(Observation::Contact(first, ContactTarget::Object(second)));
        }
    }
    pub(super) fn trace_at(&self, point: Point) {
        if let Some(trace) = &self.trace {
            trace.borrow_mut().at(point);
        }
    }
    pub(super) fn trace_observe(&self, observation: Observation) {
        if let Some(trace) = &self.trace {
            trace.borrow_mut().observe(observation);
        }
    }
    // Ordinary play returns live input unchanged. Playback supplies input without changing world state.
    pub(super) fn input_value(&self, control: Control, live: u8) -> u8 {
        let live = self.recording.as_ref().map_or(live, |recording| {
            recording.borrow_mut().input(control, live)
        });
        self.trace
            .as_ref()
            .map_or(live, |trace| trace.borrow_mut().input(control, live))
    }

    // Each checkpoint observes complete records, including those outside the displayed room.
    pub(super) fn trace_checkpoint(&self) {
        if self.trace.is_none() {
            return;
        }
        let r = &self.world;
        for (object, position) in [
            (Object::Player, r.player),
            (Object::SafePlayer, r.safe_player),
            (Object::Light, r.light),
            (Object::Dot, r.dot),
            (Object::Rhindle, r.rhindle.position),
            (Object::Yorgle, r.yorgle.position),
            (Object::Grundle, r.grundle.position),
            (Object::Magnet, r.magnet),
            (Object::Sword, r.sword),
            (Object::Chalice, r.chalice),
            (Object::Bridge, r.bridge),
            (Object::GoldKey, r.gold_key),
            (Object::WhiteKey, r.white_key),
            (Object::BlackKey, r.black_key),
            (Object::Bat, r.bat.position),
        ] {
            self.trace_observe(Observation::Position(object, position.into()));
        }
        for (object, dragon) in [
            (Object::Rhindle, r.rhindle),
            (Object::Yorgle, r.yorgle),
            (Object::Grundle, r.grundle),
        ] {
            let mode = match dragon.state {
                DragonState::Wandering => Mode::Wandering,
                DragonState::Dead => Mode::Dead,
                DragonState::Swallowed => Mode::Swallowed,
                DragonState::Biting(_) => Mode::Biting,
            };
            self.trace_observe(Observation::State(object, State::Mode(mode)));
            self.trace_observe(Observation::State(
                object,
                State::Direction(direction(dragon.direction)),
            ));
            self.trace_observe(Observation::State(
                object,
                State::Counter(
                    Counter::Bite,
                    match dragon.state {
                        DragonState::Biting(v) => v,
                        _ => 0,
                    },
                ),
            ));
        }
        for (object, gate) in [
            (Object::GoldGate, r.gates.gold),
            (Object::WhiteGate, r.gates.white),
            (Object::BlackGate, r.gates.black),
        ] {
            let mode = match gate {
                GateState::Open => Mode::Open,
                GateState::Closed => Mode::Closed,
                GateState::Moving(v) if v < 28 => Mode::Closing,
                GateState::Moving(_) => Mode::Opening,
            };
            self.trace_observe(Observation::State(object, State::Mode(mode)));
            self.trace_observe(Observation::State(
                object,
                State::Counter(
                    Counter::Gate,
                    match gate {
                        GateState::Moving(v) => v,
                        _ => 0,
                    },
                ),
            ));
        }
        self.trace_observe(Observation::State(
            Object::Player,
            State::Carry {
                item: r.carrying.item.object().map(object),
                x: r.carrying.offset.x,
                y: r.carrying.offset.y,
            },
        ));
        self.trace_observe(Observation::State(
            Object::Bat,
            State::Carry {
                item: r.bat.carried.map(|cargo| record(cargo.record())),
                x: 8,
                y: 0,
            },
        ));
        self.trace_observe(Observation::State(
            Object::Bat,
            State::Direction(direction(r.bat.direction)),
        ));
        self.trace_observe(Observation::State(
            Object::Bat,
            State::Counter(Counter::Animation, r.bat.animation),
        ));
        self.trace_observe(Observation::State(
            Object::Bat,
            State::Counter(Counter::Flight, r.bat.flight_timer),
        ));
        self.trace_observe(Observation::State(Object::World, State::Game(game(r.game))));
        self.trace_observe(Observation::State(Object::World, State::Won(r.won)));
        self.trace_observe(Observation::State(
            Object::World,
            State::Counter(Counter::IdleTicks, r.idle_ticks),
        ));
        self.trace_observe(Observation::State(
            Object::World,
            State::Counter(Counter::IdlePeriods, r.idle_periods),
        ));
    }

    pub(super) fn trace_contact(&self, target: ContactTarget, positive: bool) {
        if positive {
            self.trace_observe(Observation::Contact(Object::Player, target));
        }
    }

    pub(super) fn trace_surface_contact(&self, surface: Surface, positive: bool) {
        let target = match surface {
            Surface::Walls => Target::Walls,
            Surface::LeftBarrier => Target::LeftBarrier,
            Surface::RightBarrier => Target::RightBarrier,
        };
        self.trace_contact(ContactTarget::Surface(target), positive);
    }

    pub(super) fn trace_player_contact(&self, slot: usize, positive: bool) {
        if let Some(id) = self.display.visible[slot] {
            self.trace_contact(ContactTarget::Object(object(id)), positive);
        }
    }
}

// Identity conversion is explicit so source enum layouts need not agree.
fn room(value: Room) -> adventure_trace::Room {
    match value {
        Room::GameSelection => adventure_trace::Room::GameSelection,
        Room::WestHall => adventure_trace::Room::WestHall,
        Room::BelowGoldCastle => adventure_trace::Room::BelowGoldCastle,
        Room::SecretPassage => adventure_trace::Room::SecretPassage,
        Room::BlueMazeNorth => adventure_trace::Room::BlueMazeNorth,
        Room::BlueMaze05 => adventure_trace::Room::BlueMaze05,
        Room::BlueMazeSouth => adventure_trace::Room::BlueMazeSouth,
        Room::BlueMazeCenter => adventure_trace::Room::BlueMazeCenter,
        Room::BlueMazeEntrance => adventure_trace::Room::BlueMazeEntrance,
        Room::CatacombsCenter => adventure_trace::Room::CatacombsCenter,
        Room::CatacombsEntrance => adventure_trace::Room::CatacombsEntrance,
        Room::CatacombsSide => adventure_trace::Room::CatacombsSide,
        Room::SidePassage0c => adventure_trace::Room::SidePassage0c,
        Room::SidePassage0d => adventure_trace::Room::SidePassage0d,
        Room::AboveWhiteCastle => adventure_trace::Room::AboveWhiteCastle,
        Room::WhiteCastle => adventure_trace::Room::WhiteCastle,
        Room::BlackCastle => adventure_trace::Room::BlackCastle,
        Room::GoldCastle => adventure_trace::Room::GoldCastle,
        Room::GoldTreasury => adventure_trace::Room::GoldTreasury,
        Room::BlackMaze13 => adventure_trace::Room::BlackMaze13,
        Room::BlackMaze14 => adventure_trace::Room::BlackMaze14,
        Room::BlackMazeDot => adventure_trace::Room::BlackMazeDot,
        Room::BlackMazeEntrance => adventure_trace::Room::BlackMazeEntrance,
        Room::RedMaze17 => adventure_trace::Room::RedMaze17,
        Room::RedMazeNorth => adventure_trace::Room::RedMazeNorth,
        Room::RedMazeSouth => adventure_trace::Room::RedMazeSouth,
        Room::WhiteCastleInterior => adventure_trace::Room::WhiteCastleInterior,
        Room::BlackCastleInterior => adventure_trace::Room::BlackCastleInterior,
        Room::PurpleChamber => adventure_trace::Room::PurpleChamber,
        Room::RedHall => adventure_trace::Room::RedHall,
        Room::RobinettCredits => adventure_trace::Room::RobinettCredits,
    }
}
fn record(value: RecordId) -> Object {
    match value {
        RecordId::Player => Object::Player,
        RecordId::Light => Object::Light,
        RecordId::Dot => Object::Dot,
        RecordId::Rhindle => Object::Rhindle,
        RecordId::Yorgle => Object::Yorgle,
        RecordId::Grundle => Object::Grundle,
        RecordId::Magnet => Object::Magnet,
        RecordId::Sword => Object::Sword,
        RecordId::Chalice => Object::Chalice,
        RecordId::Bridge => Object::Bridge,
        RecordId::GoldKey => Object::GoldKey,
        RecordId::WhiteKey => Object::WhiteKey,
        RecordId::BlackKey => Object::BlackKey,
        RecordId::Bat => Object::Bat,
    }
}
fn object(value: ObjectId) -> Object {
    match value {
        ObjectId::Light => Object::Light,
        ObjectId::GoldGate => Object::GoldGate,
        ObjectId::WhiteGate => Object::WhiteGate,
        ObjectId::BlackGate => Object::BlackGate,
        ObjectId::Credits => Object::Credits,
        ObjectId::GameNumber => Object::GameNumber,
        ObjectId::Rhindle => Object::Rhindle,
        ObjectId::Yorgle => Object::Yorgle,
        ObjectId::Grundle => Object::Grundle,
        ObjectId::Sword => Object::Sword,
        ObjectId::Bridge => Object::Bridge,
        ObjectId::GoldKey => Object::GoldKey,
        ObjectId::WhiteKey => Object::WhiteKey,
        ObjectId::BlackKey => Object::BlackKey,
        ObjectId::Bat => Object::Bat,
        ObjectId::Dot => Object::Dot,
        ObjectId::Chalice => Object::Chalice,
        ObjectId::Magnet => Object::Magnet,
    }
}

impl From<Position> for adventure_trace::Position {
    fn from(value: Position) -> Self {
        Self {
            room: room(value.room),
            x: value.x,
            y: value.y,
        }
    }
}
