use std::cell::RefCell;

use adventure_refactored::adventure::Adventure;
use adventure_refactored::replay::ReplayHost;
use adventure_trace::Session;

#[path = "../../tests/trace_fixture.rs"]
pub mod fixture;

// The refactor consumes translation inputs and independently produces every expected state record.
#[test]
fn translation_traces_validate_in_refactored_game() {
    for number in 1..=3 {
        for frames in [1, 2, 3, 17, 401] {
            let (original, records) = fixture::record(number, frames, 2);
            let mut game = Adventure::default();
            game.trace = Some(RefCell::new(Session::replay(records.clone()).unwrap()));
            game.run(&mut ReplayHost::new(frames)).unwrap();
            assert_eq!(game.world.player.x, original.ram.player.x);
            assert_eq!(game.world.player.y, original.ram.player.y);
            assert_eq!(game.world.player.room as u8, original.ram.player.room as u8);
        }
    }
}

#[test]
fn key_and_gate_trace_validates_in_refactor() {
    let (_, records) = fixture::record_castle();
    let mut game = Adventure::default();
    game.trace = Some(RefCell::new(Session::replay(records).unwrap()));
    game.run(&mut ReplayHost::new(270)).unwrap();
}

#[test]
fn canceled_boot_trace_validates_in_refactor() {
    let mut recording = Session::record();
    recording.finish().unwrap();
    let mut game = Adventure::default();
    game.trace = Some(RefCell::new(
        Session::replay(recording.records().to_vec()).unwrap(),
    ));
    game.run(&mut ReplayHost::new(0)).unwrap();
}
