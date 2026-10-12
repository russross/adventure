use std::fmt;
use std::str::FromStr;

use crate::{ContactTarget, Event, Moment, Position, Record, State};

// The timestamp is nominal game time. Every record carries its semantic point and ordinal.
impl fmt::Display for Moment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:02}:{:02}:{:02}/{}#{}",
            self.frame / 3600,
            self.frame / 60 % 60,
            self.frame % 60,
            self.point,
            self.ordinal
        )
    }
}
impl FromStr for Moment {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        let (clock, rest) = s.split_once('/').ok_or("timestamp needs /point#ordinal")?;
        let (point, ordinal) = rest.split_once('#').ok_or("timestamp needs ordinal")?;
        let mut parts = clock.split(':');
        let minutes: u64 = number(parts.next().ok_or("missing minutes")?)?;
        let seconds: u64 = number(parts.next().ok_or("missing seconds")?)?;
        let frame: u64 = number(parts.next().ok_or("missing frame")?)?;
        if parts.next().is_some() || seconds >= 60 || frame >= 60 {
            return Err("invalid clock".into());
        }
        let frame = minutes
            .checked_mul(3600)
            .and_then(|v| v.checked_add(seconds * 60 + frame))
            .ok_or("clock overflow")?;
        Ok(Self {
            frame,
            point: point.parse()?,
            ordinal: number(ordinal)?,
        })
    }
}
fn number<T: FromStr>(s: &str) -> Result<T, String> {
    if s.is_empty() || !s.bytes().all(|c| c.is_ascii_digit()) {
        return Err(format!("invalid number {s}"));
    }
    s.parse().map_err(|_| format!("number out of range {s}"))
}
fn pair<T: FromStr>(s: &str) -> Result<(T, T), String> {
    let (x, y) = s.split_once(',').ok_or("expected x,y")?;
    Ok((
        x.parse().map_err(|_| "invalid x")?,
        y.parse().map_err(|_| "invalid y")?,
    ))
}
fn position(room: &str, xy: &str) -> Result<Position, String> {
    let (x, y) = pair(xy)?;
    Ok(Position {
        room: room.parse()?,
        x,
        y,
    })
}

// A fixed grammar rejects extra tokens rather than silently losing evidence.
impl FromStr for Record {
    type Err = String;
    fn from_str(line: &str) -> Result<Self, String> {
        let tokens: Vec<_> = line.split_whitespace().collect();
        let (clock, rest) = tokens.split_first().ok_or("empty record")?;
        let event = match rest {
            ["trace", "1", "adventure-original", "boot"] => Event::Begin,
            ["init", game @ ("1" | "2" | "3")] => Event::Init(game.parse()?),
            ["input", control, value] => {
                let control = control.parse::<crate::Control>()?;
                let value = number(value)?;
                control.validate(value)?;
                Event::Input(control, value)
            }
            [verb @ ("place" | "enter" | "stop"), object, room, xy] => {
                let object = object.parse()?;
                let p = position(room, xy)?;
                match *verb {
                    "place" => Event::Place(object, p),
                    "enter" => Event::Enter(object, p),
                    _ => Event::Stop(object, p),
                }
            }
            ["motion", object, room, xy, step, period] => {
                let (dx, dy): (i16, i16) =
                    pair(step.strip_prefix("step=").ok_or("missing step=")?)?;
                let period = number(period.strip_prefix("period=").ok_or("missing period=")?)?;
                if period == 0
                    || !(-128..=127).contains(&dx)
                    || !(-128..=127).contains(&dy)
                    || (dx == 0 && dy == 0)
                {
                    return Err("invalid motion".into());
                }
                Event::Motion {
                    object: object.parse()?,
                    position: position(room, xy)?,
                    dx,
                    dy,
                    period,
                }
            }
            ["state", object, "mode", mode] => {
                Event::State(object.parse()?, State::Mode(mode.parse()?))
            }
            ["state", object, "counter", counter, value] => Event::State(
                object.parse()?,
                State::Counter(counter.parse()?, number(value)?),
            ),
            ["state", object, "carry", item, offset] => {
                let (x, y) = pair(offset)?;
                Event::State(
                    object.parse()?,
                    State::Carry {
                        item: if *item == "none" {
                            None
                        } else {
                            Some(item.parse()?)
                        },
                        x,
                        y,
                    },
                )
            }
            ["state", object, "direction", value] => {
                let value = number(value)?;
                if value > 15 {
                    return Err("invalid direction".into());
                }
                Event::State(object.parse()?, State::Direction(value))
            }
            ["state", object, "game", value @ ("1" | "2" | "3")] => {
                Event::State(object.parse()?, State::Game(value.parse()?))
            }
            ["state", object, "won", value @ ("0" | "1")] => {
                Event::State(object.parse()?, State::Won(*value == "1"))
            }
            ["contact", object, target] => Event::Contact(
                object.parse()?,
                match target.parse() {
                    Ok(surface) => ContactTarget::Surface(surface),
                    Err(_) => ContactTarget::Object(target.parse()?),
                },
            ),
            ["end"] => Event::End,
            _ => return Err("invalid record or unsupported trace version".into()),
        };
        Ok(Self {
            at: clock.parse()?,
            event,
        })
    }
}

impl fmt::Display for Record {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ", self.at)?;
        match self.event {
            Event::Begin => f.write_str("trace 1 adventure-original boot"),
            Event::Init(game) => write!(f, "init {game}"),
            Event::Input(c, v) => write!(f, "input {c} {v}"),
            Event::Place(o, p) | Event::Enter(o, p) | Event::Stop(o, p) => {
                let verb = match self.event {
                    Event::Place(..) => "place",
                    Event::Enter(..) => "enter",
                    _ => "stop",
                };
                write!(f, "{verb} {o} {} {},{}", p.room, p.x, p.y)
            }
            Event::Motion {
                object,
                position: p,
                dx,
                dy,
                period,
            } => write!(
                f,
                "motion {object} {} {},{} step={dx},{dy} period={period}",
                p.room, p.x, p.y
            ),
            Event::State(o, s) => {
                write!(f, "state {o} ")?;
                match s {
                    State::Mode(m) => write!(f, "mode {m}"),
                    State::Counter(c, v) => write!(f, "counter {c} {v}"),
                    State::Carry { item, x, y } => {
                        write!(f, "carry ")?;
                        match item {
                            Some(i) => write!(f, "{i}"),
                            None => f.write_str("none"),
                        }?;
                        write!(f, " {x},{y}")
                    }
                    State::Direction(v) => write!(f, "direction {v}"),
                    State::Game(v) => write!(f, "game {v}"),
                    State::Won(v) => write!(f, "won {}", u8::from(v)),
                }
            }
            Event::Contact(o, t) => {
                write!(f, "contact {o} ")?;
                match t {
                    ContactTarget::Object(o) => write!(f, "{o}"),
                    ContactTarget::Surface(s) => write!(f, "{s}"),
                }
            }
            Event::End => f.write_str("end"),
        }
    }
}
