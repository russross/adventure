//! Repeatable complete-trace and idle host performance workloads.
use port::video::Frame;
use scanline as port;

include!("../../tests/performance.rs");
implement_host!(present);

fn create_sdl(cached: bool) -> Result<SdlHardware, String> {
    if cached {
        return Err("cached audio is only available in frame".into());
    }
    SdlHardware::new(true, None)
}

fn cached_renderer() -> Result<Box<dyn SampleRenderer>, String> {
    Err("cached audio is only available in frame".into())
}

// Adapt the public game entry and host boundary to the shared workload.
fn run_game(game: &mut Adventure, host: &mut Host) -> Result<(), String> {
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
