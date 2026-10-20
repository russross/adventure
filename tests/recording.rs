use std::cell::RefCell;
use std::io::Cursor;
use std::io::Write;
use std::rc::Rc;

use adventure::adventure::Adventure;
use adventure::input::Controls;
use adventure::recording::Recording;
use adventure::replay::{PlaybackHost, ReplayHost};
use adventure::runtime::FrameHost;
use adventure::sound::{Channel, Sound};
use adventure::video::Frame;
use adventure_trace::Control;

#[test]
fn easy_recording_reaches_victory_at_its_declared_endpoint() {
    let recording = Recording::replay(Cursor::new(include_str!("easy.record"))).unwrap();
    let mut host = ReplayHost::new(recording.frames());
    let mut game = Adventure::default();
    game.session.recording = Some(RefCell::new(recording));
    game.run(&mut host).unwrap();
    assert!(game.world.won);
}

#[derive(Clone, Default)]
struct Bytes(Rc<RefCell<Vec<u8>>>);
impl Write for Bytes {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.borrow_mut().extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

struct Host {
    limit: usize,
    pictures: Vec<(Vec<u8>, [Channel; 2])>,
}
// Borrow the capture host so its presented output remains available after playback.

impl FrameHost for Host {
    // Exercise held controls, releases, selection, Game 3 initialization, and both axes.
    fn sample_controls(&mut self) -> Result<Option<Controls>, String> {
        let n = self.pictures.len();
        let mut controls = Controls {
            select: matches!(n, 2..=4 | 8..=10),
            reset: matches!(n, 15..=18),
            color: n < 70,
            ..Controls::default()
        };
        controls.left.direction.up = (20..50).contains(&n);
        controls.left.direction.right = (40..90).contains(&n);
        controls.left.fire = (60..65).contains(&n);
        Ok(Some(controls))
    }
    fn present(&mut self, frame: &Frame, sound: &Sound) -> Result<Option<Controls>, String> {
        if self.pictures.len() == self.limit {
            return Ok(None);
        }
        self.pictures
            .push((frame.pixels().to_vec(), sound.channels));
        self.sample_controls()
    }
}

#[test]
fn compact_recording_preserves_presentations_and_replays_without_live_input() {
    for limit in [0, 1, 2, 3, 17, 120] {
        let mut expected = Host {
            limit,
            pictures: Vec::new(),
        };
        let mut original = Adventure::default();
        original.run(&mut expected).unwrap();
        let bytes = Bytes::default();
        let mut captured = Host {
            limit,
            pictures: Vec::new(),
        };
        let mut recording = Adventure::default();
        recording.session.recording =
            Some(RefCell::new(Recording::record_to(bytes.clone()).unwrap()));
        recording.run(&mut captured).unwrap();
        assert_eq!(captured.pictures, expected.pictures);
        let session = Recording::replay(Cursor::new(bytes.0.borrow().clone())).unwrap();
        assert_eq!(session.frames(), limit as u64);
        let mut replay = Adventure::default();
        replay.session.recording = Some(RefCell::new(session));
        let mut actual = Host {
            limit: usize::MAX,
            pictures: Vec::new(),
        };
        replay
            .run(&mut PlaybackHost::new(&mut actual, limit as u64))
            .unwrap();
        assert_eq!(actual.pictures, expected.pictures);
        assert_eq!(replay.world.player, original.world.player);
    }
}

#[test]
fn only_changed_nondefault_observed_values_are_saved() {
    let bytes = Bytes::default();
    let mut recording = Recording::record_to(bytes.clone()).unwrap();
    recording.input(Control::Color, 1);
    recording.input(Control::Reset, 0);
    recording.input(Control::Reset, 1);
    recording.input(Control::Reset, 1);
    recording.input(Control::Reset, 0);
    recording.finish().unwrap();
    assert_eq!(
        String::from_utf8(bytes.0.borrow().clone()).unwrap(),
        "adventure-input 1\ninput 2 reset 1\ninput 4 reset 0\nend 0 5\n"
    );
}

#[test]
fn malformed_recordings_are_rejected() {
    for text in [
        "",
        "adventure-input 2\nend 0 0\n",
        "adventure-input 1\n",
        "adventure-input 1\ninput 0 fire 2\nend 1 1\n",
        "adventure-input 1\ninput 2 reset 1\ninput 1 reset 0\nend 1 3\n",
        "adventure-input 1\ninput 1 reset 1\nend 1 1\n",
        "adventure-input 1\nend 0 0\nend 0 0\n",
    ] {
        assert!(Recording::replay(Cursor::new(text)).is_err(), "{text}");
    }
}
