use super::*;
use crate::tia::{Contacts, Frame, HEIGHT};

#[path = "rom_oracle.rs"]
mod oracle;
use oracle::Cpu;

#[derive(Default)]
struct Host {
    controls: Controls,
    lines: Vec<usize>,
}

impl FrameHost for Host {
    fn sample_controls(&mut self) -> Result<Option<Controls>, String> {
        Ok(Some(self.controls))
    }

    fn vertical_sync(&mut self, frame: &Frame, _sound: &Sound) -> Result<Option<Controls>, String> {
        self.lines.push(frame.painted_lines);
        Ok(Some(self.controls))
    }
}

const RECORDS: [(usize, RecordId); 14] = [
    (0x8a, RecordId::Player),
    (0xa1, RecordId::Dot),
    (0xa4, RecordId::Rhindle),
    (0xa9, RecordId::Yorgle),
    (0xae, RecordId::Grundle),
    (0xb3, RecordId::Magnet),
    (0xb6, RecordId::Sword),
    (0xb9, RecordId::Chalice),
    (0xbc, RecordId::Bridge),
    (0xbf, RecordId::GoldKey),
    (0xc2, RecordId::WhiteKey),
    (0xc5, RecordId::BlackKey),
    (0xcb, RecordId::Bat),
    (0xd9, RecordId::Light),
];

fn direction_byte(direction: Direction) -> u8 {
    let mut value = 255;
    for (pressed, bit) in [
        (direction.right, 128),
        (direction.left, 64),
        (direction.up, 16),
        (direction.down, 32),
    ] {
        if pressed {
            value &= !bit;
        }
    }
    value
}

fn initialized(game: Game) -> Adventure {
    let mut adventure = Adventure::default();
    adventure.ram.game = game;
    assert!(adventure.initialize_world(&mut Host::default()).unwrap());
    adventure
}

// Serialization belongs only to the reference comparison, never to the running port.
fn reference(game: &Adventure) -> Cpu {
    let mut cpu = Cpu::new();
    for (address, record) in RECORDS {
        let position = game.ram.position(record);
        cpu.memory[address..address + 3].copy_from_slice(&[
            position.room as u8,
            position.x,
            position.y,
        ]);
    }
    for (address, dragon) in [
        (0xa4, game.ram.rhindle),
        (0xa9, game.ram.yorgle),
        (0xae, game.ram.grundle),
    ] {
        cpu.memory[address + 3] = direction_byte(dragon.direction);
        cpu.memory[address + 4] = dragon.state.value();
    }
    cpu.memory[0xc8] = game.ram.gates.gold.value();
    cpu.memory[0xc9] = game.ram.gates.white.value();
    cpu.memory[0xca] = game.ram.gates.black.value();
    cpu.memory[0xce] = direction_byte(game.ram.bat.direction);
    cpu.memory[0xcf] = game.ram.bat.animation;
    cpu.memory[0xd0] = game.ram.bat.carried.map_or(0x78, |record| {
        RECORDS.iter().find(|(_, r)| *r == record).unwrap().0 as u8
    });
    cpu.memory[0xd1] = game.ram.bat.flight_timer;
    cpu.memory[0x95] = game.ram.first_visible as u8 * 9;
    cpu.memory[0x96] = game.ram.second_visible as u8 * 9;
    cpu.memory[0x9c] = game.ram.last_candidate as u8 * 9;
    cpu.memory[0x9d] = game.ram.held as u8 * 9;
    cpu.memory[0x9e] = game.ram.held_offset_x;
    cpu.memory[0x9f] = game.ram.held_offset_y;
    cpu.memory[0x99] = direction_byte(game.ram.joystick_direction);
    cpu.memory[0xdd] = game.ram.game.index() as u8 * 2;
    cpu.memory[0xde] = if game.ram.won { 255 } else { 0 };
    cpu.memory[0xdf] = game.ram.sound_remaining;
    cpu.memory[0xe0] = game.ram.sound_effect as u8;
    cpu.memory[0xe2..0xe5].copy_from_slice(&[
        game.ram.safe_player.room as u8,
        game.ram.safe_player.x,
        game.ram.safe_player.y,
    ]);
    cpu.memory[0xe5] = game.ram.idle_ticks;
    cpu.memory[0xe6] = game.ram.idle_periods;
    cpu.memory[0x92] =
        u8::from(!game.ram.previous_reset) | u8::from(!game.ram.previous_select) << 1;
    cpu.memory[0xd7] = if game.ram.fire_was_pressed { 0 } else { 128 };

    // Inject controller and pixel-contact observations into the original hardware read addresses.
    cpu.memory[0x280] = direction_byte(game.controls.left.direction) & 0xf0
        | direction_byte(game.controls.right.direction) >> 4;
    cpu.memory[0x282] = 0x34
        | u8::from(!game.controls.reset)
        | u8::from(!game.controls.select) << 1
        | u8::from(game.controls.color) << 3
        | u8::from(game.controls.left_difficulty == Difficulty::A) << 6
        | u8::from(game.controls.right_difficulty == Difficulty::A) << 7;
    cpu.memory[0x3c] = if game.controls.left.fire { 0 } else { 128 };
    cpu.memory[0x32] = u8::from(game.tia.contacts.player_objects[0]) << 6;
    cpu.memory[0x33] = u8::from(game.tia.contacts.player_objects[1]) << 6;
    cpu.memory[0x34] = u8::from(game.tia.contacts.player_barriers[0]) << 6;
    cpu.memory[0x35] = u8::from(game.tia.contacts.player_barriers[1]) << 6;
    cpu.memory[0x36] = u8::from(game.tia.contacts.player_walls) << 7;
    cpu.memory[0x37] = u8::from(game.tia.contacts.objects_touch) << 7;
    cpu
}

