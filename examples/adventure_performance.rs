//! Repeatable complete-trace and idle host performance workloads.
use adventure as port;
use port::validation::Validation;
use port::video::Frame;

include!("../tests/performance.rs");

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
