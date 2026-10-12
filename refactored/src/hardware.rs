//! SDL picture/audio presentation and controls. Frame pacing occurs only at vertical sync.

use std::collections::HashSet;
use std::num::NonZeroU32;
use std::path::Path;
use std::thread;
use std::time::{Duration, Instant};

use sdl2::audio::{AudioQueue, AudioSpecDesired};
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::{Keycode, Mod};
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::Canvas;
use sdl2::surface::Surface;
use sdl2::video::Window;

use crate::input::{Controls, Difficulty, Direction, Joystick};
use crate::sound::{Sound, Synthesizer};
use crate::video::{Frame, HEIGHT, WIDTH};

// Double the horizontal pixels to preserve the console's displayed proportions.
const WINDOW_WIDTH: u32 = WIDTH as u32 * 2;
const WINDOW_HEIGHT: u32 = HEIGHT as u32;

pub trait FrameHost {
    /// Sample controls without presenting, generating audio, or advancing the frame limit.
    fn sample_controls(&mut self) -> Result<Option<Controls>, String>;

    /// Process input and present picture and sound; None means the host is stopping.
    fn present(&mut self, frame: &Frame, sound: &Sound) -> Result<Option<Controls>, String>;
}

#[derive(Default)]
struct KeyboardConsole {
    pressed: HashSet<Keycode>,
    controls: Controls,
}

impl KeyboardConsole {
    fn key_down(&mut self, key: Keycode) {
        self.pressed.insert(key);
        match key {
            Keycode::F3 => self.controls.color = true,
            Keycode::F4 => self.controls.color = false,
            Keycode::F5 => self.controls.left_difficulty = Difficulty::A,
            Keycode::F6 => self.controls.left_difficulty = Difficulty::B,
            Keycode::F7 => self.controls.right_difficulty = Difficulty::A,
            Keycode::F8 => self.controls.right_difficulty = Difficulty::B,
            _ => {}
        }
    }

    // Aliases remain independent held keys, so releasing one cannot release another.
    fn sample(&mut self) -> Controls {
        let held = |keys: &[Keycode]| keys.iter().any(|key| self.pressed.contains(key));
        self.controls.left = Joystick {
            direction: Direction {
                right: held(&[Keycode::Right, Keycode::Kp6]),
                left: held(&[Keycode::Left, Keycode::Kp4]),
                up: held(&[Keycode::Up, Keycode::Kp8]),
                down: held(&[Keycode::Down, Keycode::Kp2]),
            },
            fire: held(&[Keycode::LCtrl, Keycode::Space, Keycode::Kp5]),
        };
        self.controls.right = Joystick {
            direction: Direction {
                right: held(&[Keycode::J]),
                left: held(&[Keycode::G]),
                up: held(&[Keycode::Y]),
                down: held(&[Keycode::H]),
            },
            fire: held(&[Keycode::F]),
        };
        self.controls.select = held(&[Keycode::F1, Keycode::Tab]);
        self.controls.reset = held(&[Keycode::F2, Keycode::Return, Keycode::KpEnter]);
        self.controls
    }
}

pub struct SdlHardware {
    _sdl: sdl2::Sdl,
    canvas: Canvas<Window>,
    events: sdl2::EventPump,
    keyboard: KeyboardConsole,
    next_frame: Instant,
    paced: bool,
    frames: usize,
    frame_limit: Option<usize>,
    rgb: Vec<u8>,
    audio: AudioQueue<f32>,
    synthesizer: Synthesizer,
    samples: Vec<f32>,
    sample_rate: u32,
    sample_remainder: u32,
    muted: bool,
}

