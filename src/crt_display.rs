//! Retained Atari signals presented directly by the native GPU or CPU backend.
use crate::video::{Frame, HEIGHT, WIDTH};
use crt::source::atari2600::{AtariEncoder, AtariLine, AtariProfile, TiaColor};
use crt::{
    CompositeConfig, ConnectionProfile, RasterLayout, Renderer, SignalLineBuffer, ViewConfig,
};
use crt_native::Display;
use sdl2::pixels::PixelFormatEnum;
use sdl2::surface::Surface;
use std::path::Path;

pub(crate) struct CrtDisplay {
    renderer: Renderer,
    encoder: AtariEncoder,
    signal: SignalLineBuffer,
    indexed: Option<Vec<u8>>,
    size: (u32, u32),
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
            size: (0, 0),
        })
    }
    pub(crate) fn update(&mut self, display: &Display, frame: &Frame) -> Result<bool, String> {
        let size = display.size();
        if size.0 == 0 || size.1 == 0 {
            return Ok(false);
        }
        let mut changed = size != self.size;
        if changed {
            let mut view = ViewConfig::new(size.0, size.1, self.renderer.raster());
            view.reference_aspect = (WIDTH * 2) as f32 / HEIGHT as f32;
            self.renderer.set_view(view).map_err(|e| e.to_string())?;
            self.size = size;
        }
        // The source comparison belongs to this CRT renderer and survives time
        // spent in simple mode. The native backend chooses immediate/deferred decode.
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
            display
                .submit_line(
                    &mut self.renderer,
                    row as u32,
                    self.signal.as_line().map_err(|e| e.to_string())?,
                )
                .map_err(|e| e.to_string())?;
            changed = true;
        }
        self.indexed
            .get_or_insert_with(|| vec![0; WIDTH * HEIGHT])
            .copy_from_slice(&frame.pixels);
        Ok(changed)
    }
    pub(crate) fn draw(&mut self, display: &mut Display, force: bool) -> Result<(), String> {
        display.present_crt(&mut self.renderer, force)
    }
    pub(crate) fn snapshot(&mut self, display: &mut Display) -> Result<Vec<u8>, String> {
        display.snapshot(&mut self.renderer)
    }
    pub(crate) fn save_snapshot(
        &mut self,
        display: &mut Display,
        path: &Path,
    ) -> Result<(), String> {
        let mut bytes = self.snapshot(display)?;
        let view = self.renderer.view();
        let surface = Surface::from_data(
            &mut bytes,
            view.width,
            view.height,
            view.width * 4,
            PixelFormatEnum::RGBA32,
        )?;
        surface
            .convert_format(PixelFormatEnum::RGB24)?
            .save_bmp(path)
    }
}
