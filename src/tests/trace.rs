use std::cell::RefCell;

use adventure::adventure::Adventure;
use adventure::hardware::FrameHost;
use adventure::input::Controls;
use adventure::replay::{PlaybackHost, ReplayHost};
use adventure::sound::{Channel, Sound};
use adventure::video::Frame;
use adventure_trace::Session;

#[path = "../../tests/trace_fixture.rs"]
pub mod fixture;

#[test]
fn recorded_easy_playthrough_reaches_victory() {
    let records = fixture::easy_playthrough();
    let frames = records.last().unwrap().at.frame;
    let mut game = Adventure::default();
    game.trace = Some(RefCell::new(Session::replay(records).unwrap()));
    fixture::attach_validation(&mut game);
    game.run(&mut ReplayHost::new(frames)).unwrap();
    assert!(game.world.won);
}

// Both implementations check generated runs across games and stopping phases.
#[test]
fn traces_validate_both_implementations_in_all_games() {
    for number in 1..=3 {
        for frames in [1, 2, 3, 17, 401] {
            let (_, records) = fixture::record(number, frames, 2);
            let mut game = Adventure::default();
            game.trace = Some(RefCell::new(Session::replay(records).unwrap()));
            fixture::attach_validation(&mut game);
            game.run(&mut ReplayHost::new(frames)).unwrap();
        }
    }
}

#[test]
fn key_and_gate_trace_validates_both_implementations() {
    let (_, records) = fixture::record_castle();
    let mut game = Adventure::default();
    game.trace = Some(RefCell::new(Session::replay(records).unwrap()));
    fixture::attach_validation(&mut game);
    game.run(&mut ReplayHost::new(270)).unwrap();
}

#[test]
fn canceled_boot_trace_validates_both_implementations() {
    let mut recording = Session::record();
    recording.finish().unwrap();
    let mut game = Adventure::default();
    game.trace = Some(RefCell::new(
        Session::replay(recording.records().to_vec()).unwrap(),
    ));
    fixture::attach_validation(&mut game);
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