fn assert_state(game: &Adventure, cpu: &Cpu, context: &str) {
    let translated = reference(game);
    for (address, _) in RECORDS {
        assert_eq!(
            &translated.memory[address..address + 3],
            &cpu.memory[address..address + 3],
            "{context}: position {address:02x}"
        );
    }
    for address in [0xa7, 0xac, 0xb1, 0xce, 0x99] {
        assert_eq!(
            translated.memory[address] & 0xf0,
            cpu.memory[address] & 0xf0,
            "{context}: direction {address:02x}"
        );
    }
    for address in [
        0xa8, 0xad, 0xb2, 0xc8, 0xc9, 0xca, 0xcf, 0xd0, 0xd1, 0x95, 0x96, 0x9c, 0x9d, 0x9e, 0x9f,
        0xdd, 0xde, 0xdf, 0xe0, 0xe2, 0xe3, 0xe4, 0xe5, 0xe6,
    ] {
        assert_eq!(
            translated.memory[address], cpu.memory[address],
            "{context}: state {address:02x}"
        );
    }
    assert_eq!(
        translated.memory[0x92] & 3,
        cpu.memory[0x92] & 3,
        "{context}: console history"
    );
    assert_eq!(
        translated.memory[0xd7] & 128,
        cpu.memory[0xd7] & 128,
        "{context}: fire history"
    );
}

#[test]
fn initialization_and_all_game_three_seeds_match_rom() {
    for game in [Game::One, Game::Two, Game::Three] {
        let mut port = Adventure::default();
        port.ram.game = game;
        let mut cpu = reference(&port);
        cpu.call(0xf3d3);
        port.initialize_world(&mut Host::default()).unwrap();
        assert_state(&port, &cpu, "initialization");
    }
    for seed in 0..=255 {
        let mut port = initialized(Game::Two);
        port.ram.idle_ticks = seed;
        let mut cpu = reference(&port);
        cpu.call(0xf412);
        port.randomize_game_three();
        assert_state(&port, &cpu, &format!("random seed {seed}"));
    }
}

