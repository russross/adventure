//! Repeatable complete-trace and idle host performance workloads.
use adventure as port;
use port::sound::{AudioMode, EffectPlayer};
use port::validation::Validation;
use port::video::Frame;

include!("../tests/performance.rs");

fn create_sdl(cached: bool) -> Result<SdlHardware, String> {
    SdlHardware::with_audio_mode(
        true,
        None,
        if cached {
            AudioMode::Cached
        } else {
            AudioMode::Synthesized
        },
    )
}

fn cached_renderer() -> Result<Box<dyn SampleRenderer>, String> {
    Ok(Box::new(EffectPlayer::new(
        NonZeroU32::new(48_000).unwrap(),
    )))
}

// Idle cached output skips callbacks; active output copies PCM and applies transition ramps.
impl SampleRenderer for EffectPlayer {
    fn render_picture(&mut self, sound: &Sound, samples: &mut [f32]) {
        self.update(sound, false);
        if self.idle() {
            samples.fill(0.0);
        } else {
            self.render(samples);
        }
    }
}

// Adapt the public game entry and host boundary to the shared workload.
fn run_game(game: &mut Adventure, host: &mut Host) -> Result<(), String> {
    if let Some(trace) = &game.trace {
        game.validation = Some(Validation::attach(&mut trace.borrow_mut())?);
    }
    game.run(host)
}

fn present_sdl(
    host: &mut SdlHardware,
    frame: &Frame,
    sound: &Sound,
) -> Result<Option<Controls>, String> {
    host.present(frame, sound)
}

fn won(game: &Adventure) -> bool {
    game.world.won
}
