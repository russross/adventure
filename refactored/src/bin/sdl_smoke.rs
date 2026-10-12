//! Drive real SDL events through a short play session, then save the presented picture.

use std::num::NonZeroU32;
use std::path::Path;
use std::process;

use adventure_refactored::game::{Adventure, ObjectId, Room};
use adventure_refactored::hardware::{FrameHost, SdlHardware};
use adventure_refactored::input::Controls;
use adventure_refactored::sound::{Sound, Synthesizer};
use adventure_refactored::video::Frame;
use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Mod};

struct Session {
    display: SdlHardware,
    events: sdl2::EventSubsystem,
    frame: usize,
    synthesizer: Synthesizer,
    audible_samples: usize,
}

impl FrameHost for Session {
    fn sample_controls(&mut self) -> Result<Option<Controls>, String> {
        self.display.sample_controls()
    }

    fn present(&mut self, picture: &Frame, sound: &Sound) -> Result<Option<Controls>, String> {
        let transition = match self.frame {
            0 => Some((Keycode::Return, true)),
            3 => Some((Keycode::Return, false)),
            6 => Some((Keycode::Left, true)),
            9 | 10 | 15 => Some((Keycode::M, true)),
            12 | 16 => Some((Keycode::M, false)),
            18 => Some((Keycode::Num1, true)),
            19 => Some((Keycode::Num1, false)),
            21 | 27 => Some((Keycode::Num2, true)),
            22 | 28 => Some((Keycode::Num2, false)),
            24 => Some((Keycode::Num3, true)),
            25 => Some((Keycode::Num3, false)),
            54 => Some((Keycode::Left, false)),
            57 => Some((Keycode::Up, true)),
            90 => Some((Keycode::Up, false)),
            _ => None,
        };
        if let Some((key, pressed)) = transition {
            let event = if pressed {
                Event::KeyDown {
                    timestamp: 0,
                    window_id: 1,
                    keycode: Some(key),
                    scancode: None,
                    keymod: Mod::NOMOD,
                    repeat: self.frame == 10,
                }
            } else {
                Event::KeyUp {
                    timestamp: 0,
                    window_id: 1,
                    keycode: Some(key),
                    scancode: None,
                    keymod: Mod::NOMOD,
                    repeat: false,
                }
            };
            self.events.push_event(event)?;
        }
        self.frame += 1;
        let mut samples = [0.0; 800];
        self.synthesizer.render(sound, &mut samples);
        self.audible_samples += samples.iter().filter(|sample| sample.abs() > 0.001).count();
        let controls = self.display.present(picture, sound)?;
        let frame = self.frame - 1;
        if self.display.is_muted() != (9..15).contains(&frame) {
            return Err(format!("mute toggle or key repeat failed at frame {frame}"));
        }
        // Verify the real SDL window dimensions after resizing, while the held
        // arrow continues to drive the same game session through each shortcut.
        let expected_size = match frame {
            18..=20 => (320, 194),
            24..=26 => (960, 582),
            _ => (640, 388),
        };
        if self.display.window_size() != expected_size {
            return Err(format!("window size shortcut failed at frame {frame}"));
        }
        Ok(controls)
    }
}

fn execute() -> Result<(), String> {
    let sdl = sdl2::init()?;
    let mut session = Session {
        display: SdlHardware::new(true, Some(105))?,
        events: sdl.event()?,
        frame: 0,
        synthesizer: Synthesizer::new(NonZeroU32::new(48_000).unwrap()),
        audible_samples: 0,
    };
    let mut game = Adventure::default();
    game.run(&mut session)?;
    session
        .display
        .save_snapshot(Path::new("../target/refactored-smoke.bmp"))?;
    if game.world.player.room != Room::GoldCastle || game.world.held != ObjectId::GoldKey {
        return Err(format!(
            "SDL session failed to pick up the Gold Key: player={:?}, held={:?}",
            game.world.player, game.world.held
        ));
    }
    if session.audible_samples == 0 {
        return Err("SDL session produced no Gold Key pickup audio".into());
    }
    println!(
        "SDL session: Gold Key pickup with audio, mute toggle/repeat, 1x/2x/3x resizing, 105 presented frames."
    );
    Ok(())
}

fn main() {
    if let Err(error) = execute() {
        eprintln!("SDL smoke: {error}");
        process::exit(1);
    }
}