#[test]
fn boundaries_castle_exits_and_secret_room_match_rom() {
    for game in [Game::One, Game::Two, Game::Three] {
        for room in ROOM_IDS {
            for record in [RecordId::Player, RecordId::Bridge] {
                for (x, y) in [
                    (0, 0),
                    (255, 255),
                    (2, 13),
                    (3, 13),
                    (155, 13),
                    (159, 105),
                    (80, 106),
                    (80, 255),
                    (240, 50),
                    (100, 200),
                ] {
                    let mut port = initialized(game);
                    *port.ram.position_mut(record) = Position::new(room, x, y);
                    port.ram.dot.room = Room::SecretPassage;
                    port.ram.gates = Gates {
                        gold: GateState::Open,
                        white: GateState::Open,
                        black: GateState::Open,
                    };
                    let mut cpu = reference(&port);
                    cpu.a = 255;
                    cpu.x = RECORDS.iter().find(|(_, r)| *r == record).unwrap().0 as u8;
                    cpu.y = 0;
                    cpu.call(0xf5ff);
                    port.move_object_through_world(record, Direction::default(), 0);
                    assert_state(
                        &port,
                        &cpu,
                        &format!("boundary {game:?} {room:?} {record:?} {x}/{y}"),
                    );
                }
            }
        }
    }
}

#[test]
fn bridge_rollback_pickup_and_carried_offsets_match_rom() {
    for phase in [
        MovementPhase::Both,
        MovementPhase::Vertical,
        MovementPhase::Horizontal,
    ] {
        for dx in [0, 9, 10, 22, 23, 255] {
            for dy in [0, 24, 25, 251, 252, 255] {
                let mut port = initialized(Game::Two);
                port.ram.player = Position::new(Room::SecretPassage, 60u8.wrapping_add(dx), 50);
                port.ram.safe_player = Position::new(Room::SecretPassage, 64, 50);
                port.ram.bridge = Position::new(Room::SecretPassage, 60, 50u8.wrapping_add(dy));
                port.tia.contacts.player_walls = true;
                port.controls.left.direction.right = true;
                port.ram.joystick_direction = port.controls.left.direction;
                let mut cpu = reference(&port);
                cpu.y = match phase {
                    MovementPhase::Both => 0,
                    MovementPhase::Vertical => 1,
                    MovementPhase::Horizontal => 2,
                };
                cpu.call(0xf4c2);
                port.move_player(phase);
                assert_state(&port, &cpu, "bridge rollback");
            }
        }
    }
    for held in [ObjectId::Empty, ObjectId::Sword] {
        let mut port = initialized(Game::One);
        port.ram.player = Position::new(Room::WestHall, 50, 50);
        port.ram.sword = port.ram.player;
        port.ram.first_visible = ObjectId::Sword;
        port.ram.held = held;
        port.ram.joystick_direction.right = true;
        port.tia.contacts.player_objects[0] = true;
        let mut cpu = reference(&port);
        cpu.call(0xf556);
        port.handle_pickup_and_drop();
        assert_state(&port, &cpu, "pickup");
        let mut cpu = reference(&port);
        cpu.call(0xf5d4);
        port.move_held_object();
        assert_state(&port, &cpu, "carried offset");
    }
}

