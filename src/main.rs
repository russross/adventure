use std::cell::RefCell;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write, sink};
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::process;

use adventure::adventure::Adventure;
use adventure::hardware::{FrameHost, RendererMode, SdlHardware};
use adventure::input::Controls;
use adventure::recording::Recording;
use adventure::replay::{PlaybackHost, ReplayHost};
use adventure::sound::Sound;
use adventure::video::Frame;
use adventure_trace::{Record, Session};

#[derive(Default)]
enum Mode {
    #[default]
    Play,
    Replay(PathBuf),
    Verify(PathBuf),
}

#[derive(Default)]
struct Options {
    mode: Mode,
    frames: Option<NonZeroUsize>,
    headless: Option<bool>,
    unpaced: bool,
    snapshot: Option<PathBuf>,
    record: Option<PathBuf>,
    trace: Option<PathBuf>,
    validate: bool,
    renderer: RendererMode,
}

// Presentation selection never changes the recorded input or semantic clocks.
enum Host {
    Display(Box<SdlHardware>),
    Headless(ReplayHost),
}

impl FrameHost for Host {
    fn sample_controls(&mut self) -> Result<Option<Controls>, String> {
        match self {
            Self::Display(host) => host.sample_controls(),
            Self::Headless(host) => host.sample_controls(),
        }
    }

    fn present(&mut self, frame: &Frame, sound: &Sound) -> Result<Option<Controls>, String> {
        match self {
            Self::Display(host) => host.present(frame, sound),
            Self::Headless(host) => host.present(frame, sound),
        }
    }
}

fn print_controls() {
    println!(
        "Move: arrows / keypad 8,2,4,6; fire/drop: Space / Left Ctrl / keypad 5\n\
         Select: F1 / Tab; reset/start: F2 / Enter; color/B&W: F3/F4\n\
         Difficulty: left F5/F6, right F7/F8; mute: M; renderer: C; size: 1/2/3/4/5\n\
         Quit: Escape / Ctrl+Q / Cmd+Q"
    );
}

fn parse(args: impl Iterator<Item = String>) -> Result<Option<Options>, String> {
    let mut options = Options::default();
    let mut args = args;
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--frames" => {
                options.frames = Some(
                    args.next()
                        .ok_or("--frames needs a positive integer")?
                        .parse()
                        .map_err(|_| "--frames needs a positive integer")?,
                );
            }
            "--headless" | "--display" => {
                if options.headless.is_some() {
                    return Err("select only one of --headless and --display".into());
                }
                options.headless = Some(argument == "--headless");
            }
            "--renderer" => {
                options.renderer = match args.next().as_deref() {
                    Some("crt") => RendererMode::Crt,
                    Some("simple") => RendererMode::Simple,
                    _ => return Err("--renderer needs crt or simple".into()),
                };
            }
            "--unpaced" => options.unpaced = true,
            "--validate" => options.validate = true,
            "--record" | "--trace" | "--snapshot" => {
                let path = PathBuf::from(
                    args.next()
                        .ok_or_else(|| format!("{argument} needs a file path"))?,
                );
                let destination = match argument.as_str() {
                    "--record" => &mut options.record,
                    "--trace" => &mut options.trace,
                    _ => &mut options.snapshot,
                };
                if destination.replace(path).is_some() {
                    return Err(format!("{argument} may only be specified once"));
                }
            }
            "--replay" | "--verify" => {
                if !matches!(options.mode, Mode::Play) {
                    return Err("select only one of --replay and --verify".into());
                }
                let path = PathBuf::from(
                    args.next()
                        .ok_or_else(|| format!("{argument} needs a file path"))?,
                );
                options.mode = if argument == "--replay" {
                    Mode::Replay(path)
                } else {
                    Mode::Verify(path)
                };
            }
            "--help" | "-h" => {
                println!(
                    "Usage: adventure [OPTIONS]\n\
                    --record FILE   Record inputs during interactive play\n\
                    --replay FILE   Play an input recording (display and sound by default)\n\
                    --trace FILE    Record a trace and validate live against the cartridge\n\
                    --verify FILE   Validate a trace against both implementations (headless by default)\n\
                    --validate      Validate live play against the cartridge without saving a trace\n\
                    --headless      Run silently at full speed without SDL\n\
                    --display       Run with display, sound, and 60 Hz pacing\n\
                    --renderer MODE Select crt (default) or simple rendering\n\
                    --unpaced       Disable display pacing\n\
                    --frames N      Stop live play after N pictures\n\
                    --snapshot FILE Save the final displayed picture as BMP"
                );
                print_controls();
                return Ok(None);
            }
            _ => return Err(format!("unknown option {argument}; use --help for usage")),
        }
    }

    // Playback owns its endpoint; input recordings never enable raw validation.
    if !matches!(options.mode, Mode::Play)
        && (options.record.is_some()
            || options.trace.is_some()
            || options.validate
            || options.frames.is_some())
    {
        return Err("--replay and --verify cannot be combined with --record, --trace, --validate, or --frames".into());
    }
    let headless = options
        .headless
        .unwrap_or(matches!(options.mode, Mode::Verify(_)));
    if headless && options.snapshot.is_some() {
        return Err("--snapshot requires --display".into());
    }
    if headless && matches!(options.mode, Mode::Play) && options.frames.is_none() {
        return Err("headless live play requires --frames N".into());
    }
    if options.record.is_some() && options.record == options.trace {
        return Err("recording and trace must use different files".into());
    }
    Ok(Some(options))
}

