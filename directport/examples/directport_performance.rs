//! Repeatable complete-trace and idle host performance workloads.
use directport as port;
use port::tia::Frame;

include!("../../tests/performance.rs");
implement_host!(vertical_sync);

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
    game.cartridge_entry(host)
}

fn present_sdl(
    host: &mut SdlHardware,
    frame: &Frame,
    sound: &Sound,
) -> Result<Option<Controls>, String> {
    host.vertical_sync(frame, sound)
}

fn won(game: &Adventure) -> bool {
    game.ram.won
}