#[test]
fn dragon_state_machine_matches_rom_across_games_and_difficulties() {
    for game in [Game::One, Game::Two, Game::Three] {
        for dragon in [DragonId::Rhindle, DragonId::Yorgle, DragonId::Grundle] {
            for state in [
                DragonState::Wandering,
                DragonState::Dead,
                DragonState::Swallowed,
                DragonState::Biting(232),
                DragonState::Biting(251),
                DragonState::Biting(255),
            ] {
                for bits in 0..16 {
                    let mut port = initialized(game);
                    port.controls.left_difficulty = if bits & 1 != 0 {
                        Difficulty::A
                    } else {
                        Difficulty::B
                    };
                    port.controls.right_difficulty = if bits & 2 != 0 {
                        Difficulty::A
                    } else {
                        Difficulty::B
                    };
                    port.ram.player = Position::new(Room::WestHall, 70, 50);
                    port.ram.safe_player = port.ram.player;
                    port.ram.sword = Position::new(Room::WestHall, 60, 50);
                    port.ram.dragon_mut(dragon).position = Position::new(Room::WestHall, 80, 50);
                    port.ram.dragon_mut(dragon).state = state;
                    port.ram.first_visible = dragon.object();
                    port.ram.second_visible = ObjectId::Sword;
                    port.tia.contacts = Contacts {
                        player_objects: [bits & 4 != 0, false],
                        objects_touch: bits & 8 != 0,
                        ..Contacts::default()
                    };
                    let mut cpu = reference(&port);
                    let (entry, list, speed) = match dragon {
                        DragonId::Rhindle => (0xf795, &RHINDLE_REACTIONS[..], 3),
                        DragonId::Yorgle => (0xf7b0, &YORGLE_REACTIONS[..], 2),
                        DragonId::Grundle => (0xf7cb, &GRUNDLE_REACTIONS[..], 2),
                    };
                    cpu.call(entry);
                    port.update_dragon(dragon, list, speed);
                    assert_state(
                        &port,
                        &cpu,
                        &format!("dragon {game:?} {dragon:?} {state:?} {bits}"),
                    );
                }
            }
        }
    }
}

#[test]
fn bat_gates_magnet_and_darkness_match_rom() {
    for timer in [0, 1, 16, 255] {
        let mut port = initialized(Game::Two);
        port.ram.bat.position = Position::new(Room::WestHall, 60, 50);
        port.ram.bat.flight_timer = timer;
        port.ram.chalice = Position::new(Room::WestHall, 61, 50);
        port.ram.held = ObjectId::Chalice;
        let mut cpu = reference(&port);
        cpu.call(0xf8a5);
        port.update_bat();
        assert_state(&port, &cpu, "bat");
    }
    for value in [0, 1, 2, 27, 28, 29, 54, 55] {
        let mut port = initialized(Game::One);
        port.ram.gates.gold = GateState::from_value(value);
        port.ram.first_visible = ObjectId::GoldGate;
        port.ram.second_visible = ObjectId::GoldKey;
        port.tia.contacts.objects_touch = true;
        port.tia.contacts.player_objects[0] = true;
        let mut cpu = reference(&port);
        cpu.call(0xf93c);
        port.update_castle_gates();
        assert_state(&port, &cpu, "gate");
    }
    let mut port = initialized(Game::Two);
    port.ram.magnet = Position::new(Room::WestHall, 70, 70);
    port.ram.gold_key = Position::new(Room::WestHall, 60, 50);
    let mut cpu = reference(&port);
    cpu.call(0xf9b3);
    port.update_magnet();
    assert_state(&port, &cpu, "magnet");
    for room in ROOM_IDS {
        for x in [0, 13, 14, 129, 140, 159, 255] {
            port.ram.player = Position::new(room, x, 50);
            let mut cpu = reference(&port);
            cpu.call(0xf9e7);
            port.update_dark_room_light();
            assert_state(&port, &cpu, "darkness");
        }
    }
}

