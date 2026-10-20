//! SDL picture/audio presentation and controls. Frame pacing occurs only at vertical sync.

use std::collections::HashSet;
use std::path::Path;
use std::thread;
use std::time::{Duration, Instant};

use crt_native::{BackendPreference, Display};
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::{Keycode, Mod};
use sdl2::pixels::{Color, PixelFormat, PixelFormatEnum};
use sdl2::surface::Surface;

use crate::crt_display::CrtDisplay;
use crate::input::{Controls, Difficulty, Direction, Joystick};
use crate::sound::Sound;
use crate::streaming_audio::StreamingAudio;
use crate::video::{Frame, FrameStamp, HEIGHT, WIDTH};

// Double the horizontal pixels to preserve the console's displayed proportions.
const WINDOW_WIDTH: u32 = WIDTH as u32 * 2;
const WINDOW_HEIGHT: u32 = HEIGHT as u32;

pub use crate::runtime::FrameHost;

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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RendererMode {
    #[default]
    Crt,
    Simple,
}

pub struct SdlHardware {
    renderer: RendererMode,
    crt: CrtDisplay,
    display: Display,
    _sdl: sdl2::Sdl,
    events: sdl2::EventPump,
    keyboard: KeyboardConsole,
    next_frame: Instant,
    paced: bool,
    frames: usize,
    last_frame_work: Duration,
    frame_limit: Option<usize>,
    pixels: TexturePixels,
    stamp: Option<FrameStamp>,
    refresh: DisplayRefresh,
    #[cfg(test)]
    redraws: usize,
    audio: StreamingAudio,
    muted: bool,
}

impl SdlHardware {
    pub fn new(paced: bool, frame_limit: Option<usize>) -> Result<Self, String> {
        Self::with_renderer(paced, frame_limit, RendererMode::default())
    }

    pub fn with_renderer(
        paced: bool,
        frame_limit: Option<usize>,
        renderer: RendererMode,
    ) -> Result<Self, String> {
        let sdl = sdl2::init()?;
        let video = sdl.video()?;
        let window = video
            .window("Adventure", WINDOW_WIDTH * 5, WINDOW_HEIGHT * 5)
            .position_centered()
            .resizable()
            .metal_view()
            .build()
            .map_err(|error| error.to_string())?;
        // The presenter exclusively owns the window's graphics backend. Host
        // pacing remains authoritative for both direct GPU and SDL fallback.
        let display = Display::new(window, BackendPreference::from_env()?)?;
        println!("Graphics: {}", display.backend_name());
        let pixels = TexturePixels::new(&[PixelFormatEnum::RGBA32])?;
        let events = sdl.event_pump()?;
        let audio = StreamingAudio::new(&sdl.audio()?)?;
        Ok(Self {
            renderer,
            crt: CrtDisplay::new()?,
            _sdl: sdl,
            display,
            events,
            keyboard: KeyboardConsole::default(),
            next_frame: Instant::now(),
            paced,
            frames: 0,
            last_frame_work: Duration::ZERO,
            frame_limit,
            pixels,
            stamp: None,
            refresh: DisplayRefresh::Redraw,
            #[cfg(test)]
            redraws: 0,
            audio,
            muted: false,
        })
    }

    pub fn save_snapshot(&mut self, path: &Path) -> Result<(), String> {
        if self.renderer == RendererMode::Crt {
            // Snapshot requests also work before the first presentation or after a size shortcut.
            self.crt.resize(&self.display)?;
            return self.crt.save_snapshot(&mut self.display, path);
        }
        let pitch = self.pixels.pitch() as u32;
        let surface = Surface::from_data(
            &mut self.pixels.bytes,
            WIDTH as u32,
            HEIGHT as u32,
            pitch,
            self.pixels.format,
        )?;
        // Export the established 24-bit BMP format independently of the renderer's upload layout.
        surface
            .convert_format(PixelFormatEnum::RGB24)?
            .save_bmp(path)
    }

    pub fn renderer(&self) -> RendererMode {
        self.renderer
    }

    pub fn is_muted(&self) -> bool {
        self.muted
    }

    pub fn window_size(&self) -> (u32, u32) {
        self.display.window().size()
    }
    pub fn gpu_rendering(&self) -> bool {
        self.display.is_gpu()
    }

    /// Input, audio, source preparation, and presentation time, excluding pacing.
    /// GPU execution can continue asynchronously after this host work completes.
    pub fn last_frame_work(&self) -> Duration {
        self.last_frame_work
    }
}

