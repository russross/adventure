use std::cell::RefCell;

use adventure_port::adventure::Adventure;
use adventure_port::hardware::FrameHost;
use adventure_port::input::{Controls, Difficulty, Direction};
use adventure_port::replay::ReplayHost;
use adventure_port::sound::Sound;
use adventure_port::tia::Frame;
use adventure_trace::{Record, Session};

struct ScriptedHost {
    frame: u64,
    limit: u64,
    game: u8,
    seed: u64,
}
impl ScriptedHost {
    fn controls(&self) -> Controls {
        let f = self.frame;
        let direction = if f < 30 {
            Direction::default()
        } else {
            match (f / 17 + self.seed) % 5 {
                0 => Direction {
                    right: true,
                    ..Direction::default()
                },
                1 => Direction {
                    up: true,
                    ..Direction::default()
                },
                2 => Direction {
                    left: true,
                    down: true,
                    ..Direction::default()
                },
                3 => Direction {
                    right: true,
                    left: true,
                    ..Direction::default()
                },
                _ => Direction::default(),
            }
        };
        Controls {
            select: self.game >= 2 && (3..6).contains(&f) || self.game == 3 && (9..12).contains(&f),
            reset: (18..21).contains(&f) || (210..213).contains(&f),
            color: f % 71 < 35,
            left_difficulty: if f % 53 < 26 {
                Difficulty::A
            } else {
                Difficulty::B
            },
            right_difficulty: if f % 47 < 23 {
                Difficulty::A
            } else {
                Difficulty::B
            },
            left: adventure_port::input::Joystick {
                direction,
                fire: f % 19 < 3,
            },
            right: adventure_port::input::Joystick {
                direction: Direction {
                    up: f.is_multiple_of(41),
                    ..Direction::default()
                },
                fire: true,
            },
        }
    }
}
impl FrameHost for ScriptedHost {
    fn sample_controls(&mut self) -> Result<Option<Controls>, String> {
        Ok(Some(self.controls()))
    }
    fn vertical_sync(
        &mut self,
        _frame: &Frame,
        _sound: &Sound,
    ) -> Result<Option<Controls>, String> {
        if self.frame == self.limit {
            return Ok(None);
        }
        self.frame += 1;
        Ok(Some(self.controls()))
    }
}
pub fn record(game: u8, frames: u64, seed: u64) -> (Adventure, Vec<Record>) {
    let mut game_state = Adventure {
        trace: Some(RefCell::new(Session::record())),
        ..Adventure::default()
    };
    game_state
        .cartridge_entry(&mut ScriptedHost {
            frame: 0,
            limit: frames,
            game,
            seed,
        })
        .unwrap();
    let records = game_state
        .trace
        .as_ref()
        .unwrap()
        .borrow()
        .records()
        .iter()
        .map(|r| r.to_string().parse().unwrap())
        .collect();
    (game_state, records)
}
pub fn replay(records: Vec<Record>) -> Result<Adventure, String> {
    let frames = records.last().unwrap().at.frame;
    let mut game = Adventure {
        trace: Some(RefCell::new(Session::replay(records)?)),
        ..Adventure::default()
    };
    game.cartridge_entry(&mut ReplayHost::new(frames))?;
    Ok(game)
}

// Retrieve the Gold Key, align it with the gate, and enter the treasury through the opening.
pub fn record_castle() -> (Adventure, Vec<Record>) {
    struct Host(u64);
    impl FrameHost for Host {
        fn sample_controls(&mut self) -> Result<Option<Controls>, String> {
            Ok(Some(Controls::default()))
        }
        fn vertical_sync(
            &mut self,
            _frame: &Frame,
            _sound: &Sound,
        ) -> Result<Option<Controls>, String> {
            if self.0 == 270 {
                return Ok(None);
            }
            let f = self.0;
            self.0 += 1;
            Ok(Some(Controls {
                reset: f < 3,
                left: adventure_port::input::Joystick {
                    direction: Direction {
                        left: (6..54).contains(&f),
                        up: (57..90).contains(&f) || (183..252).contains(&f),
                        down: (93..129).contains(&f),
                        right: (132..180).contains(&f),
                    },
                    fire: false,
                },
                ..Controls::default()
            }))
        }
    }
    let mut game = Adventure {
        trace: Some(RefCell::new(Session::record())),
        ..Adventure::default()
    };
    game.cartridge_entry(&mut Host(0)).unwrap();
    let records = game.trace.as_ref().unwrap().borrow().records().to_vec();
    (game, records)
}