#[test]
fn visible_selection_and_every_object_image_range_match_rom() {
    let rom = include_bytes!("../adventure.bin");
    for room in ROOM_IDS {
        let mut port = initialized(Game::Two);
        port.ram.player.room = room;
        for _ in 0..20 {
            let mut cpu = reference(&port);
            cpu.call(0xf10f);
            port.prepare_room_display();
            assert_state(&port, &cpu, "display selection");
            assert_eq!(
                [
                    port.ram.display.first_position.x,
                    port.ram.display.first_position.y,
                    port.ram.display.second_position.x,
                    port.ram.display.second_position.y
                ],
                cpu.memory[0x86..0x8a]
            );
            assert_eq!(
                [port.tia.first_object.color, port.tia.second_object.color],
                [cpu.memory[6], cpu.memory[7]]
            );
        }
    }
    for (index, descriptor) in OBJECT_CATALOG.iter().enumerate() {
        let address = 0xf44 + index * 9 + 4;
        let pointer = usize::from(rom[address]) | usize::from(rom[address + 1]) << 8;
        for state in 0..=255 {
            let mut range = pointer - 0xf000;
            while state > rom[range] {
                range += 3;
            }
            let bitmap = (usize::from(rom[range + 1]) | usize::from(rom[range + 2]) << 8) - 0xf000;
            let expected =
                &rom[bitmap..=rom[bitmap..].iter().position(|b| *b == 0).unwrap() + bitmap];
            let selected = Adventure::default().select_object_image(descriptor.images, state);
            assert_eq!(
                &selected.rows[selected.start..selected.start + expected.len()],
                expected
            );
        }
    }
}

#[test]
fn inactivity_and_all_effect_parameters_match_rom() {
    for effect in [
        SoundEffect::Victory,
        SoundEffect::Roar,
        SoundEffect::Swallow,
        SoundEffect::DragonSlain,
        SoundEffect::Drop,
        SoundEffect::Pickup,
    ] {
        for remaining in [0, 1, 4, 16, 128, 255] {
            let mut port = initialized(Game::One);
            port.ram.sound_effect = effect;
            port.ram.sound_remaining = remaining;
            let mut cpu = reference(&port);
            cpu.call(0xfa23);
            port.play_game_sound();
            assert_state(&port, &cpu, "sound");
            assert_eq!(
                [
                    port.sound.channels[0].waveform,
                    port.sound.channels[0].frequency,
                    port.sound.channels[0].volume
                ],
                [cpu.memory[0x15], cpu.memory[0x17], cpu.memory[0x19]]
            );
        }
    }
    for (low, high, active) in [
        (255, 255, false),
        (255, 127, false),
        (0, 128, true),
        (0, 128, false),
    ] {
        let mut port = initialized(Game::One);
        port.ram.idle_ticks = low;
        port.ram.idle_periods = high;
        port.controls.right.direction.up = active;
        let mut cpu = reference(&port);
        cpu.call(0xf2b2);
        port.track_inactivity();
        assert_state(&port, &cpu, "inactivity");
    }
}

#[test]
fn normal_turn_and_victory_keep_the_original_vertical_boundaries() {
    let mut game = initialized(Game::One);
    let mut host = Host::default();
    assert!(game.run_game(&mut host).unwrap());
    assert_eq!(host.lines, [0, HEIGHT, HEIGHT]);
    game.ram.chalice.room = Room::GoldTreasury;
    host.lines.clear();
    assert!(game.run_game(&mut host).unwrap());
    assert!(game.ram.won);
    assert_eq!(host.lines, [HEIGHT, HEIGHT, HEIGHT]);
    host.lines.clear();
    assert!(game.run_game(&mut host).unwrap());
    assert_eq!(host.lines, [HEIGHT]);
}

#[test]
fn startup_samples_held_switches_before_initialization_without_presenting() {
    for (reset, select) in [(false, false), (true, false), (false, true), (true, true)] {
        let controls = Controls {
            reset,
            select,
            ..Controls::default()
        };
        let mut game = Adventure::default();
        let mut host = Host {
            controls,
            ..Host::default()
        };

        // The ROM reads the physical switches while remembering its initial history.
        let original = Adventure {
            controls,
            ..Adventure::default()
        };
        let mut cpu = reference(&original);
        cpu.call(0xf3d3);
        assert!(game.power_on(&mut host).unwrap());
        assert_eq!(game.controls, controls);
        assert!(host.lines.is_empty());
        assert_state(&game, &cpu, "startup controls");

        // Keeping either switch held must not start play or advance the selected game.
        for _ in 0..2 {
            cpu.call(0xf384);
            assert!(game.run_game(&mut host).unwrap());
            assert_eq!(game.ram.player.room as u8, cpu.memory[0x8a]);
            assert_eq!(game.ram.game.index() as u8 * 2, cpu.memory[0xdd]);
        }
        assert_eq!(host.lines, [0, HEIGHT, HEIGHT, HEIGHT, HEIGHT, HEIGHT]);

        // Releasing and pressing still produces the ordinary console edge.
        game.controls = Controls::default();
        game.poll_console_switches(&mut host).unwrap();
        game.controls = controls;
        let mut cpu = reference(&game);
        cpu.call(0xf384);
        game.poll_console_switches(&mut host).unwrap();
        assert_state(&game, &cpu, "switch edge after startup release");
    }
}

