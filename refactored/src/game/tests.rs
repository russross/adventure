use super::Adventure;
use super::assets::{
    GRUNDLE_REACTIONS, OBJECT_CATALOG, RHINDLE_REACTIONS, ROOM_IDS, YORGLE_REACTIONS,
};
use super::model::{
    Bitmap, DragonId, DragonState, Game, GateState, Gates, MovementPhase, ObjectId, Position,
    RecordId, Room, SoundEffect,
};
use crate::hardware::FrameHost;
use crate::input::{Controls, Difficulty, Direction};
use crate::sound::Sound;
use crate::video::{Contacts, Frame, HEIGHT};

#[path = "../reference/rom_oracle.rs"]
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

    fn present(&mut self, frame: &Frame, _sound: &Sound) -> Result<Option<Controls>, String> {
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
    adventure.world.game = game;
    assert!(adventure.initialize_world(&mut Host::default()).unwrap());
    adventure
}

// Serialization belongs only to the reference comparison, never to the running port.
fn reference(game: &Adventure) -> Cpu {
    let mut cpu = Cpu::new();
    for (address, record) in RECORDS {
        let position = game.world.position(record);
        cpu.memory[address..address + 3].copy_from_slice(&[
            position.room as u8,
            position.x,
            position.y,
        ]);
    }
    for (address, dragon) in [
        (0xa4, game.world.rhindle),
        (0xa9, game.world.yorgle),
        (0xae, game.world.grundle),
    ] {
        cpu.memory[address + 3] = direction_byte(dragon.direction);
        cpu.memory[address + 4] = dragon.state.value();
    }
    cpu.memory[0xc8] = game.world.gates.gold.value();
    cpu.memory[0xc9] = game.world.gates.white.value();
    cpu.memory[0xca] = game.world.gates.black.value();
    cpu.memory[0xce] = direction_byte(game.world.bat.direction);
    cpu.memory[0xcf] = game.world.bat.animation;
    cpu.memory[0xd0] = game.world.bat.carried.map_or(0x78, |record| {
        RECORDS.iter().find(|(_, r)| *r == record).unwrap().0 as u8
    });
    cpu.memory[0xd1] = game.world.bat.flight_timer;
    cpu.memory[0x95] = game.world.first_visible as u8 * 9;
    cpu.memory[0x96] = game.world.second_visible as u8 * 9;
    cpu.memory[0x9c] = game.world.last_candidate as u8 * 9;
    cpu.memory[0x9d] = game.world.held as u8 * 9;
    cpu.memory[0x9e] = game.world.held_offset_x;
    cpu.memory[0x9f] = game.world.held_offset_y;
    cpu.memory[0x99] = direction_byte(game.world.joystick_direction);
    cpu.memory[0xdd] = game.world.game.index() as u8 * 2;
    cpu.memory[0xde] = if game.world.won { 255 } else { 0 };
    cpu.memory[0xdf] = game.world.sound_remaining;
    cpu.memory[0xe0] = game.world.sound_effect as u8;
    cpu.memory[0xe2..0xe5].copy_from_slice(&[
        game.world.safe_player.room as u8,
        game.world.safe_player.x,
        game.world.safe_player.y,
    ]);
    cpu.memory[0xe5] = game.world.idle_ticks;
    cpu.memory[0xe6] = game.world.idle_periods;
    cpu.memory[0x92] =
        u8::from(!game.world.previous_reset) | u8::from(!game.world.previous_select) << 1;
    cpu.memory[0xd7] = if game.world.fire_was_pressed { 0 } else { 128 };

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
    cpu.memory[0x32] = u8::from(game.video.contacts.player_objects[0]) << 6;
    cpu.memory[0x33] = u8::from(game.video.contacts.player_objects[1]) << 6;
    cpu.memory[0x34] = u8::from(game.video.contacts.player_barriers[0]) << 6;
    cpu.memory[0x35] = u8::from(game.video.contacts.player_barriers[1]) << 6;
    cpu.memory[0x36] = u8::from(game.video.contacts.player_walls) << 7;
    cpu.memory[0x37] = u8::from(game.video.contacts.objects_touch) << 7;
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
        port.world.game = game;
        let mut cpu = reference(&port);
        cpu.call(0xf3d3);
        port.initialize_world(&mut Host::default()).unwrap();
        assert_state(&port, &cpu, "initialization");
    }
    for seed in 0..=255 {
        let mut port = initialized(Game::Two);
        port.world.idle_ticks = seed;
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
                    *port.world.position_mut(record) = Position::new(room, x, y);
                    port.world.dot.room = Room::SecretPassage;
                    port.world.gates = Gates {
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
                port.world.player = Position::new(Room::SecretPassage, 60u8.wrapping_add(dx), 50);
                port.world.safe_player = Position::new(Room::SecretPassage, 64, 50);
                port.world.bridge = Position::new(Room::SecretPassage, 60, 50u8.wrapping_add(dy));
                port.video.contacts.player_walls = true;
                port.controls.left.direction.right = true;
                port.world.joystick_direction = port.controls.left.direction;
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
        port.world.player = Position::new(Room::WestHall, 50, 50);
        port.world.sword = port.world.player;
        port.world.first_visible = ObjectId::Sword;
        port.world.held = held;
        port.world.joystick_direction.right = true;
        port.video.contacts.player_objects[0] = true;
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
                    port.world.player = Position::new(Room::WestHall, 70, 50);
                    port.world.safe_player = port.world.player;
                    port.world.sword = Position::new(Room::WestHall, 60, 50);
                    port.world.dragon_mut(dragon).position = Position::new(Room::WestHall, 80, 50);
                    port.world.dragon_mut(dragon).state = state;
                    port.world.first_visible = dragon.object();
                    port.world.second_visible = ObjectId::Sword;
                    port.video.contacts = Contacts {
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
        port.world.bat.position = Position::new(Room::WestHall, 60, 50);
        port.world.bat.flight_timer = timer;
        port.world.chalice = Position::new(Room::WestHall, 61, 50);
        port.world.held = ObjectId::Chalice;
        let mut cpu = reference(&port);
        cpu.call(0xf8a5);
        port.update_bat();
        assert_state(&port, &cpu, "bat");
    }
    for value in [0, 1, 2, 27, 28, 29, 54, 55] {
        let mut port = initialized(Game::One);
        port.world.gates.gold = GateState::from_value(value);
        port.world.first_visible = ObjectId::GoldGate;
        port.world.second_visible = ObjectId::GoldKey;
        port.video.contacts.objects_touch = true;
        port.video.contacts.player_objects[0] = true;
        let mut cpu = reference(&port);
        cpu.call(0xf93c);
        port.update_castle_gates();
        assert_state(&port, &cpu, "gate");
    }
    let mut port = initialized(Game::Two);
    port.world.magnet = Position::new(Room::WestHall, 70, 70);
    port.world.gold_key = Position::new(Room::WestHall, 60, 50);
    let mut cpu = reference(&port);
    cpu.call(0xf9b3);
    port.update_magnet();
    assert_state(&port, &cpu, "magnet");
    for room in ROOM_IDS {
        for x in [0, 13, 14, 129, 140, 159, 255] {
            port.world.player = Position::new(room, x, 50);
            let mut cpu = reference(&port);
            cpu.call(0xf9e7);
            port.update_dark_room_light();
            assert_state(&port, &cpu, "darkness");
        }
    }
}

#[test]
fn visible_selection_and_every_object_image_range_match_rom() {
    let rom = include_bytes!("../../../target/refactored-oracle.bin");
    for room in ROOM_IDS {
        let mut port = initialized(Game::Two);
        port.world.player.room = room;
        for _ in 0..20 {
            let mut cpu = reference(&port);
            cpu.call(0xf10f);
            port.prepare_room_display();
            assert_state(&port, &cpu, "display selection");
            assert_eq!(
                [
                    port.display.first_position.x,
                    port.display.first_position.y,
                    port.display.second_position.x,
                    port.display.second_position.y
                ],
                cpu.memory[0x86..0x8a]
            );
            assert_eq!(
                [port.display.objects[0].color, port.display.objects[1].color],
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
            port.world.sound_effect = effect;
            port.world.sound_remaining = remaining;
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
        port.world.idle_ticks = low;
        port.world.idle_periods = high;
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
    assert!(game.advance_turn(&mut host).unwrap());
    assert_eq!(host.lines, [0, HEIGHT, HEIGHT]);
    game.world.chalice.room = Room::GoldTreasury;
    host.lines.clear();
    assert!(game.advance_turn(&mut host).unwrap());
    assert!(game.world.won);
    assert_eq!(host.lines, [HEIGHT, HEIGHT, HEIGHT]);
    host.lines.clear();
    assert!(game.advance_turn(&mut host).unwrap());
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
            assert!(game.advance_turn(&mut host).unwrap());
            assert_eq!(game.world.player.room as u8, cpu.memory[0x8a]);
            assert_eq!(game.world.game.index() as u8 * 2, cpu.memory[0xdd]);
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

        fn present(&mut self, _: &Frame, _: &Sound) -> Result<Option<Controls>, String> {
            panic!("startup termination must not present a frame");
        }
    }

    // Host termination and failure both unwind before the scheduler can draw or present.
    for input in [Ok(None), Err(String::from("input unavailable"))] {
        let expected = input.clone().map(|_| ());
        let mut host = StoppedHost { input };
        assert_eq!(Adventure::default().run(&mut host), expected);
    }
}

#[test]
fn console_edges_preserve_partial_reset_world_reset_and_game_selection() {
    for room in [Room::GameSelection, Room::GoldCastle] {
        for won in [false, true] {
            for (reset, select) in [(false, false), (true, false), (false, true), (true, true)] {
                let mut game = initialized(Game::Two);
                game.world.player.room = room;
                game.world.won = won;
                game.world.rhindle.state = DragonState::Dead;
                game.world.chalice = Position::new(Room::WestHall, 100, 90);
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
        game.world.player = Position::new(room, 80, 50);
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
            game.video.frame.painted_lines, raster.tia.frame.painted_lines,
            "{room:?}: lines"
        );
        assert_eq!(
            game.video.frame.pixels, raster.tia.frame.pixels,
            "{room:?}: pixels"
        );
        assert_eq!(
            format!("{:?}", game.video.contacts),
            format!("{:?}", raster.tia.contacts),
            "{room:?}: contacts"
        );
    }
}

// Identical scripted controls exercise retained pictures across whole scheduler turns.
#[derive(Debug, PartialEq, Eq)]
struct Presented {
    pixels: Vec<u8>,
    lines: usize,
    voices: [[u8; 3]; 2],
}

#[derive(Default)]
struct TraceHost {
    frame: usize,
    seed: u32,
    presented: Vec<Presented>,
}

impl TraceHost {
    fn input(&self) -> adventure_port::input::Controls {
        use adventure_port::input::{Controls, Difficulty, Direction, Joystick};
        let random = self
            .seed
            .wrapping_add(self.frame as u32 / 9)
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);
        Controls {
            left: Joystick {
                direction: Direction {
                    right: random & 1 != 0,
                    left: random & 2 != 0,
                    up: random & 4 != 0,
                    down: random & 8 != 0,
                },
                fire: self.frame % 37 < 4,
            },
            right: Joystick {
                direction: Direction {
                    up: self.frame % 71 < 3,
                    ..Direction::default()
                },
                fire: false,
            },
            reset: self.frame % 113 < 3,
            select: self.frame % 251 >= 240,
            color: self.frame % 43 < 21,
            left_difficulty: if self.frame % 59 < 30 {
                Difficulty::A
            } else {
                Difficulty::B
            },
            right_difficulty: if self.frame % 61 < 30 {
                Difficulty::A
            } else {
                Difficulty::B
            },
        }
    }

    fn local_input(&self) -> Controls {
        let input = self.input();
        let direction = |d: adventure_port::input::Direction| Direction {
            right: d.right,
            left: d.left,
            up: d.up,
            down: d.down,
        };
        let difficulty = |d| {
            if d == adventure_port::input::Difficulty::A {
                Difficulty::A
            } else {
                Difficulty::B
            }
        };
        Controls {
            left: crate::input::Joystick {
                direction: direction(input.left.direction),
                fire: input.left.fire,
            },
            right: crate::input::Joystick {
                direction: direction(input.right.direction),
                fire: input.right.fire,
            },
            reset: input.reset,
            select: input.select,
            color: input.color,
            left_difficulty: difficulty(input.left_difficulty),
            right_difficulty: difficulty(input.right_difficulty),
        }
    }
}

impl FrameHost for TraceHost {
    fn sample_controls(&mut self) -> Result<Option<Controls>, String> {
        Ok(Some(self.local_input()))
    }

    fn present(&mut self, frame: &Frame, sound: &Sound) -> Result<Option<Controls>, String> {
        self.presented.push(Presented {
            pixels: frame.pixels.clone(),
            lines: frame.painted_lines,
            voices: sound.channels.map(|c| [c.waveform, c.frequency, c.volume]),
        });
        let input = self.local_input();
        self.frame += 1;
        Ok(Some(input))
    }
}

impl adventure_port::hardware::FrameHost for TraceHost {
    fn sample_controls(&mut self) -> Result<Option<adventure_port::input::Controls>, String> {
        Ok(Some(self.input()))
    }

    fn vertical_sync(
        &mut self,
        frame: &adventure_port::tia::Frame,
        sound: &adventure_port::sound::Sound,
    ) -> Result<Option<adventure_port::input::Controls>, String> {
        self.presented.push(Presented {
            pixels: frame.pixels.clone(),
            lines: frame.painted_lines,
            voices: sound.channels.map(|c| [c.waveform, c.frequency, c.volume]),
        });
        let input = self.input();
        self.frame += 1;
        Ok(Some(input))
    }
}

#[test]
fn long_frame_and_state_sequences_match_translation_in_all_games() {
    use adventure_port::adventure as translated;
    for (game, original_game) in [
        (Game::One, translated::Game::One),
        (Game::Two, translated::Game::Two),
        (Game::Three, translated::Game::Three),
    ] {
        for seed in [0, 91, 255] {
            let mut port = Adventure::default();
            let mut original = translated::Adventure::default();
            let mut host = TraceHost {
                seed,
                ..TraceHost::default()
            };
            let mut reference_host = TraceHost {
                seed,
                ..TraceHost::default()
            };
            port.world.game = game;
            original.ram.game = original_game;
            port.world.idle_ticks = seed as u8;
            original.ram.idle_ticks = seed as u8;
            port.initialize_world(&mut host).unwrap();
            original.initialize_world(&mut reference_host).unwrap();

            // Compare exact frame bytes and every persistent public game record after each turn.
            for turn in 0..400 {
                if turn == 300 {
                    port.world.chalice.room = Room::GoldTreasury;
                    original.ram.chalice.room = translated::Room::GoldTreasury;
                }
                assert!(port.advance_turn(&mut host).unwrap());
                assert!(original.run_game(&mut reference_host).unwrap());
                assert_eq!(
                    format!("{:?}", port.video.contacts),
                    format!("{:?}", original.tia.contacts),
                    "{game:?}, seed {seed}, turn {turn}: contacts"
                );
                assert_eq!(
                    host.presented, reference_host.presented,
                    "{game:?}, seed {seed}, turn {turn}: frames"
                );
                host.presented.clear();
                reference_host.presented.clear();
                macro_rules! compare {
                    ($($field:ident),+ $(,)?) => { $(assert_eq!(format!("{:?}", port.world.$field), format!("{:?}", original.ram.$field), "{game:?}, seed {seed}, turn {turn}: {}", stringify!($field));)+ };
                }
                compare!(
                    player,
                    safe_player,
                    light,
                    dot,
                    rhindle,
                    yorgle,
                    grundle,
                    magnet,
                    sword,
                    chalice,
                    bridge,
                    gold_key,
                    white_key,
                    black_key,
                    bat,
                    gates,
                    game,
                    won,
                    held,
                    held_offset_x,
                    held_offset_y,
                    first_visible,
                    second_visible,
                    last_candidate,
                    joystick_direction,
                    idle_ticks,
                    idle_periods
                );
                assert_eq!(host.frame, reference_host.frame);
            }
        }
    }
}

#[test]
fn picture_steps_stop_cleanly_at_each_scheduler_boundary() {
    struct Bounded {
        accepted: usize,
        calls: usize,
    }
    impl FrameHost for Bounded {
        fn sample_controls(&mut self) -> Result<Option<Controls>, String> {
            Ok(Some(Controls::default()))
        }
        fn present(&mut self, _: &Frame, _: &Sound) -> Result<Option<Controls>, String> {
            self.calls += 1;
            Ok((self.calls <= self.accepted).then_some(Controls::default()))
        }
    }
    for accepted in 0..=4 {
        let mut host = Bounded { accepted, calls: 0 };
        let mut game = Adventure::default();
        game.run(&mut host).unwrap();
        assert_eq!(host.calls, accepted + 1);
        assert_eq!(
            game.video.frame.painted_lines,
            if accepted == 0 { 0 } else { HEIGHT }
        );
    }
}

#[test]
fn initial_game_one_picture_matches_stella_pixels() {
    let mut game = initialized(Game::One);
    game.world.player = Position::new(Room::GoldCastle, 80, 32);
    game.world.safe_player = game.world.player;
    game.prepare_room_display();
    game.draw_room();
    let expected = include_bytes!("../../../tests/stella-game-one.pixels");
    assert_eq!(&game.video.frame.pixels[..expected.len()], expected);
}

#[test]
fn prepared_scanlines_match_stella_coordinate_boundaries() {
    use crate::video::{Appearance, ObjectLine, PlayfieldRow, SpriteWidth, WIDTH};
    let cases = [
        (0, 159),
        (1, 0),
        (32, 31),
        (80, 79),
        (159, 158),
        (164, 3),
        (165, 156),
        (179, 10),
        (225, 156),
        (239, 10),
        (240, 11),
        (255, 26),
    ];
    static NO_WALLS: [PlayfieldRow; 7] = [PlayfieldRow::new(0, 0, 0); 7];
    for (coordinate, expected) in cases {
        for width in [SpriteWidth::Normal, SpriteWidth::Wide] {
            let mut game = initialized(Game::One);
            game.display.appearance = Appearance {
                foreground: 28,
                player_width: 1,
                ..Appearance::default()
            };
            game.display.room_walls = &NO_WALLS;
            game.display.first_position = Position::new(Room::GoldCastle, coordinate, 64);
            game.display.first_image = Bitmap::new(&[0x80, 0]);
            game.display.second_image = Bitmap::new(&[0]);
            game.display.objects = [
                ObjectLine {
                    color: 72,
                    width,
                    ..ObjectLine::default()
                },
                ObjectLine::default(),
            ];
            game.world.player = Position::new(Room::GoldCastle, coordinate, 32);
            game.draw_room();
            let object: Vec<_> = game.video.frame.pixels[83 * WIDTH..84 * WIDTH]
                .iter()
                .enumerate()
                .filter_map(|(x, &color)| (color == 72).then_some(x))
                .collect();
            let (shift, count) = match width {
                SpriteWidth::Normal => (0, 1),
                SpriteWidth::Wide => (1, 4),
            };
            let mut expected_pixels: Vec<_> = (0..count)
                .map(|offset| (expected + shift + offset) % WIDTH)
                .collect();
            expected_pixels.sort_unstable();
            assert_eq!(
                object, expected_pixels,
                "coordinate {coordinate}, {width:?}"
            );
            let player: Vec<_> = game.video.frame.pixels[147 * WIDTH..148 * WIDTH]
                .iter()
                .enumerate()
                .filter_map(|(x, &color)| (color == 28).then_some(x))
                .collect();
            assert_eq!(
                player,
                [(expected + WIDTH - 1) % WIDTH],
                "player coordinate {coordinate}"
            );
        }
    }
}
