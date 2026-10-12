use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::io::Write;

use crate::{
    ContactTarget, Control, Counter, Event, Game, Mode, Moment, Object, Observation, Point,
    Position, Record, State,
};

#[derive(Clone, Copy)]
struct Moving {
    position: Position,
    frame: u64,
    point: Point,
    velocity: Option<(i16, i16, u64)>,
}

#[derive(Clone, Copy)]
struct StateTrack {
    state: State,
    frame: u64,
    point: Point,
    period: Option<u64>,
}

// Position samples at the owning operation verify continued motion; unrelated unchanged samples are redundant.
#[derive(Default)]
pub struct Machine {
    positions: BTreeMap<Object, Moving>,
    states: BTreeMap<(Object, u8), StateTrack>,
    contact_frame: u64,
    gate_keys: BTreeSet<Object>,
}
impl Machine {
    pub fn observe(&mut self, frame: u64, point: Point, observation: Observation) -> Option<Event> {
        if frame != self.contact_frame {
            self.contact_frame = frame;
            self.gate_keys.clear();
        }
        match observation {
            Observation::Position(object, position) | Observation::Transition(object, position) => {
                let explicit = matches!(observation, Observation::Transition(..));
                let Some(old) = self.positions.get(&object).copied() else {
                    self.positions.insert(
                        object,
                        Moving {
                            position,
                            frame,
                            point,
                            velocity: None,
                        },
                    );
                    return Some(Event::Place(object, position));
                };
                if !explicit
                    && old.position == position
                    && (old.point != point || old.frame == frame)
                {
                    return None;
                }
                let elapsed = frame.saturating_sub(old.frame);
                let dx = i16::from(position.x.wrapping_sub(old.position.x) as i8);
                let dy = i16::from(position.y.wrapping_sub(old.position.y) as i8);
                let crossing = explicit || old.position.room != position.room;
                let velocity = if crossing && old.point == point {
                    old.velocity
                } else if elapsed > 0 && (dx != 0 || dy != 0) && old.point == point {
                    Some((dx, dy, elapsed))
                } else {
                    None
                };
                self.positions.insert(
                    object,
                    Moving {
                        position,
                        frame,
                        point,
                        velocity,
                    },
                );
                if crossing {
                    return Some(Event::Enter(object, position));
                }
                if old.position == position {
                    return old.velocity.map(|_| Event::Stop(object, position));
                }
                match velocity {
                    Some((dx, dy, period)) if old.velocity != velocity => Some(Event::Motion {
                        object,
                        position,
                        dx,
                        dy,
                        period,
                    }),
                    Some(_) => None,
                    None => Some(Event::Place(object, position)),
                }
            }
            Observation::State(object, state) => {
                let key = (object, state.key());
                let old = self.states.get(&key).copied();
                let mut cadence = None;
                // Terminal modes determine their inactive progress counters without another text record.
                let mode = self
                    .states
                    .get(&(object, State::Mode(Mode::Open).key()))
                    .map(|s| s.state);
                let inferred_zero = match state {
                    State::Counter(Counter::Gate, 0) => {
                        matches!(mode, Some(State::Mode(Mode::Open | Mode::Closed)))
                    }
                    State::Counter(Counter::Bite, 0) => matches!(
                        mode,
                        Some(State::Mode(Mode::Wandering | Mode::Dead | Mode::Swallowed))
                    ),
                    _ => false,
                };
                if inferred_zero {
                    self.states.insert(
                        key,
                        StateTrack {
                            state,
                            frame,
                            point,
                            period: None,
                        },
                    );
                    return None;
                }
                if let Some(StateTrack {
                    state: before,
                    frame: previous_frame,
                    point: owner,
                    period,
                }) = old
                {
                    if before == state && owner != point {
                        return None;
                    }
                    if let (State::Counter(c, value), State::Counter(d, after)) = (before, state) {
                        let elapsed = frame.saturating_sub(previous_frame);
                        let active = match c {
                            Counter::Flight | Counter::Bite | Counter::Gate => value != 0,
                            Counter::IdlePeriods => false,
                            _ => true,
                        };
                        let expected = if c == Counter::Animation {
                            value.wrapping_add(1) % 8
                        } else {
                            value.wrapping_add(
                                1 + u8::from(
                                    c == Counter::Gate && self.gate_keys.contains(&object),
                                ),
                            )
                        };
                        if c == d && owner == point && elapsed > 0 && active {
                            cadence = Some(elapsed);
                            self.states.insert(
                                key,
                                StateTrack {
                                    state,
                                    frame,
                                    point,
                                    period: cadence,
                                },
                            );
                            if period == cadence
                                && expected == after
                                && (after != 0 || c == Counter::Animation)
                            {
                                return None;
                            }
                            return Some(Event::State(object, state));
                        }
                    }
                    if before == state {
                        return None;
                    }
                }
                self.states.insert(
                    key,
                    StateTrack {
                        state,
                        frame,
                        point,
                        period: cadence,
                    },
                );
                Some(Event::State(object, state))
            }
            Observation::Contact(object, target) => {
                if point == Point::Gates
                    && let ContactTarget::Object(other) = target
                {
                    for (gate, key) in [
                        (Object::GoldGate, Object::GoldKey),
                        (Object::WhiteGate, Object::WhiteKey),
                        (Object::BlackGate, Object::BlackKey),
                    ] {
                        if (object == gate && other == key) || (object == key && other == gate) {
                            self.gate_keys.insert(gate);
                        }
                    }
                }
                Some(Event::Contact(object, target))
            }
        }
    }
}

