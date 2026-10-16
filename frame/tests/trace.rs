use std::cell::RefCell;

use adventure_trace::Session;
use frame::adventure::Adventure;
use frame::hardware::FrameHost;
use frame::input::Controls;
use frame::replay::{PlaybackHost, ReplayHost};
use frame::sound::{Channel, Sound};
use frame::video::Frame;

#[path = "../../tests/trace_fixture.rs"]
pub mod fixture;

#[test]
fn recorded_easy_playthrough_reaches_victory() {
    let records = fixture::easy_playthrough();
    let frames = records.last().unwrap().at.frame;
    let mut game = Adventure::default();
    game.trace = Some(RefCell::new(Session::replay(records).unwrap()));
    game.run(&mut ReplayHost::new(frames)).unwrap();
    assert!(game.world.won);
}

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

// Capture presentation independently of trace validation, including audio controls.
struct CaptureHost<'a> {
    pictures: &'a mut Vec<(Vec<u8>, [Channel; 2])>,
    controls: Controls,
    limit: usize,
}

impl FrameHost for CaptureHost<'_> {
    fn sample_controls(&mut self) -> Result<Option<Controls>, String> {
        Ok(Some(self.controls))
    }

    fn present(&mut self, frame: &Frame, sound: &Sound) -> Result<Option<Controls>, String> {
        if self.pictures.len() == self.limit {
            return Ok(None);
        }
        self.pictures.push((frame.pixels.to_vec(), sound.channels));
        self.sample_controls()
    }
}

#[test]
fn playback_presents_entire_run_and_ignores_live_gameplay_controls() {
    for number in 1..=3 {
        for frames in [1, 2, 3, 17, 401] {
            let (_, records) = fixture::record(number, frames, 2);
            let mut expected = Vec::new();
            let mut reference = Adventure::default();
            reference.trace = Some(RefCell::new(Session::replay(records.clone()).unwrap()));
            reference
                .run(&mut CaptureHost {
                    pictures: &mut expected,
                    controls: Controls::default(),
                    limit: frames as usize,
                })
                .unwrap();

            // Hold console switches and motion throughout playback; the recording wins.
            let mut actual = Vec::new();
            let mut controls = Controls {
                reset: true,
                select: true,
                color: false,
                ..Controls::default()
            };
            controls.left.direction.right = true;
            controls.left.fire = true;
            let mut host = PlaybackHost::new(
                CaptureHost {
                    pictures: &mut actual,
                    controls,
                    limit: usize::MAX,
                },
                frames,
            );
            let mut game = Adventure::default();
            game.trace = Some(RefCell::new(Session::replay(records).unwrap()));
            game.run(&mut host).unwrap();
            assert!(!host.interrupted());
            assert_eq!(actual.len(), frames as usize);
            assert_eq!(actual, expected);
        }
    }
}

#[test]
fn playback_distinguishes_early_quit_from_recorded_end() {
    let (_, records) = fixture::record(1, 17, 2);
    let mut pictures = Vec::new();
    let mut host = PlaybackHost::new(
        CaptureHost {
            pictures: &mut pictures,
            controls: Controls::default(),
            limit: 2,
        },
        17,
    );
    let mut game = Adventure::default();
    game.trace = Some(RefCell::new(Session::replay(records).unwrap()));
    assert!(game.run(&mut host).is_err());
    assert!(host.interrupted());
    assert_eq!(pictures.len(), 2);
}
