use std::cell::RefCell;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter};
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::process;

use adventure_trace::{Record, Session};
use frame::adventure::Adventure;
use frame::hardware::SdlHardware;
use frame::recording::Recording;
use frame::replay::PlaybackHost;

#[derive(Default)]
struct Options {
    frames: Option<NonZeroUsize>,
    unpaced: bool,
    snapshot: Option<PathBuf>,
    record: Option<PathBuf>,
    replay: Option<PathBuf>,
    trace: Option<PathBuf>,
    verify: Option<PathBuf>,
}

// Share the compact keyboard reference between normal startup and command help.
fn print_controls() {
    println!(
        "\nAdventure controls\n\
         ------------------\n\
           Move          Arrow keys / keypad 8, 2, 4, 6\n\
           Fire / drop   Space / Left Ctrl / keypad 5\n\
           Select game   F1 / Tab\n\
           Reset / start F2 / Enter / keypad Enter\n\
           Color / B&W   F3 / F4\n\
           Difficulty    Left: F5 (A), F6 (B); Right: F7 (A), F8 (B)\n\
           Mute toggle   M\n\
           Window size   1: 1x   2: 2x   3: 3x\n\
           Quit          Escape / Ctrl+Q / Cmd+Q\n"
    );
}

fn execute() -> Result<(), String> {
    let mut options = Options::default();
    let mut args = env::args().skip(1);
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--frames" => {
                let value = args.next().ok_or("--frames needs a positive integer")?;
                options.frames = Some(
                    value
                        .parse()
                        .map_err(|_| "--frames needs a positive integer")?,
                );
            }
            "--unpaced" => options.unpaced = true,
            "--record" => {
                options.record = Some(PathBuf::from(
                    args.next().ok_or("--record needs a recording path")?,
                ))
            }
            "--replay" => {
                options.replay = Some(PathBuf::from(
                    args.next().ok_or("--replay needs a recording path")?,
                ))
            }
            "--trace" => {
                options.trace = Some(PathBuf::from(
                    args.next().ok_or("--trace needs a trace path")?,
                ))
            }
            "--verify" => {
                options.verify = Some(PathBuf::from(
                    args.next().ok_or("--verify needs a trace path")?,
                ))
            }
            "--snapshot" => {
                options.snapshot = Some(PathBuf::from(
                    args.next().ok_or("--snapshot needs a BMP path")?,
                ))
            }
            "--help" | "-h" => {
                println!(
                    "Usage: frame [--frames N] [--unpaced] [--snapshot FILE.bmp] [--record FILE] [--trace FILE]\n       frame --replay FILE\n       frame --verify FILE"
                );
                print_controls();
                return Ok(());
            }
            _ => return Err(format!("unknown option {argument}; use --help for usage")),
        }
    }

    // Playback owns the stopping boundary; presentation shortcuts remain live.
    if (options.replay.is_some() || options.verify.is_some())
        && (options.replay.is_some() && options.verify.is_some()
            || options.record.is_some()
            || options.trace.is_some()
            || options.snapshot.is_some()
            || options.frames.is_some()
            || options.unpaced)
    {
        return Err(
            "--replay and --verify cannot be combined with each other or live-play options".into(),
        );
    }
    if let Some(path) = options.replay {
        let file = File::open(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let recording = Recording::replay(BufReader::new(file))
            .map_err(|e| format!("{}: {e}", path.display()))?;
        let frames = recording.frames();
        let mut game = Adventure::default();
        game.recording = Some(RefCell::new(recording));
        let mut host = PlaybackHost::new(SdlHardware::new(true, None)?, frames);
        print_controls();
        game.run(&mut host)?;
        return Ok(());
    }

    // Verification independently generates the complete semantic trace.
    if let Some(path) = options.verify {
        let file = File::open(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let records: Vec<Record> = BufReader::new(file)
            .lines()
            .enumerate()
            .map(|(index, line)| {
                line.map_err(|e| e.to_string())?
                    .parse()
                    .map_err(|e| format!("{}:{}: {e}", path.display(), index + 1))
            })
            .collect::<Result<_, String>>()?;
        let frames = records.last().ok_or("empty trace")?.at.frame;
        let mut game = Adventure::default();
        game.trace = Some(RefCell::new(Session::replay(records)?));
        let mut host = PlaybackHost::new(SdlHardware::new(true, None)?, frames);
        print_controls();
        let result = game.run(&mut host);
        if host.interrupted() {
            return Ok(());
        }
        result?;
        println!("Verified {frames} frames");
        return Ok(());
    }

    // SDL owns presentation and input; the port owns game state and the original call order.
    let mut hardware = SdlHardware::new(!options.unpaced, options.frames.map(NonZeroUsize::get))?;
    print_controls();
    let mut game = Adventure::default();
    if let Some(path) = options.record {
        let file = File::create(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        game.recording = Some(RefCell::new(Recording::record_to(BufWriter::new(file))?));
    }
    if let Some(path) = options.trace {
        let file = File::create(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        game.trace = Some(RefCell::new(Session::record_to(BufWriter::new(file))?));
    }
    game.run(&mut hardware)?;
    if let Some(path) = options.snapshot {
        hardware.save_snapshot(&path)?;
    }
    Ok(())
}

fn main() {
    if let Err(error) = execute() {
        eprintln!("Adventure: {error}");
        process::exit(1);
    }
}