// A session uses the same normalizer for recording and replay, stopping at the first divergence.
#[derive(Default)]
pub struct Session {
    machine: Machine,
    pub frame: u64,
    pub point: Option<Point>,
    ordinals: BTreeMap<Point, u32>,
    records: Vec<Record>,
    expected: Option<VecDeque<Record>>,
    error: Option<String>,
    writer: Option<Box<dyn Write>>,
    finished: bool,
    transitions: BTreeSet<(Point, Object)>,
}
impl Session {
    pub fn record() -> Self {
        let mut s = Self::default();
        s.emit(Point::Boot, Event::Begin);
        s
    }
    pub fn record_to(writer: impl Write + 'static) -> Result<Self, String> {
        let mut s = Self {
            writer: Some(Box::new(writer)),
            ..Self::default()
        };
        s.emit(Point::Boot, Event::Begin);
        s.check()?;
        Ok(s)
    }
    pub fn replay(records: Vec<Record>) -> Result<Self, String> {
        if records.first().map(|r| &r.event) != Some(&Event::Begin)
            || records.last().map(|r| &r.event) != Some(&Event::End)
        {
            return Err("trace needs boot header and explicit end".into());
        }
        let mut previous_frame = 0;
        let mut ordinals = BTreeMap::new();
        for (index, record) in records.iter().enumerate() {
            if record.at.frame < previous_frame {
                return Err("trace clock runs backward".into());
            }
            if record.at.frame != previous_frame {
                ordinals.clear();
            }
            let ordinal = ordinals.entry(record.at.point).or_insert(0);
            if record.at.ordinal != *ordinal {
                return Err(format!("invalid event ordinal at {}", record.at));
            }
            *ordinal += 1;
            previous_frame = record.at.frame;
            if matches!(record.event, Event::Begin) && index != 0
                || matches!(record.event, Event::End) && index + 1 != records.len()
            {
                return Err("misplaced boot or end record".into());
            }
        }
        let mut s = Self {
            expected: Some(records.into()),
            ..Self::default()
        };
        s.emit(Point::Boot, Event::Begin);
        s.check()?;
        Ok(s)
    }
    pub fn next_frame(&mut self) {
        self.frame += 1;
        self.ordinals.clear();
        self.transitions.clear();
    }
    pub fn at(&mut self, point: Point) {
        self.point = Some(point);
    }
    pub fn initialize(&mut self, game: Game) {
        self.at(Point::Init);
        self.emit(Point::Init, Event::Init(game));
    }
    pub fn observe(&mut self, observation: Observation) {
        let point = self.point.unwrap_or(Point::Boot);
        if let Observation::Transition(object, _) = observation {
            self.transitions.insert((point, object));
        }
        if let Some(event) = self.machine.observe(self.frame, point, observation) {
            self.emit(point, event);
        }
    }
    pub fn crossed(&self, object: Object) -> bool {
        self.transitions
            .contains(&(self.point.unwrap_or(Point::Boot), object))
    }
    pub fn input(&mut self, control: Control, live: u8) -> u8 {
        let point = self.point.unwrap_or(Point::Boot);
        let value = match self.expected.as_ref().and_then(|v| v.front()) {
            Some(Record {
                event: Event::Input(c, v),
                ..
            }) if *c == control => *v,
            _ => live,
        };
        self.emit(point, Event::Input(control, value));
        value
    }
    fn emit(&mut self, point: Point, event: Event) {
        if self.error.is_some() {
            return;
        }
        if self.finished {
            self.error = Some("event after trace end".into());
            return;
        }
        let ordinal = self.ordinals.entry(point).or_insert(0);
        let record = Record {
            at: Moment {
                frame: self.frame,
                point,
                ordinal: *ordinal,
            },
            event,
        };
        *ordinal += 1;
        if let Some(expected) = &mut self.expected {
            if expected.front() != Some(&record) {
                self.error = Some(format!(
                    "trace divergence: expected {}, observed {record}",
                    expected
                        .front()
                        .map(ToString::to_string)
                        .unwrap_or_else(|| "end of stream".into())
                ));
                return;
            }
            expected.pop_front();
        } else if let Some(writer) = &mut self.writer {
            if let Err(error) = writeln!(writer, "{record}") {
                self.error = Some(format!("cannot write trace: {error}"));
            }
        } else {
            self.records.push(record);
        }
    }
    pub fn check(&self) -> Result<(), String> {
        self.error.clone().map_or(Ok(()), Err)
    }
    pub fn finish(&mut self) -> Result<(), String> {
        self.emit(Point::End, Event::End);
        self.finished = true;
        self.check()?;
        if let Some(writer) = &mut self.writer {
            writer
                .flush()
                .map_err(|e| format!("cannot flush trace: {e}"))?;
        }
        if self.expected.as_ref().is_some_and(|v| !v.is_empty()) {
            return Err("unconsumed trace records".into());
        }
        Ok(())
    }
    pub fn records(&self) -> &[Record] {
        &self.records
    }
    pub fn stopped_at_boot(&self) -> bool {
        self.expected
            .as_ref()
            .and_then(|records| records.front())
            .is_some_and(|record| record.at.frame == 0 && matches!(record.event, Event::End))
    }
}
