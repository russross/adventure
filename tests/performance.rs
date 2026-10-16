// Complete-trace workloads share the same input and stopping boundary.
use std::env;
use std::hint::black_box;
use std::num::NonZeroU32;
use std::process::ExitCode;
use std::time::Instant;

use adventure_trace::{Record, Session};
use port::adventure::Adventure;
use port::hardware::{FrameHost, SdlHardware};
use port::input::Controls;
use port::sound::{Sound, Synthesizer};

#[derive(Clone, Copy)]
enum Mode {
    Headless,
    Synthesis,
    Sdl,
    IdleSdl,
}

struct Host {
    remaining: u64,
    synthesis: Option<Synthesizer>,
    samples: [f32; 800],
    sdl: Option<SdlHardware>,
}

impl Host {
    // The extra boundary finishes verification without producing another picture or sound.
    fn boundary(&mut self, frame: &Frame, sound: &Sound) -> Result<Option<Controls>, String> {
        if self.remaining == 0 {
            return Ok(None);
        }
        self.remaining -= 1;
        black_box(&frame.pixels);
        if let Some(synthesizer) = &mut self.synthesis {
            synthesizer.render(sound, &mut self.samples);
            black_box(&self.samples);
        }
        if let Some(sdl) = &mut self.sdl {
            return present_sdl(sdl, frame, sound)
                .map(|controls| controls.map(|_| Controls::default()));
        }
        Ok(Some(Controls::default()))
    }
}

impl FrameHost for Host {
    fn sample_controls(&mut self) -> Result<Option<Controls>, String> {
        Ok(Some(Controls::default()))
    }

    fn present(&mut self, frame: &Frame, sound: &Sound) -> Result<Option<Controls>, String> {
        self.boundary(frame, sound)
    }
}

fn execute() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let mode = match args.next().as_deref() {
        Some("headless") => Mode::Headless,
        Some("synthesis") => Mode::Synthesis,
        Some("sdl") => Mode::Sdl,
        Some("idle-sdl") => Mode::IdleSdl,
        _ => {
            return Err(
                "usage: performance MODE [REPEATS]; MODE=headless|synthesis|sdl|idle-sdl".into(),
            )
        }
    };
    let repeats = args
        .next()
        .map_or(Ok(1), |value| value.parse::<usize>())
        .map_err(|_| "REPEATS must be a positive integer")?;
    if repeats == 0 || args.next().is_some() {
        return Err("REPEATS must be a positive integer with no extra arguments".into());
    }
    let records: Vec<Record> = include_str!("easy.trace")
        .lines()
        .map(str::parse)
        .collect::<Result<_, _>>()?;
    let frames = records.last().ok_or("empty trace")?.at.frame;
    let started = Instant::now();
    for _ in 0..repeats {
        let mut game = Adventure::default();
        if !matches!(mode, Mode::IdleSdl) {
            game.session.set_trace(Session::replay(records.clone())?);
        }
        let mut host = Host {
            remaining: frames,
            synthesis: match mode {
                Mode::Synthesis => Some(Synthesizer::new(NonZeroU32::new(48_000).unwrap())),
                _ => None,
            },
            samples: [0.0; 800],
            sdl: if matches!(mode, Mode::Sdl | Mode::IdleSdl) {
                Some(SdlHardware::new(true, None)?)
            } else {
                None
            },
        };
        run_game(&mut game, &mut host)?;
        if host.remaining != 0 {
            return Err("playback interrupted".into());
        }
        if !matches!(mode, Mode::IdleSdl) && !won(&game) {
            return Err("playthrough did not reach victory".into());
        }
        black_box(game);
    }
    println!(
        "{} frames={} repeats={repeats} wall_seconds={:.6}",
        env!("CARGO_PKG_NAME"),
        frames,
        started.elapsed().as_secs_f64()
    );
    Ok(())
}

fn main() -> ExitCode {
    match execute() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Performance: {error}");
            ExitCode::FAILURE
        }
    }
}
