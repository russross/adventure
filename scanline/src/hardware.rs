//! SDL picture/audio presentation and controls. Frame pacing occurs only at vertical sync.

use std::collections::HashSet;
use std::path::Path;
use std::thread;
use std::time::{Duration, Instant};

use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::{Keycode, Mod};
use sdl2::pixels::{Color, PixelFormat, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture};
use sdl2::surface::Surface;
use sdl2::video::Window;

use crate::input::{Controls, Difficulty, Direction, Joystick};
use crate::sound::Sound;
use crate::streaming_audio::StreamingAudio;
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
    texture: Texture,
    events: sdl2::EventPump,
    keyboard: KeyboardConsole,
    next_frame: Instant,
    paced: bool,
    frames: usize,
    frame_limit: Option<usize>,
    pixels: TexturePixels,
    indexed: Option<Vec<u8>>,
    refresh: DisplayRefresh,
    #[cfg(test)]
    redraws: usize,
    audio: StreamingAudio,
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
        // Default SDL selection prefers acceleration but permits a software fallback.
        // Vsync stays disabled so the host retains its single explicit pacing point.
        let mut canvas = window
            .into_canvas()
            .build()
            .map_err(|error| error.to_string())?;
        canvas
            .set_logical_size(WINDOW_WIDTH, WINDOW_HEIGHT)
            .map_err(|error| error.to_string())?;
        // SDL's canvas-owned texture is released with this renderer. It is private and
        // used only with this canvas; no manual texture destruction is necessary.
        let pixels = TexturePixels::new(&canvas.info().texture_formats)?;
        let texture = pixels.create_texture(&mut canvas)?;
        let events = sdl.event_pump()?;
        // One persistent synthesizer retains phase through effects, silence, and mute.
        let audio = StreamingAudio::new(&sdl.audio()?)?;
        Ok(Self {
            _sdl: sdl,
            canvas,
            texture,
            events,
            keyboard: KeyboardConsole::default(),
            next_frame: Instant::now(),
            paced,
            frames: 0,
            frame_limit,
            pixels,
            indexed: None,
            refresh: DisplayRefresh::Redraw,
            #[cfg(test)]
            redraws: 0,
            audio,
            muted: false,
        })
    }

    pub fn save_snapshot(&mut self, path: &Path) -> Result<(), String> {
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
        let Some(controls) = self.sample_controls()? else {
            return Ok(None);
        };

        self.audio.present(sound, self.muted)?;

        // Device resets require a new texture and a complete upload even for unchanged pixels.
        if self.refresh == DisplayRefresh::RecreateTexture {
            self.texture = self.pixels.create_texture(&mut self.canvas)?;
            self.indexed = None;
            self.refresh = DisplayRefresh::Redraw;
        }
        let spans = self
            .pixels
            .convert_changed_rows(&frame.pixels, self.indexed.as_deref());
        for &(first, end) in &spans {
            let pitch = self.pixels.pitch();
            self.texture
                .update(
                    Rect::new(0, first as i32, WIDTH as u32, (end - first) as u32),
                    &self.pixels.bytes[first * pitch..end * pitch],
                    pitch,
                )
                .map_err(|error| error.to_string())?;
        }
        self.indexed
            .get_or_insert_with(|| vec![0; WIDTH * HEIGHT])
            .copy_from_slice(&frame.pixels);
        // Each redraw rebuilds the transient backbuffer; idle boundaries leave the window alone.
        if !spans.is_empty() || self.refresh != DisplayRefresh::Clean {
            self.refresh.redraw();
            self.canvas.clear();
            self.canvas.copy(&self.texture, None, None)?;
            self.canvas.present();
            self.refresh = DisplayRefresh::Clean;
            #[cfg(test)]
            {
                self.redraws += 1;
            }
        }
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

    fn create_texture(&self, canvas: &mut Canvas<Window>) -> Result<Texture, String> {
        canvas
            .create_texture_streaming(self.format, WIDTH as u32, HEIGHT as u32)
            .map_err(|error| error.to_string())
    }

    fn convert_changed_rows(
        &mut self,
        pixels: &[u8],
        previous: Option<&[u8]>,
    ) -> Vec<(usize, usize)> {
        let mut spans = Vec::new();
        let mut start = None;
        let stride = self.format.byte_size_per_pixel();
        let pitch = self.pitch();
        // Compare complete indexed rows so palette changes and erasures follow the same path.
        for row in 0..HEIGHT {
            let range = row * WIDTH..(row + 1) * WIDTH;
            if previous.is_none_or(|old| old[range.clone()] != pixels[range.clone()]) {
                start.get_or_insert(row);
                for (output, color) in self.bytes[row * pitch..(row + 1) * pitch]
                    .chunks_mut(stride)
                    .zip(&pixels[range])
                {
                    output.copy_from_slice(&self.palette[usize::from(*color >> 1)][..stride]);
                }
            } else if let Some(first) = start.take() {
                spans.push((first, row));
            }
        }
        if let Some(first) = start {
            spans.push((first, HEIGHT));
        }
        spans
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
