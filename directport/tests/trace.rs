use std::cell::RefCell;

use adventure_trace::{Event, Session};
use directport::adventure::{Adventure, Game, ObjectId, Room};
use directport::hardware::FrameHost;
use directport::input::{Controls, Direction};
use directport::sound::Sound;
use directport::tia::Frame;

#[path = "../../tests/trace_fixture.rs"]
pub mod fixture;
use fixture::{record, replay};

// Exercise switch edges, random initialization, controls, movement, and bounded termination at all picture phases.
#[test]
fn all_games_roundtrip_with_only_consumed_inputs() {
    for number in 1..=3 {
        for frames in [1, 2, 3, 17, 31, 400, 401, 402] {
            let (original, records) = record(number, frames, 2);
            let verified = replay(records).unwrap();
            assert_eq!(original.ram.player, verified.ram.player);
            assert_eq!(original.ram.bat, verified.ram.bat);
            assert_eq!(original.ram.held, verified.ram.held);
            if frames > 30 {
                assert_eq!(
                    original.ram.game,
                    match number {
                        1 => Game::One,
                        2 => Game::Two,
                        _ => Game::Three,
                    }
                );
            }
        }
    }
}
#[test]
fn corrupted_state_and_extra_input_fail_replay() {
    let (_, mut records) = record(1, 90, 0);
    let r = records
        .iter_mut()
        .find(|r| matches!(r.event, Event::Place(..)))
        .unwrap();
    if let Event::Place(_, p) = &mut r.event {
        p.x = p.x.wrapping_add(1);
    }
    assert!(replay(records).err().unwrap().contains("trace divergence"));
    let (_, mut records) = record(1, 90, 0);
    let r = records
        .iter_mut()
        .find(|r| matches!(r.event, Event::Input(..)))
        .unwrap();
    r.at.point = adventure_trace::Point::Pickup;
    assert!(replay(records).is_err());
}

// Idle input changes that the game never consumes must not alter its semantic trace.
#[test]
fn unused_right_fire_is_not_serialized() {
    let (game, records) = record(1, 90, 0);
    assert!(records.iter().any(|r| matches!(
        r.event,
        Event::Input(adventure_trace::Control::RightDirection, _)
    )));
    let replayed = replay(records).unwrap();
    assert_eq!(game.ram.player, replayed.ram.player);
    assert!(!replayed.controls.right.fire);
}

#[test]
fn gold_key_pickup_roundtrips() {
    struct PickupHost(u64);
    impl FrameHost for PickupHost {
        fn sample_controls(&mut self) -> Result<Option<Controls>, String> {
            Ok(Some(Controls::default()))
        }
        fn vertical_sync(
            &mut self,
            _frame: &Frame,
            _sound: &Sound,
        ) -> Result<Option<Controls>, String> {
            if self.0 == 105 {
                return Ok(None);
            }
            let f = self.0;
            self.0 += 1;
            Ok(Some(Controls {
                reset: f < 3,
                left: directport::input::Joystick {
                    direction: Direction {
                        left: (6..54).contains(&f),
                        up: (57..90).contains(&f),
                        ..Direction::default()
                    },
                    fire: false,
                },
                ..Controls::default()
            }))
        }
    }
    let mut game = Adventure {
        trace: Some(RefCell::new(Session::record())),
        ..Adventure::default()
    };
    game.cartridge_entry(&mut PickupHost(0)).unwrap();
    assert_eq!(game.ram.player.room, Room::GoldCastle);
    assert_eq!(game.ram.held, ObjectId::GoldKey);
    let records = game.trace.as_ref().unwrap().borrow().records().to_vec();
    let replayed = replay(records).unwrap();
    assert_eq!(replayed.ram.held, ObjectId::GoldKey);
}

#[test]
fn quitting_before_initial_control_sample_roundtrips() {
    struct Quit;
    impl FrameHost for Quit {
        fn sample_controls(&mut self) -> Result<Option<Controls>, String> {
            Ok(None)
        }
        fn vertical_sync(
            &mut self,
            _frame: &Frame,
            _sound: &Sound,
        ) -> Result<Option<Controls>, String> {
            Err("unexpected presentation".into())
        }
    }
    let mut game = Adventure {
        trace: Some(RefCell::new(Session::record())),
        ..Adventure::default()
    };
    game.cartridge_entry(&mut Quit).unwrap();
    let records = game.trace.as_ref().unwrap().borrow().records().to_vec();
    assert_eq!(records.len(), 2);
    replay(records).unwrap();
}