impl SdlHardware {
    pub fn new(paced: bool, frame_limit: Option<usize>) -> Result<Self, String> {
        let sdl = sdl2::init()?;
        let video = sdl.video()?;
        let window = video
            .window("Adventure", WINDOW_WIDTH * 2, WINDOW_HEIGHT * 2)
            .position_centered()
            .resizable()
            .build()
            .map_err(|error| error.to_string())?;
        let mut canvas = window
            .into_canvas()
            .software()
            .build()
            .map_err(|error| error.to_string())?;
        canvas
            .set_logical_size(WINDOW_WIDTH, WINDOW_HEIGHT)
            .map_err(|error| error.to_string())?;
        let events = sdl.event_pump()?;
        // Queue one picture's worth of mono audio at each vertical boundary.
        // SDL consumes it independently; game execution stays on the main thread.
        let audio = sdl
            .audio()?
            .open_queue::<f32, _>(
                None,
                &AudioSpecDesired {
                    freq: Some(48_000),
                    channels: Some(1),
                    samples: Some(512),
                },
            )
            .map_err(|error| format!("cannot open audio output: {error}"))?;
        let sample_rate = u32::try_from(audio.spec().freq)
            .ok()
            .and_then(NonZeroU32::new)
            .ok_or("invalid SDL audio sample rate")?;
        let synthesizer = Synthesizer::new(sample_rate);
        audio.resume();
        Ok(Self {
            _sdl: sdl,
            canvas,
            events,
            keyboard: KeyboardConsole::default(),
            next_frame: Instant::now(),
            paced,
            frames: 0,
            frame_limit,
            rgb: vec![0; WIDTH * HEIGHT * 3],
            audio,
            synthesizer,
            samples: Vec::with_capacity(sample_rate.get() as usize / 60 + 1),
            sample_rate: sample_rate.get(),
            sample_remainder: 0,
            muted: false,
        })
    }

    pub fn save_snapshot(&mut self, path: &Path) -> Result<(), String> {
        let surface = Surface::from_data(
            &mut self.rgb,
            WIDTH as u32,
            HEIGHT as u32,
            WIDTH as u32 * 3,
            PixelFormatEnum::RGB24,
        )?;
        surface.save_bmp(path)
    }

    pub fn is_muted(&self) -> bool {
        self.muted
    }

    pub fn window_size(&self) -> (u32, u32) {
        self.canvas.window().size()
    }
}

impl FrameHost for SdlHardware {
    fn sample_controls(&mut self) -> Result<Option<Controls>, String> {
        for event in self.events.poll_iter() {
            match event {
                Event::Quit { .. } => return Ok(None),
                Event::Window {
                    win_event: WindowEvent::FocusLost,
                    ..
                } => self.keyboard.pressed.clear(),
                Event::KeyDown {
                    keycode: Some(key),
                    keymod,
                    repeat: false,
                    ..
                } => {
                    let quit_modifier = keymod
                        .intersects(Mod::LCTRLMOD | Mod::RCTRLMOD | Mod::LGUIMOD | Mod::RGUIMOD);
                    if key == Keycode::Escape || key == Keycode::Q && quit_modifier {
                        return Ok(None);
                    }
                    // Host shortcuts never reach the game's console/joystick state.
                    // Clear pending sound immediately, but keep synthesis running.
                    match key {
                        Keycode::M => {
                            self.muted = !self.muted;
                            self.audio.clear();
                            if self.muted {
                                self.audio.pause();
                            } else {
                                self.audio.resume();
                            }
                        }
                        Keycode::Num1 | Keycode::Num2 | Keycode::Num3 => {
                            let scale = match key {
                                Keycode::Num1 => 1,
                                Keycode::Num2 => 2,
                                _ => 3,
                            };
                            self.canvas
                                .window_mut()
                                .set_size(WINDOW_WIDTH * scale, WINDOW_HEIGHT * scale)
                                .map_err(|error| error.to_string())?;
                        }
                        _ => self.keyboard.key_down(key),
                    }
                }
                Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    self.keyboard.pressed.remove(&key);
                }
                _ => {}
            }
        }
        Ok(Some(self.keyboard.sample()))
    }

    fn present(&mut self, frame: &Frame, sound: &Sound) -> Result<Option<Controls>, String> {
        if self.frame_limit.is_some_and(|limit| self.frames >= limit) {
            return Ok(None);
        }
        let Some(controls) = self.sample_controls()? else {
            return Ok(None);
        };

        // Keep fractional samples across frames if SDL supplies a rate not divisible
        // by 60. Oscillator phase continues even when an unpaced run outruns playback.
        self.sample_remainder += self.sample_rate;
        self.samples
            .resize((self.sample_remainder / 60) as usize, 0.0);
        self.sample_remainder %= 60;
        self.synthesizer.render(sound, &mut self.samples);
        if self.audio.size() > self.sample_rate / 60 * 3 * size_of::<f32>() as u32 {
            self.audio.clear();
        }
        if !self.muted {
            self.audio.queue_audio(&self.samples)?;
        }

        // Expand indexed colors only when the complete picture reaches the SDL boundary.
        for (pixel, color) in self
            .rgb
            .as_chunks_mut::<3>()
            .0
            .iter_mut()
            .zip(&frame.pixels)
        {
            pixel.copy_from_slice(&rgb_color(*color));
        }
        let creator = self.canvas.texture_creator();
        let mut texture = creator
            .create_texture_streaming(PixelFormatEnum::RGB24, WIDTH as u32, HEIGHT as u32)
            .map_err(|error| error.to_string())?;
        texture
            .update(None, &self.rgb, WIDTH * 3)
            .map_err(|error| error.to_string())?;
        self.canvas.clear();
        self.canvas.copy(&texture, None, None)?;
        self.canvas.present();
        self.frames += 1;

        // No scanline sleeps or renderer vsync: this is the only pacing point.
        if self.paced {
            self.next_frame += Duration::from_nanos(1_000_000_000 / 60);
            if let Some(remaining) = self.next_frame.checked_duration_since(Instant::now()) {
                thread::sleep(remaining);
            } else {
                self.next_frame = Instant::now();
            }
        }
        Ok(Some(controls))
    }
}

