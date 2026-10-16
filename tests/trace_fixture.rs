use adventure::adventure::Adventure;
use adventure::input::{Controls, Difficulty, Direction};
use adventure::replay::ReplayHost;
use adventure::runtime::FrameHost;
use adventure::sound::Sound;
use adventure::video::Frame;
use adventure_trace::{Record, Session};

pub fn attach_validation(game: &mut Adventure) {
    game.session.enable_validation().unwrap();
}

// Keep the recorded complete playthrough independent of generated test inputs.
pub fn easy_playthrough() -> Vec<Record> {
    let records: Vec<Record> = include_str!("easy.trace")
        .lines()
        .map(|line| line.parse().unwrap())
        .collect();
    assert_eq!(records.last().unwrap().at.frame, 4045);
    assert!(records.iter().any(|record| matches!(
        record.event,
        adventure_trace::Event::State(_, adventure_trace::State::Won(true))
    )));
    records
}

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
            left: adventure::input::Joystick {
                direction,
                fire: f % 19 < 3,
            },
            right: adventure::input::Joystick {
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
    fn present(&mut self, _frame: &Frame, _sound: &Sound) -> Result<Option<Controls>, String> {
        if self.frame == self.limit {
            return Ok(None);
        }
        self.frame += 1;
        Ok(Some(self.controls()))
    }
}
pub fn record(game: u8, frames: u64, seed: u64) -> (Adventure, Vec<Record>) {
    let mut game_state = Adventure::default();
    game_state.session.set_trace(Session::record());
    attach_validation(&mut game_state);
    game_state
        .run(&mut ScriptedHost {
            frame: 0,
            limit: frames,
            game,
            seed,
        })
        .unwrap();
    let records = game_state
        .session
        .trace()
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
    let mut game = Adventure::default();
    game.session.set_trace(Session::replay(records)?);
    attach_validation(&mut game);
    game.run(&mut ReplayHost::new(frames))?;
    Ok(game)
}

// Retrieve the Gold Key, align it with the gate, and enter the treasury through the opening.
pub fn record_castle() -> (Adventure, Vec<Record>) {
    struct Host(u64);
    impl FrameHost for Host {
        fn sample_controls(&mut self) -> Result<Option<Controls>, String> {
            Ok(Some(Controls::default()))
        }
        fn present(&mut self, _frame: &Frame, _sound: &Sound) -> Result<Option<Controls>, String> {
            if self.0 == 270 {
                return Ok(None);
            }
            let f = self.0;
            self.0 += 1;
            Ok(Some(Controls {
                reset: f < 3,
                left: adventure::input::Joystick {
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
    let mut game = Adventure::default();
    game.session.set_trace(Session::record());
    attach_validation(&mut game);
    game.run(&mut Host(0)).unwrap();
    let records = game
        .session
        .trace()
        .as_ref()
        .unwrap()
        .borrow()
        .records()
        .to_vec();
    (game, records)
}
