use crate::*;

fn position(x: u8) -> Position {
    Position {
        room: Room::WestHall,
        x,
        y: 40,
    }
}
fn sample(machine: &mut Machine, frame: u64, x: u8) -> Option<Event> {
    machine.observe(
        frame,
        Point::Bat,
        Observation::Position(Object::Bat, position(x)),
    )
}

// Motion omissions require the same displacement and cadence, and room edges always break a segment.
#[test]
fn motion_changes_stops_and_explicit_self_room_edges() {
    let mut m = Machine::default();
    assert_eq!(
        sample(&mut m, 0, 10),
        Some(Event::Place(Object::Bat, position(10)))
    );
    assert!(matches!(
        sample(&mut m, 3, 13),
        Some(Event::Motion {
            dx: 3,
            period: 3,
            ..
        })
    ));
    assert_eq!(sample(&mut m, 6, 16), None);
    assert_eq!(sample(&mut m, 6, 16), None);
    assert!(matches!(
        sample(&mut m, 9, 15),
        Some(Event::Motion { dx: -1, .. })
    ));
    assert_eq!(
        sample(&mut m, 12, 15),
        Some(Event::Stop(Object::Bat, position(15)))
    );
    assert_eq!(
        m.observe(
            12,
            Point::Bat,
            Observation::Transition(Object::Bat, position(15))
        ),
        Some(Event::Enter(Object::Bat, position(15)))
    );
}
#[test]
fn irregular_cadence_and_wrapping_remain_visible() {
    let mut m = Machine::default();
    sample(&mut m, 0, 254);
    assert!(matches!(
        sample(&mut m, 3, 1),
        Some(Event::Motion { dx: 3, .. })
    ));
    assert!(matches!(
        sample(&mut m, 7, 4),
        Some(Event::Motion { period: 4, .. })
    ));
    let other_room = Position {
        room: Room::RedHall,
        ..position(7)
    };
    assert_eq!(
        m.observe(
            11,
            Point::Bat,
            Observation::Position(Object::Bat, other_room)
        ),
        Some(Event::Enter(Object::Bat, other_room))
    );
}

// Counter stalls and accelerated progression cannot disappear behind animation compression.
#[test]
fn counter_prediction_checks_value_and_time() {
    let mut m = Machine::default();
    let counter =
        |value| Observation::State(Object::Bat, State::Counter(Counter::Animation, value));
    assert!(m.observe(0, Point::Bat, counter(0)).is_some());
    assert!(m.observe(3, Point::Bat, counter(1)).is_some());
    assert!(m.observe(6, Point::Bat, counter(2)).is_none());
    assert!(m.observe(9, Point::Bat, counter(2)).is_some());
    assert!(m.observe(12, Point::Bat, counter(4)).is_some());
    assert!(m.observe(16, Point::Bat, counter(5)).is_some());
}
#[test]
fn text_roundtrip_and_strict_validation() {
    let examples = [
        "00:00:00/boot#0 trace 1 adventure-original boot",
        "125:04:59/bat#0 motion bat west-hall 3,40 step=-3,3 period=3",
        "00:00:01/pickup#0 state player carry gold-key 8,255",
        "00:00:02/gates#0 state gold-gate mode opening",
        "00:00:03/player-both#0 contact player walls",
        "00:00:04/bat#0 state bat counter flight 16",
        "00:00:05/end#0 end",
    ];
    for line in examples {
        assert_eq!(line.parse::<Record>().unwrap().to_string(), line);
    }
    for line in [
        "00:60:00/boot#0 end",
        "00:00:60/boot#0 end",
        "00:00:00/boot#0 input reset 2",
        "00:00:00/boot#0 input fire 0 extra",
        "00:00:00/boot#0 place unknown west-hall 1,2",
        "00:00:00/boot#0 place bat west-hall 256,2",
        "00:00:00/boot#0 trace 2 adventure-original boot",
        "00:00:00/boot#0 motion bat west-hall 1,2 step=1,0 period=0",
    ] {
        assert!(line.parse::<Record>().is_err(), "{line}");
    }
}