// Fixed NTSC presentation palette. Color bit zero is ignored, as on the TIA.
// Reference: https://github.com/stella-emu/stella/blob/master/src/common/PaletteHandler.cxx
const NTSC_PALETTE: [u32; 128] = [
    0x000000, 0x4a4a4a, 0x6f6f6f, 0x8e8e8e, 0xaaaaaa, 0xc0c0c0, 0xd6d6d6, 0xececec, 0x484800,
    0x69690f, 0x86861d, 0xa2a22a, 0xbbbb35, 0xd2d240, 0xe8e84a, 0xfcfc54, 0x7c2c00, 0x904811,
    0xa26221, 0xb47a30, 0xc3903d, 0xd2a44a, 0xdfb755, 0xecc860, 0x901c00, 0xa33915, 0xb55328,
    0xc66c3a, 0xd5824a, 0xe39759, 0xf0aa67, 0xfcbc74, 0x940000, 0xa71a1a, 0xb83232, 0xc84848,
    0xd65c5c, 0xe46f6f, 0xf08080, 0xfc9090, 0x840064, 0x97197a, 0xa8308f, 0xb846a2, 0xc659b3,
    0xd46cc3, 0xe07cd2, 0xec8ce0, 0x500084, 0x68199a, 0x7d30ad, 0x9246c0, 0xa459d0, 0xb56ce0,
    0xc57cee, 0xd48cfc, 0x140090, 0x331aa3, 0x4e32b5, 0x6848c6, 0x7f5cd5, 0x956fe3, 0xa980f0,
    0xbc90fc, 0x000094, 0x181aa7, 0x2d32b8, 0x4248c8, 0x545cd6, 0x656fe4, 0x7580f0, 0x8490fc,
    0x001c88, 0x183b9d, 0x2d57b0, 0x4272c2, 0x548ad2, 0x65a0e1, 0x75b5ef, 0x84c8fc, 0x003064,
    0x185080, 0x2d6d98, 0x4288b0, 0x54a0c5, 0x65b7d9, 0x75cceb, 0x84e0fc, 0x004030, 0x18624e,
    0x2d8169, 0x429e82, 0x54b899, 0x65d1ae, 0x75e7c2, 0x84fcd4, 0x004400, 0x1a661a, 0x328432,
    0x48a048, 0x5cba5c, 0x6fd26f, 0x80e880, 0x90fc90, 0x143c00, 0x355f18, 0x527e2d, 0x6e9c42,
    0x87b754, 0x9ed065, 0xb4e775, 0xc8fc84, 0x303800, 0x505916, 0x6d762b, 0x88923e, 0xa0ab4f,
    0xb7c25f, 0xccd86e, 0xe0ec7c, 0x482c00, 0x694d14, 0x866a26, 0xa28638, 0xbb9f47, 0xd2b656,
    0xe8cc63, 0xfce070,
];

fn rgb_color(color: u8) -> [u8; 3] {
    let rgb = NTSC_PALETTE[usize::from(color >> 1)];
    [(rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8]
}

#[cfg(test)]
#[path = "hardware_tests.rs"]
mod tests;
