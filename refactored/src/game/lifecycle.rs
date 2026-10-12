use super::Adventure;
use super::PicturePhase;
use super::assets::{RANDOM_ROOM_BOUNDS, ROOM_IDS, STARTING_WORLDS};
use super::model::{
    DragonState, Game, MovementPhase, ObjectId, Position, PreparedDisplay, Room, SoundEffect, World,
};
use crate::hardware::FrameHost;
use crate::sound::Sound;
use crate::video::Video;

impl Adventure {
    // The entry keeps startup and the repeated game-turn sequence separate.
    pub fn run(&mut self, host: &mut impl FrameHost) -> Result<(), String> {
        if !self.power_on(host)? {
            return Ok(());
        }
        while self.advance_picture(host)? {}
        Ok(())
    }

    // Clear state once, place the fixed barriers, and initialize the selected world.
    pub(super) fn power_on(&mut self, host: &mut impl FrameHost) -> Result<bool, String> {
        self.world = World::default();
        self.phase = PicturePhase::Player;
        self.video = Video::default();
        self.display = PreparedDisplay::default();
        self.sound = Sound::default();
        let Some(controls) = host.sample_controls()? else {
            return Ok(false);
        };
        self.controls = controls;
        self.initialize_world(host)
    }

    // A turn ends after three active pictures or one victory picture.
    pub fn advance_turn(&mut self, host: &mut impl FrameHost) -> Result<bool, String> {
        loop {
            if !self.advance_picture(host)? {
                return Ok(false);
            }
            if self.phase == PicturePhase::Player {
                return Ok(true);
            }
        }
    }

    // Each phase makes its update boundary explicit and presents exactly once.
    pub fn advance_picture(&mut self, host: &mut impl FrameHost) -> Result<bool, String> {
        match self.phase {
            PicturePhase::Player => {
                if !self.poll_console_switches(host)? {
                    return Ok(false);
                }
                self.play_game_sound();
                self.track_inactivity();
                if self.world.won {
                    if !self.present_picture(host)? {
                        return Ok(false);
                    }
                    self.draw_room();
                    self.prepare_room_display();
                    return Ok(true);
                }
                if self.world.chalice.room == Room::GoldTreasury {
                    self.world.sound_remaining = 255;
                    self.world.won = true;
                    self.world.sound_effect = SoundEffect::Victory;
                }

                // Player movement precedes presentation; pickup uses the newly drawn contacts.
                self.move_player(MovementPhase::Both);
                self.move_held_object();
                if !self.present_picture(host)? {
                    return Ok(false);
                }
                self.prepare_room_display();
                self.draw_room();
                self.handle_pickup_and_drop();
                self.phase = PicturePhase::BatAndGates;
            }
            PicturePhase::BatAndGates => {
                self.move_player(MovementPhase::Vertical);
                self.update_dark_room_light();
                if !self.present_picture(host)? {
                    return Ok(false);
                }
                self.update_bat();
                self.update_castle_gates();
                self.draw_room();
                self.update_grundle();
                self.update_yorgle();
                self.phase = PicturePhase::RemainingActors;
            }
            PicturePhase::RemainingActors => {
                if !self.present_picture(host)? {
                    return Ok(false);
                }
                self.move_player(MovementPhase::Horizontal);
                self.update_rhindle();
                self.update_magnet();
                self.draw_room();
                self.phase = PicturePhase::Player;
            }
        }
        Ok(true)
    }

    // Present the accumulated frame and sample controls at the vertical boundary.
    pub(super) fn present_picture(&mut self, host: &mut impl FrameHost) -> Result<bool, String> {
        match host.present(&self.video.frame, &self.sound)? {
            Some(controls) => {
                self.controls = controls;
                Ok(true)
            }
            None => Ok(false),
        }
    }

    // Button edges preserve RESET's partial revival and SELECT's world reset.
    pub(super) fn poll_console_switches(
        &mut self,
        host: &mut impl FrameHost,
    ) -> Result<bool, String> {
        if self.controls.reset && !self.world.previous_reset {
            if self.world.won {
                return self.initialize_world(host);
            }
            self.world.player = Position::new(Room::GoldCastle, 80, 32);
            self.world.safe_player = self.world.player;
            self.world.rhindle.state = DragonState::Wandering;
            self.world.yorgle.state = DragonState::Wandering;
            self.world.grundle.state = DragonState::Wandering;
            self.world.sound_remaining = 0;
            self.world.held = ObjectId::Empty;
        }
        if self.controls.select && !self.world.previous_select {
            if self.world.player.room == Room::GameSelection {
                self.world.game = self.world.game.next();
            }
            return self.initialize_world(host);
        }
        self.remember_console_switches();
        Ok(true)
    }

    // Only the copied world fields are reset; display caches and counters remain live.
    pub fn initialize_world(&mut self, host: &mut impl FrameHost) -> Result<bool, String> {
        self.world.player.room = Room::GameSelection;
        self.world.safe_player.room = Room::GameSelection;
        self.world.player.y = 0;
        self.world.safe_player.y = 0;
        let world = STARTING_WORLDS[self.world.game.index()];
        self.world.dot = world.dot;
        self.world.rhindle = world.rhindle;
        self.world.yorgle = world.yorgle;
        self.world.grundle = world.grundle;
        self.world.magnet = world.magnet;
        self.world.sword = world.sword;
        self.world.chalice = world.chalice;
        self.world.bridge = world.bridge;
        self.world.gold_key = world.gold_key;
        self.world.white_key = world.white_key;
        self.world.black_key = world.black_key;
        self.world.gates = world.gates;
        self.world.bat = world.bat;

        if self.world.game == Game::Three {
            self.randomize_game_three();
            if !self.present_picture(host)? {
                return Ok(false);
            }
            self.draw_room();
        }
        self.world.won = false;
        self.world.held = ObjectId::Empty;
        self.remember_console_switches();
        Ok(true)
    }

    // The shared console-poll tail needs no stack or shared pointer state.
    pub(super) fn remember_console_switches(&mut self) {
        self.world.previous_reset = self.controls.reset;
        self.world.previous_select = self.controls.select;
    }

    // Work backward through the eleven constraints, retaining the counter as seed.
    pub(super) fn randomize_game_three(&mut self) {
        for bounds in RANDOM_ROOM_BOUNDS.iter().rev() {
            loop {
                let old = self.world.idle_ticks;
                self.world.idle_ticks = (old >> 5).wrapping_add(old).wrapping_add(1);
                let candidate = self.world.idle_ticks & 31;
                if candidate >= bounds.first as u8 && candidate <= bounds.last as u8 {
                    self.world.position_mut(bounds.record).room = ROOM_IDS[usize::from(candidate)];
                    break;
                }
            }
        }
    }

    // The counter also supplies dynamic colors and Game 3's placement seed.
    pub(super) fn track_inactivity(&mut self) {
        self.world.idle_ticks = self.world.idle_ticks.wrapping_add(1);
        if self.world.idle_ticks == 0 {
            self.world.idle_periods = self.world.idle_periods.wrapping_add(1);
            if self.world.idle_periods == 0 {
                self.world.idle_periods = 0x80;
            }
        }
        if !self.controls.left.direction.is_idle()
            || !self.controls.right.direction.is_idle()
            || self.controls.reset
            || self.controls.select
        {
            self.world.idle_periods = 0;
        }
    }
}
