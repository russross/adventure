use std::env;
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::process;

use adventure_refactored::adventure::Adventure;
use adventure_refactored::hardware::SdlHardware;

#[derive(Default)]
struct Options {
    frames: Option<NonZeroUsize>,
    unpaced: bool,
    snapshot: Option<PathBuf>,
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
            "--snapshot" => {
                options.snapshot = Some(PathBuf::from(
                    args.next().ok_or("--snapshot needs a BMP path")?,
                ))
            }
            "--help" | "-h" => {
                println!("Usage: adventure [--frames N] [--unpaced] [--snapshot FILE.bmp]");
                print_controls();
                return Ok(());
            }
            _ => return Err(format!("unknown option {argument}; use --help for usage")),
        }
    }

    // SDL presents pictures and polls controls; the game advances its picture phases.
    let mut hardware = SdlHardware::new(!options.unpaced, options.frames.map(NonZeroUsize::get))?;
    print_controls();
    let mut game = Adventure::default();
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
