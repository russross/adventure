//! Emit deterministic frame-runtime observations for the independent Python port's tests.

use std::io::{self, BufWriter, Write};
use std::process::ExitCode;

use frame::adventure::{Adventure, GateState, Position};
use frame::hardware::FrameHost;
use frame::input::{Controls, Difficulty, Direction, Joystick};
use frame::sound::Sound;
use frame::video::Frame;

const PICTURES: usize = 420;

struct ReferenceHost<'a> {
    output: &'a mut dyn Write,
    game: usize,
    pictures: usize,
}

// Console edges select each game through its ordinary initialization path.
fn controls(game: usize, picture: usize) -> Controls {
    let direction = if picture < 105 {
        Direction {
            left: (18..66).contains(&picture),
            up: (69..102).contains(&picture),
            ..Direction::default()
        }
    } else {
        let phase = picture / 17;
        Direction {
            right: phase.is_multiple_of(4),
            left: phase % 4 == 2,
            up: phase % 4 == 1,
            down: phase % 4 == 3,
        }
    };
    Controls {
        select: game >= 2 && picture == 0 || game == 3 && picture == 6,
        reset: picture == 12 || picture == 300,
        color: (picture / 90).is_multiple_of(2),
        left_difficulty: if picture < 180 {
            Difficulty::B
        } else {
            Difficulty::A
        },
        right_difficulty: if picture < 210 {
            Difficulty::B
        } else {
            Difficulty::A
        },
        left: Joystick {
            direction,
            fire: picture % 31 < 3,
        },
        ..Controls::default()
    }
}

impl FrameHost for ReferenceHost<'_> {
    fn sample_controls(&mut self) -> Result<Option<Controls>, String> {
        Ok(Some(Controls::default()))
    }

    fn present(&mut self, frame: &Frame, sound: &Sound) -> Result<Option<Controls>, String> {
        if self.pictures == PICTURES {
            return Ok(None);
        }
        self.output
            .write_all(&frame.pixels)
            .map_err(|error| error.to_string())?;
        for channel in sound.channels {
            self.output
                .write_all(&[channel.waveform, channel.frequency, channel.volume])
                .map_err(|error| error.to_string())?;
        }
        let next = controls(self.game, self.pictures);
        self.pictures += 1;
        Ok(Some(next))
    }
}

// Final positions and control state catch divergence outside the current room's picture.
fn world_bytes(game: &Adventure) -> Vec<u8> {
    let world = &game.world;
    let positions: [Position; 15] = [
        world.player,
        world.light,
        world.dot,
        world.rhindle.position,
        world.yorgle.position,
        world.grundle.position,
        world.magnet,
        world.sword,
        world.chalice,
        world.bridge,
        world.gold_key,
        world.white_key,
        world.black_key,
        world.bat.position,
        world.safe_player,
    ];
    let mut bytes: Vec<u8> = positions
        .into_iter()
        .flat_map(|position| [position.room as u8, position.x, position.y])
        .collect();
    let held = world
        .carrying
        .item
        .object()
        .map_or(255, |object| object as u8);
    bytes.extend([
        world.game as u8,
        u8::from(world.won),
        held,
        world.carrying.offset.x,
        world.carrying.offset.y,
        world.idle_ticks,
        world.idle_periods,
    ]);
    for gate in [world.gates.gold, world.gates.white, world.gates.black] {
        bytes.push(match gate {
            GateState::Open => 1,
            GateState::Closed => 28,
            GateState::Moving(value) => value,
        });
    }
    bytes
}

fn run() -> Result<(), String> {
    let mut output = BufWriter::new(io::stdout().lock());
    for number in 1..=3 {
        let mut game = Adventure::default();
        game.run(&mut ReferenceHost {
            output: &mut output,
            game: number,
            pictures: 0,
        })?;
        output
            .write_all(&world_bytes(&game))
            .map_err(|error| error.to_string())?;
    }
    output.flush().map_err(|error| error.to_string())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Python reference: {error}");
            ExitCode::FAILURE
        }
    }
}
