//! Retained Atari signal and window-resolution CRT output for the SDL host.

use std::path::Path;

use crt::source::atari2600::{AtariEncoder, AtariLine, AtariProfile, TiaColor};
use crt::{
    CompositeConfig, ConnectionProfile, Fit, OutputBufferMut, PixelFormat, RasterLayout,
    RenderStamp, Renderer, SignalLineBuffer, ViewConfig,
};
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture};
use sdl2::surface::Surface;
use sdl2::video::Window;

use crate::video::{Frame, HEIGHT, WIDTH};

pub(crate) struct CrtDisplay {
    renderer: Renderer,
    encoder: AtariEncoder,
    signal: SignalLineBuffer,
    indexed: Option<Vec<u8>>,
    stamp: Option<RenderStamp>,
    pub(crate) bytes: Vec<u8>,
    size: (u32, u32),
    texture: Option<Texture>,
}

impl CrtDisplay {
    pub(crate) fn new() -> Result<Self, String> {
        let mut config = CompositeConfig::default();
        config.receiver.connection = ConnectionProfile::CleanRf;
        Ok(Self {
            renderer: Renderer::new_composite(config, RasterLayout::atari2600(HEIGHT as u32))
                .map_err(|e| e.to_string())?,
            encoder: AtariEncoder::new(AtariProfile::default(), config.sampling)
                .map_err(|e| e.to_string())?,
            signal: SignalLineBuffer::new(),
            indexed: None,
            stamp: None,
            bytes: Vec::new(),
            size: (0, 0),
            texture: None,
        })
    }

    // A reset loses GPU storage but leaves decoded signal and the CPU image intact.
    pub(crate) fn reset_texture(&mut self) {
        self.texture = None;
        self.stamp = None;
    }

    pub(crate) fn update(
        &mut self,
        canvas: &mut Canvas<Window>,
        frame: &Frame,
    ) -> Result<bool, String> {
        let size = canvas.output_size()?;
        let aspect = (WIDTH * 2) as f64 / HEIGHT as f64;
        let width = size
            .0
            .min((f64::from(size.1) * aspect).round() as u32)
            .max(1);
        let height = size
            .1
            .min((f64::from(size.0) / aspect).round() as u32)
            .max(1);
        let size = (width, height);
        if size != self.size {
            let raster = self.renderer.raster();
            let mut view = ViewConfig::new(size.0, size.1, raster);
            view.fit = Fit::Stretch;
            view.reference_aspect = (WIDTH * 2) as f32 / HEIGHT as f32;
            self.renderer.set_view(view).map_err(|e| e.to_string())?;
            self.bytes = vec![0; size.0 as usize * size.1 as usize * 4];
            self.size = size;
            self.reset_texture();
        }

        // Compare against this renderer's last submitted picture, including time spent in simple mode.
        for (row, pixels) in frame.pixels.as_chunks::<WIDTH>().0.iter().enumerate() {
            if self
                .indexed
                .as_ref()
                .is_some_and(|old| old[row * WIDTH..(row + 1) * WIDTH] == pixels[..])
            {
                continue;
            }
            let colors = std::array::from_fn(|x| TiaColor::from_register(pixels[x]));
            self.encoder
                .encode_line(&AtariLine::new(&colors), &mut self.signal)
                .map_err(|e| e.to_string())?;
            self.renderer
                .submit_line(
                    row as u32,
                    self.signal.as_line().map_err(|e| e.to_string())?,
                )
                .map_err(|e| e.to_string())?;
        }
        self.indexed
            .get_or_insert_with(|| vec![0; WIDTH * HEIGHT])
            .copy_from_slice(&frame.pixels);
        let pitch = self.size.0 as usize * 4;
        let output =
            OutputBufferMut::new(&mut self.bytes, size.0, size.1, pitch, PixelFormat::Rgba8)
                .map_err(|e| e.to_string())?;
        let report = self
            .renderer
            .render_into(output, self.stamp)
            .map_err(|e| e.to_string())?;
        if self.texture.is_none() {
            self.texture = Some(
                canvas
                    .create_texture_streaming(PixelFormatEnum::RGBA32, size.0, size.1)
                    .map_err(|e| e.to_string())?,
            );
        }
        // Commit the stamp only after every damaged rectangle reaches GPU storage.
        self.stamp = None;
        for rect in &report.changed {
            let offset = rect.y as usize * pitch + rect.x as usize * 4;
            self.texture
                .as_mut()
                .expect("created texture")
                .update(
                    Rect::new(rect.x as i32, rect.y as i32, rect.width, rect.height),
                    &self.bytes[offset..],
                    pitch,
                )
                .map_err(|e| e.to_string())?;
        }
        self.stamp = Some(report.stamp);
        Ok(!report.changed.is_empty())
    }

    pub(crate) fn draw(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        canvas.copy(self.texture.as_ref().expect("updated texture"), None, None)
    }

    pub(crate) fn save_snapshot(&mut self, path: &Path) -> Result<(), String> {
        let surface = Surface::from_data(
            &mut self.bytes,
            self.size.0,
            self.size.1,
            self.size.0 * 4,
            PixelFormatEnum::RGBA32,
        )?;
        surface
            .convert_format(PixelFormatEnum::RGB24)?
            .save_bmp(path)
    }
}