impl FrameHost for SdlHardware {
    fn sample_controls(&mut self) -> Result<Option<Controls>, String> {
        for event in self.events.poll_iter() {
            self.refresh.observe(&event);
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
                    // Playback observes mute at the next presentation boundary.
                    match key {
                        Keycode::M => {
                            self.muted = !self.muted;
                        }
                        Keycode::C => {
                            self.renderer = match self.renderer {
                                RendererMode::Crt => RendererMode::Simple,
                                RendererMode::Simple => RendererMode::Crt,
                            };
                            self.stamp = None;
                            self.refresh.redraw();
                        }
                        Keycode::Num1
                        | Keycode::Num2
                        | Keycode::Num3
                        | Keycode::Num4
                        | Keycode::Num5 => {
                            let scale = match key {
                                Keycode::Num1 => 1,
                                Keycode::Num2 => 2,
                                Keycode::Num3 => 3,
                                Keycode::Num4 => 4,
                                _ => 5,
                            };
                            self.display
                                .window_mut()
                                .set_size(WINDOW_WIDTH * scale, WINDOW_HEIGHT * scale)
                                .map_err(|error| error.to_string())?;
                            self.refresh.redraw();
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
        let started = Instant::now();
        let Some(controls) = self.sample_controls()? else {
            return Ok(None);
        };

        self.audio.present(sound, self.muted)?;

        // Device resets require a new texture and a complete upload even for unchanged pixels.
        if self.refresh == DisplayRefresh::RecreateTexture {
            self.display.invalidate();
            self.stamp = None;
            self.refresh = DisplayRefresh::Redraw;
        }
        let changed = self.stamp != Some(frame.stamp());
        // Only the selected renderer consumes a new picture. Display events also
        // reach the CRT so drawable-size changes update its reconstruction view.
        match self.renderer {
            RendererMode::Simple if changed => {
                self.pixels.convert_damage(frame, self.stamp);
            }
            RendererMode::Crt if changed || self.refresh != DisplayRefresh::Clean => {
                self.crt.update(&self.display, frame)?;
            }
            _ => {}
        }
        // Each redraw rebuilds the transient backbuffer; idle boundaries leave the window alone.
        if changed || self.refresh != DisplayRefresh::Clean || self.display.needs_present() {
            self.refresh.redraw();
            match self.renderer {
                RendererMode::Crt => self.crt.draw(&mut self.display, true)?,
                RendererMode::Simple => self.display.present_rgba(
                    &self.pixels.bytes,
                    WIDTH as u32,
                    HEIGHT as u32,
                    WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32,
                )?,
            }
            self.refresh = DisplayRefresh::Clean;
            self.stamp = Some(frame.stamp());
            #[cfg(test)]
            {
                self.redraws += 1;
            }
        }
        self.frames += 1;
        self.last_frame_work = started.elapsed();

        // Presentation may wait for a surface image. Host pacing sleeps only for
        // the remaining 60 Hz budget, keeping the simulation cadence independent.
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DisplayRefresh {
    Clean,
    Redraw,
    RecreateTexture,
}

impl DisplayRefresh {
    fn redraw(&mut self) {
        if *self == Self::Clean {
            *self = Self::Redraw;
        }
    }

    // A device reset dominates redraw requests until the texture is replaced.
    fn observe(&mut self, event: &Event) {
        match event {
            Event::RenderDeviceReset { .. } => *self = Self::RecreateTexture,
            Event::RenderTargetsReset { .. } => self.redraw(),
            Event::Window {
                win_event:
                    WindowEvent::Shown
                    | WindowEvent::Exposed
                    | WindowEvent::Resized(..)
                    | WindowEvent::SizeChanged(..)
                    | WindowEvent::Restored
                    | WindowEvent::Maximized
                    | WindowEvent::DisplayChanged(..),
                ..
            } => self.redraw(),
            _ => {}
        }
    }
}

struct TexturePixels {
    format: PixelFormatEnum,
    palette: [[u8; 4]; 128],
    bytes: Vec<u8>,
}

impl TexturePixels {
    fn new(supported: &[PixelFormatEnum]) -> Result<Self, String> {
        let format = supported
            .iter()
            .copied()
            .find(|format| {
                matches!(
                    format,
                    PixelFormatEnum::ARGB8888
                        | PixelFormatEnum::ABGR8888
                        | PixelFormatEnum::RGBA8888
                        | PixelFormatEnum::BGRA8888
                        | PixelFormatEnum::RGB888
                        | PixelFormatEnum::BGR888
                )
            })
            .unwrap_or(PixelFormatEnum::RGB24);
        let pixel_format = PixelFormat::try_from(format)?;
        let palette = std::array::from_fn(|index| {
            let [r, g, b] = rgb_color((index * 2) as u8);
            if format == PixelFormatEnum::RGB24 {
                [r, g, b, 0]
            } else {
                Color::RGBA(r, g, b, 255)
                    .to_u32(&pixel_format)
                    .to_ne_bytes()
            }
        });
        // Store opaque black before the first picture so pre-presentation snapshots remain valid.
        let mut bytes = vec![0; WIDTH * HEIGHT * format.byte_size_per_pixel()];
        for pixel in bytes.chunks_mut(format.byte_size_per_pixel()) {
            pixel.copy_from_slice(&palette[0][..pixel.len()]);
        }
        Ok(Self {
            format,
            palette,
            bytes,
        })
    }

    fn pitch(&self) -> usize {
        WIDTH * self.format.byte_size_per_pixel()
    }

    fn convert_damage(&mut self, frame: &Frame, previous: Option<FrameStamp>) {
        let stride = self.format.byte_size_per_pixel();
        let pitch = self.pitch();
        // Palette conversion follows scene damage; unchanged rows and horizontal
        // prefixes/suffixes retain their packed pixels without a raster comparison.
        for (row, span) in frame.damage_since(previous).into_iter().enumerate() {
            let Some(span) = span else { continue };
            for (output, color) in self.bytes
                [row * pitch + span.start * stride..row * pitch + span.end * stride]
                .chunks_mut(stride)
                .zip(&frame.pixels()[row * WIDTH + span.start..row * WIDTH + span.end])
            {
                output.copy_from_slice(&self.palette[usize::from(*color >> 1)][..stride]);
            }
        }
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
#[path = "tests/hardware.rs"]
mod tests;