#[test]
fn startup_quit_and_errors_return_before_any_frame() {
    struct StoppedHost {
        input: Result<Option<Controls>, String>,
    }
    impl FrameHost for StoppedHost {
        fn sample_controls(&mut self) -> Result<Option<Controls>, String> {
            self.input.clone()
        }

        fn vertical_sync(&mut self, _: &Frame, _: &Sound) -> Result<Option<Controls>, String> {
            panic!("startup termination must not present a frame");
        }
    }

    // Host termination and failure both unwind before the scheduler can draw or present.
    for input in [Ok(None), Err(String::from("input unavailable"))] {
        let expected = input.clone().map(|_| ());
        let mut host = StoppedHost { input };
        assert_eq!(Adventure::default().cartridge_entry(&mut host), expected);
    }
}

#[test]
fn console_edges_preserve_partial_reset_world_reset_and_game_selection() {
    for room in [Room::GameSelection, Room::GoldCastle] {
        for won in [false, true] {
            for (reset, select) in [(false, false), (true, false), (false, true), (true, true)] {
                let mut game = initialized(Game::Two);
                game.ram.player.room = room;
                game.ram.won = won;
                game.ram.rhindle.state = DragonState::Dead;
                game.ram.chalice = Position::new(Room::WestHall, 100, 90);
                game.controls.reset = reset;
                game.controls.select = select;
                let mut cpu = reference(&game);
                cpu.call(0xf384);
                let mut host = Host {
                    controls: game.controls,
                    ..Host::default()
                };
                game.poll_console_switches(&mut host).unwrap();
                assert_state(&game, &cpu, "console edge");
            }
        }
    }
}

#[test]
fn functional_kernel_matches_rom_output_order_for_every_room() {
    for room in ROOM_IDS {
        let mut game = initialized(Game::Two);
        game.position_side_barriers();
        game.ram.player = Position::new(room, 80, 50);
        game.update_dark_room_light();
        let mut cpu = reference(&game);
        cpu.call(0xf10f);
        game.prepare_room_display();
        cpu.raster = Some(oracle::Raster::new(&cpu.memory));
        cpu.raster.as_mut().unwrap().tia.left_barrier.reposition(13);
        cpu.raster
            .as_mut()
            .unwrap()
            .tia
            .right_barrier
            .reposition(150);
        cpu.call(0xf008);
        game.draw_room();
        let raster = cpu.raster.as_ref().unwrap();
        assert_eq!(
            game.tia.frame.painted_lines, raster.tia.frame.painted_lines,
            "{room:?}: lines"
        );
        assert_eq!(
            game.tia.frame.pixels, raster.tia.frame.pixels,
            "{room:?}: pixels"
        );
        assert_eq!(game.tia.contacts, raster.tia.contacts, "{room:?}: contacts");
    }
}

#[test]
fn initial_game_one_picture_matches_stella_pixels() {
    let mut game = initialized(Game::One);
    game.position_side_barriers();
    game.ram.player = Position::new(Room::GoldCastle, 80, 32);
    game.ram.safe_player = game.ram.player;
    game.prepare_room_display();
    game.draw_room();
    let expected = include_bytes!("../../tests/stella-game-one.pixels");
    assert_eq!(&game.tia.frame.pixels[..expected.len()], expected);
}