fn load_trace(path: &PathBuf) -> Result<Vec<Record>, String> {
    BufReader::new(File::open(path).map_err(|error| format!("{}: {error}", path.display()))?)
        .lines()
        .enumerate()
        .map(|(index, line)| {
            line.map_err(|error| error.to_string())?
                .parse()
                .map_err(|error| format!("{}:{}: {error}", path.display(), index + 1))
        })
        .collect()
}

fn execute() -> Result<(), String> {
    let Some(options) = parse(env::args().skip(1))? else {
        return Ok(());
    };
    let headless = options
        .headless
        .unwrap_or(matches!(options.mode, Mode::Verify(_)));
    let mut game = Adventure::default();

    // Parse saved input before opening devices or creating output files.
    let playback_frames = match &options.mode {
        Mode::Replay(path) => {
            let recording = Recording::replay(BufReader::new(
                File::open(path).map_err(|error| format!("{}: {error}", path.display()))?,
            ))?;
            let frames = recording.frames();
            game.session.recording = Some(RefCell::new(recording));
            Some(frames)
        }
        Mode::Verify(path) => {
            let records = load_trace(path)?;
            let frames = records.last().ok_or("empty trace")?.at.frame;
            game.session.set_trace(Session::replay(records)?);
            Some(frames)
        }
        Mode::Play => None,
    };

    // Every trace workflow gets independent cartridge execution; input replay does not.
    if options.trace.is_some() || options.validate || matches!(options.mode, Mode::Verify(_)) {
        if game.session.trace().is_none() {
            let writer: Box<dyn Write> = if let Some(path) = &options.trace {
                Box::new(BufWriter::new(
                    File::create(path).map_err(|error| format!("{}: {error}", path.display()))?,
                ))
            } else {
                Box::new(sink())
            };
            game.session.set_trace(Session::record_to(writer)?);
        }
        game.session.enable_validation()?;
    }
    if let Some(path) = &options.record {
        game.session.recording = Some(RefCell::new(Recording::record_to(BufWriter::new(
            File::create(path).map_err(|error| format!("{}: {error}", path.display()))?,
        ))?));
    }

    let mut host = if headless {
        Host::Headless(ReplayHost::new(playback_frames.unwrap_or_else(|| {
            options.frames.expect("bounded headless play").get() as u64
        })))
    } else {
        print_controls();
        Host::Display(Box::new(SdlHardware::with_renderer(
            !options.unpaced,
            options.frames.map(NonZeroUsize::get),
            options.renderer,
        )?))
    };
    if let Some(frames) = playback_frames {
        let mut playback = PlaybackHost::new(&mut host, frames);
        let result = game.run(&mut playback);
        if playback.interrupted() {
            return Ok(());
        }
        result?;
        if matches!(options.mode, Mode::Verify(_)) {
            println!("Verified {frames} frames against main and ROM");
        }
    } else {
        game.run(&mut host)?;
    }
    if let (Some(path), Host::Display(display)) = (&options.snapshot, &mut host) {
        display.save_snapshot(path)?;
    }
    Ok(())
}

fn main() {
    if let Err(error) = execute() {
        eprintln!("Adventure: {error}");
        process::exit(1);
    }
}