#[test]
fn replay_checks_inputs_at_exact_semantic_reads() {
    let mut record = Session::record();
    record.at(Point::Console);
    record.input(Control::Reset, 1);
    record.next_frame();
    record.at(Point::Bat);
    record.observe(Observation::Position(Object::Bat, position(10)));
    record.finish().unwrap();
    let records = record.records().to_vec();
    let mut replay = Session::replay(records.clone()).unwrap();
    replay.at(Point::Console);
    assert_eq!(replay.input(Control::Reset, 0), 1);
    replay.next_frame();
    replay.at(Point::Bat);
    replay.observe(Observation::Position(Object::Bat, position(10)));
    replay.finish().unwrap();
    let mut replay = Session::replay(records).unwrap();
    replay.at(Point::Pickup);
    replay.input(Control::Reset, 0);
    assert!(replay.check().unwrap_err().contains("trace divergence"));
}
#[test]
fn input_changes_are_per_control_and_replay_at_the_recorded_read() {
    let mut record = Session::record();
    record.at(Point::Console);
    record.input(Control::Reset, 1);
    record.input(Control::Select, 0);
    record.input(Control::Reset, 1);
    record.at(Point::History);
    record.input(Control::Reset, 1);
    record.next_frame();
    record.at(Point::Console);
    record.input(Control::Reset, 1);
    record.at(Point::History);
    record.input(Control::Reset, 0);
    record.input(Control::Reset, 0);
    record.input(Control::Reset, 1);
    record.finish().unwrap();
    let inputs: Vec<_> = record
        .records()
        .iter()
        .filter_map(|r| match r.event {
            Event::Input(control, value) => Some((r.at, control, value)),
            _ => None,
        })
        .collect();
    assert_eq!(inputs.len(), 4);
    assert_eq!(inputs[2].0.ordinal, 0);
    assert_eq!(inputs[3].0.ordinal, 1);

    // Unchanged reads retain the supplied value across points and frames despite different live inputs.
    let mut replay = Session::replay(record.records().to_vec()).unwrap();
    replay.at(Point::Console);
    assert_eq!(replay.input(Control::Reset, 0), 1);
    assert_eq!(replay.input(Control::Select, 1), 0);
    assert_eq!(replay.input(Control::Reset, 0), 1);
    replay.at(Point::History);
    assert_eq!(replay.input(Control::Reset, 0), 1);
    replay.next_frame();
    replay.at(Point::Console);
    assert_eq!(replay.input(Control::Reset, 0), 1);
    replay.at(Point::History);
    assert_eq!(replay.input(Control::Reset, 1), 0);
    assert_eq!(replay.input(Control::Reset, 0), 1);
    replay.finish().unwrap();
}

#[test]
fn malformed_lifecycle_is_rejected() {
    let mut s = Session::record();
    s.finish().unwrap();
    let mut records = s.records().to_vec();
    records[1].at.ordinal = 1;
    assert!(Session::replay(records).is_err());
    assert!(Session::replay(vec![]).is_err());
}

// Separate room and coordinate stores cannot expose intermediate positions at a commit boundary.
#[test]
fn component_stores_commit_complete_objects() {
    let mut assembly = PositionAssembly::default();
    assembly.write(Object::Bat, PositionField::Room(Room::WestHall));
    assert!(assembly.commit(&[Object::Bat]).is_err());
    assembly.write(Object::Bat, PositionField::X(10));
    assembly.write(Object::Bat, PositionField::Y(40));
    assert_eq!(
        assembly.commit(&[Object::Bat]).unwrap(),
        [Observation::Position(Object::Bat, position(10))]
    );
    assembly.write(Object::Bat, PositionField::Y(32));
    assembly.write(Object::Bat, PositionField::Y(40));
    assert_eq!(
        assembly.commit(&[Object::Bat]).unwrap(),
        [Observation::Position(Object::Bat, position(10))]
    );
}

#[test]
fn writer_failures_are_reported() {
    struct Broken;
    impl std::io::Write for Broken {
        fn write(&mut self, _bytes: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("broken sink"))
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    assert!(
        Session::record_to(Broken)
            .err()
            .unwrap()
            .contains("broken sink")
    );
    let mut session = Session::record();
    session.finish().unwrap();
    assert!(session.finish().is_err());
}

#[test]
fn repeated_key_contact_accelerates_gate_prediction_once_per_update() {
    let mut machine = Machine::default();
    let counter =
        |value| Observation::State(Object::GoldGate, State::Counter(Counter::Gate, value));
    machine.observe(
        0,
        Point::Gates,
        Observation::State(Object::GoldGate, State::Mode(Mode::Opening)),
    );
    machine.observe(0, Point::Gates, counter(29));
    for (frame, value) in [(3, 31), (6, 33)] {
        for _ in 0..3 {
            machine.observe(
                frame,
                Point::Gates,
                Observation::Contact(Object::GoldKey, ContactTarget::Object(Object::GoldGate)),
            );
        }
        let event = machine.observe(frame, Point::Gates, counter(value));
        assert_eq!(event.is_none(), frame == 6);
    }
    assert!(machine.observe(9, Point::Gates, counter(33)).is_some());
}

#[test]
fn room_crossing_reanchors_continued_momentum() {
    let mut machine = Machine::default();
    sample(&mut machine, 0, 149);
    sample(&mut machine, 3, 152);
    assert_eq!(
        machine.observe(
            6,
            Point::Bat,
            Observation::Transition(Object::Bat, position(3))
        ),
        Some(Event::Enter(Object::Bat, position(3)))
    );
    assert_eq!(sample(&mut machine, 9, 6), None);
    assert_eq!(
        sample(&mut machine, 12, 6),
        Some(Event::Stop(Object::Bat, position(6)))
    );
}
