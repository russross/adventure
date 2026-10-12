//! Adapter for the two ports' existing, equivalent typed world models.

// Keep the common read and observation contract in one place while each port owns its game state.
#[macro_export]
macro_rules! port_adapter {
    ($world:ident, $raster:ident) => {
        mod trace_adapter {

            use $crate::{
                ContactTarget, Control, Counter, Game as TraceGame, Mode, Object, Observation,
                Point, State, Target,
            };

            use super::{
                Adventure, DragonState, Game as PortGame, GateState, ObjectId, Position, RecordId,
                Room,
            };
            use super::{Difficulty, Direction};

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
            fn decode_direction(value: u8) -> Direction {
                Direction {
                    right: value & 1 != 0,
                    left: value & 2 != 0,
                    up: value & 4 != 0,
                    down: value & 8 != 0,
                }
            }

            impl Adventure {
                pub(super) fn trace_initialize(&self) {
                    if let Some(trace) = &self.trace {
                        trace.borrow_mut().initialize(game(self.$world.game));
                    }
                }
                pub(super) fn trace_crossed(&self, id: RecordId) -> bool {
                    self.trace
                        .as_ref()
                        .is_some_and(|trace| trace.borrow().crossed(record(id)))
                }
                pub(super) fn trace_transition(&self, id: RecordId) {
                    self.trace_observe(Observation::Transition(
                        record(id),
                        self.$world.position(id).into(),
                    ));
                }

                pub(super) fn trace_object_contact(&self) {
                    if self.$raster.contacts.objects_touch
                        && let (Some(first), Some(second)) = (
                            object(self.$world.first_visible),
                            object(self.$world.second_visible),
                        )
                    {
                        self.trace_observe(Observation::Contact(
                            first,
                            ContactTarget::Object(second),
                        ));
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
                pub(super) fn trace_input(&mut self, control: Control) {
                    let Some(trace) = &self.trace else {
                        return;
                    };
                    let live = match control {
                        Control::Reset => u8::from(self.controls.reset),
                        Control::Select => u8::from(self.controls.select),
                        Control::Color => u8::from(self.controls.color),
                        Control::LeftDirection => direction(self.controls.left.direction),
                        Control::RightDirection => direction(self.controls.right.direction),
                        Control::Fire => u8::from(self.controls.left.fire),
                        Control::LeftDifficulty => {
                            u8::from(self.controls.left_difficulty == Difficulty::A)
                        }
                        Control::RightDifficulty => {
                            u8::from(self.controls.right_difficulty == Difficulty::A)
                        }
                    };
                    let value = trace.borrow_mut().input(control, live);
                    match control {
                        Control::Reset => self.controls.reset = value != 0,
                        Control::Select => self.controls.select = value != 0,
                        Control::Color => self.controls.color = value != 0,
                        Control::LeftDirection => {
                            self.controls.left.direction = decode_direction(value)
                        }
                        Control::RightDirection => {
                            self.controls.right.direction = decode_direction(value)
                        }
                        Control::Fire => self.controls.left.fire = value != 0,
                        Control::LeftDifficulty => {
                            self.controls.left_difficulty = if value != 0 {
                                Difficulty::A
                            } else {
                                Difficulty::B
                            }
                        }
                        Control::RightDifficulty => {
                            self.controls.right_difficulty = if value != 0 {
                                Difficulty::A
                            } else {
                                Difficulty::B
                            }
                        }
                    }
                }

                // Each checkpoint observes complete records, including those outside the displayed room.
                pub(super) fn trace_checkpoint(&self) {
                    if self.trace.is_none() {
                        return;
                    }
                    let r = &self.$world;
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
                            item: object(r.held),
                            x: r.held_offset_x,
                            y: r.held_offset_y,
                        },
                    ));
                    self.trace_observe(Observation::State(
                        Object::Bat,
                        State::Carry {
                            item: r.bat.carried.map(record),
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
                    self.trace_observe(Observation::State(
                        Object::World,
                        State::Game(game(r.game)),
                    ));
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

                pub(super) fn trace_contact(&self, target: ContactTarget, positive: bool) -> bool {
                    if positive {
                        self.trace_observe(Observation::Contact(Object::Player, target));
                    }
                    positive
                }
                pub(super) fn trace_player_slot(&self, slot: usize) -> bool {
                    let id = if slot == 0 {
                        self.$world.first_visible
                    } else {
                        self.$world.second_visible
                    };
                    let positive = self.$raster.contacts.player_objects[slot];
                    if let Some(object) = object(id) {
                        self.trace_contact(ContactTarget::Object(object), positive);
                    }
                    positive
                }
                pub(super) fn trace_blocked(&self) -> bool {
                    self.trace_contact(
                        ContactTarget::Surface(Target::Walls),
                        self.$raster.contacts.player_walls,
                    ) || self.trace_contact(
                        ContactTarget::Surface(Target::LeftBarrier),
                        self.$raster.contacts.player_barriers[0],
                    ) || self.trace_contact(
                        ContactTarget::Surface(Target::RightBarrier),
                        self.$raster.contacts.player_barriers[1],
                    ) && self.$world.second_visible != ObjectId::Dot
                        || self.trace_player_slot(0) && self.$world.first_visible != ObjectId::Light
                        || self.trace_player_slot(1)
                            && self.$world.second_visible != ObjectId::Light
                }
            }

            // Identity conversion is explicit so source enum layouts need not agree.
            fn room(value: Room) -> $crate::Room {
                match value {
                    Room::GameSelection => $crate::Room::GameSelection,
                    Room::WestHall => $crate::Room::WestHall,
                    Room::BelowGoldCastle => $crate::Room::BelowGoldCastle,
                    Room::SecretPassage => $crate::Room::SecretPassage,
                    Room::BlueMazeNorth => $crate::Room::BlueMazeNorth,
                    Room::BlueMaze05 => $crate::Room::BlueMaze05,
                    Room::BlueMazeSouth => $crate::Room::BlueMazeSouth,
                    Room::BlueMazeCenter => $crate::Room::BlueMazeCenter,
                    Room::BlueMazeEntrance => $crate::Room::BlueMazeEntrance,
                    Room::CatacombsCenter => $crate::Room::CatacombsCenter,
                    Room::CatacombsEntrance => $crate::Room::CatacombsEntrance,
                    Room::CatacombsSide => $crate::Room::CatacombsSide,
                    Room::SidePassage0c => $crate::Room::SidePassage0c,
                    Room::SidePassage0d => $crate::Room::SidePassage0d,
                    Room::AboveWhiteCastle => $crate::Room::AboveWhiteCastle,
                    Room::WhiteCastle => $crate::Room::WhiteCastle,
                    Room::BlackCastle => $crate::Room::BlackCastle,
                    Room::GoldCastle => $crate::Room::GoldCastle,
                    Room::GoldTreasury => $crate::Room::GoldTreasury,
                    Room::BlackMaze13 => $crate::Room::BlackMaze13,
                    Room::BlackMaze14 => $crate::Room::BlackMaze14,
                    Room::BlackMazeDot => $crate::Room::BlackMazeDot,
                    Room::BlackMazeEntrance => $crate::Room::BlackMazeEntrance,
                    Room::RedMaze17 => $crate::Room::RedMaze17,
                    Room::RedMazeNorth => $crate::Room::RedMazeNorth,
                    Room::RedMazeSouth => $crate::Room::RedMazeSouth,
                    Room::WhiteCastleInterior => $crate::Room::WhiteCastleInterior,
                    Room::BlackCastleInterior => $crate::Room::BlackCastleInterior,
                    Room::PurpleChamber => $crate::Room::PurpleChamber,
                    Room::RedHall => $crate::Room::RedHall,
                    Room::RobinettCredits => $crate::Room::RobinettCredits,
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
            fn object(value: ObjectId) -> Option<Object> {
                match value {
                    ObjectId::Light => Some(Object::Light),
                    ObjectId::GoldGate => Some(Object::GoldGate),
                    ObjectId::WhiteGate => Some(Object::WhiteGate),
                    ObjectId::BlackGate => Some(Object::BlackGate),
                    ObjectId::Credits => Some(Object::Credits),
                    ObjectId::GameNumber => Some(Object::GameNumber),
                    ObjectId::Rhindle => Some(Object::Rhindle),
                    ObjectId::Yorgle => Some(Object::Yorgle),
                    ObjectId::Grundle => Some(Object::Grundle),
                    ObjectId::Sword => Some(Object::Sword),
                    ObjectId::Bridge => Some(Object::Bridge),
                    ObjectId::GoldKey => Some(Object::GoldKey),
                    ObjectId::WhiteKey => Some(Object::WhiteKey),
                    ObjectId::BlackKey => Some(Object::BlackKey),
                    ObjectId::Bat => Some(Object::Bat),
                    ObjectId::Dot => Some(Object::Dot),
                    ObjectId::Chalice => Some(Object::Chalice),
                    ObjectId::Magnet => Some(Object::Magnet),
                    ObjectId::Empty => None,
                }
            }

            impl From<Position> for $crate::Position {
                fn from(value: Position) -> Self {
                    Self {
                        room: room(value.room),
                        x: value.x,
                        y: value.y,
                    }
                }
            }
        }
    };
}
