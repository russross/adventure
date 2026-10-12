use super::Adventure;
use super::assets::{
    BAT_REACTIONS, DRAGON_BITE_DELAYS, GRUNDLE_REACTIONS, OBJECT_CATALOG, RHINDLE_REACTIONS,
    YORGLE_REACTIONS,
};
use super::model::{
    ChosenReaction, Creature, DragonId, DragonState, ObjectId, Position, Reaction, RecordId,
    SoundEffect,
};
use crate::input::{Difficulty, Direction};

impl Adventure {
    // No eligible reaction retains motion; an eligible but aligned pair stops motion.
    pub(super) fn move_creature(
        &mut self,
        creature: Creature,
        list: &[Reaction],
        ignored: Option<RecordId>,
        speed: u8,
    ) -> Option<ChosenReaction> {
        let chosen = self.choose_reaction(list, ignored);
        let previous = match creature {
            Creature::Dragon(dragon) => self.world.dragon(dragon).direction,
            Creature::Bat => self.world.bat.direction,
        };
        let direction = chosen.map_or(previous, |chosen| chosen.direction);
        match creature {
            Creature::Dragon(dragon) => self.world.dragon_mut(dragon).direction = direction,
            Creature::Bat => self.world.bat.direction = direction,
        }
        self.move_object_through_world(creature.record(), direction, speed);
        chosen
    }

    // Ordered pairs use separate local identities and return the chosen pair to the bat.
    pub(super) fn choose_reaction(
        &self,
        list: &[Reaction],
        ignored: Option<RecordId>,
    ) -> Option<ChosenReaction> {
        for reaction in list {
            if self.world.position(reaction.from).room != self.world.position(reaction.to).room {
                continue;
            }
            if ignored == Some(reaction.from) || ignored == Some(reaction.to) {
                continue;
            }
            return Some(ChosenReaction {
                reaction: *reaction,
                direction: self.direction_between_objects(reaction.from, reaction.to),
            });
        }
        None
    }

    // Each reaction points from its first record toward its second record.
    pub(super) fn direction_between_objects(&self, from: RecordId, to: RecordId) -> Direction {
        let from = self.world.position(from);
        let to = self.world.position(to);
        if from.room != to.room {
            return Direction::default();
        }
        Direction {
            right: to.x > from.x,
            left: to.x < from.x,
            up: to.y > from.y,
            down: to.y < from.y,
        }
    }

    // Rhindle moves three steps using his own priorities.
    pub(super) fn update_rhindle(&mut self) {
        self.update_dragon(DragonId::Rhindle, &RHINDLE_REACTIONS, 3);
    }

    // Yorgle moves two steps and includes the Gold Key among his fears.
    pub(super) fn update_yorgle(&mut self) {
        self.update_dragon(DragonId::Yorgle, &YORGLE_REACTIONS, 2);
    }

    // Grundle moves two steps using his guarding priorities.
    pub(super) fn update_grundle(&mut self) {
        self.update_dragon(DragonId::Grundle, &GRUNDLE_REACTIONS, 2);
    }

    // Preserve wandering, sword contact, swallowing, and delayed-bite branch order.
    pub(super) fn update_dragon(&mut self, dragon: DragonId, reactions: &[Reaction], speed: u8) {
        let object = dragon.object();
        match self.world.dragon(dragon).state {
            DragonState::Wandering => {
                let ignored = if self.controls.right_difficulty == Difficulty::B {
                    Some(RecordId::Sword)
                } else {
                    None
                };
                self.move_creature(Creature::Dragon(dragon), reactions, ignored, speed);
                if self.is_touching_player(object) {
                    let difficulty = usize::from(self.controls.left_difficulty == Difficulty::A);
                    self.world.dragon_mut(dragon).state = DragonState::Biting(
                        DRAGON_BITE_DELAYS[self.world.game.index()][difficulty],
                    );
                    self.world.dragon_mut(dragon).position.x = self.world.safe_player.x;
                    self.world.dragon_mut(dragon).position.y = self.world.safe_player.y;
                    self.world.sound_effect = SoundEffect::Roar;
                    self.world.sound_remaining = 16;
                }
                if self.find_touching_object(object) == ObjectId::Sword {
                    self.world.dragon_mut(dragon).state = DragonState::Dead;
                    self.world.sound_effect = SoundEffect::DragonSlain;
                    self.world.sound_remaining = 16;
                }
            }
            DragonState::Dead => {}
            DragonState::Swallowed => {
                let position = self.world.dragon(dragon).position;
                self.world.player = Position::new(
                    position.room,
                    position.x.wrapping_add(3),
                    position.y.wrapping_sub(10),
                );
                self.world.safe_player = self.world.player;
            }
            DragonState::Biting(count) => {
                let count = count.wrapping_add(1);
                self.world.dragon_mut(dragon).state = DragonState::from_value(count);
                if count < 252 || !self.is_touching_player(object) {
                    return;
                }
                self.world.dragon_mut(dragon).state = DragonState::Swallowed;
                self.world.sound_effect = SoundEffect::Swallow;
                self.world.sound_remaining = 16;
                let position = &mut self.world.dragon_mut(dragon).position;
                position.x = position.x.min(155);
                position.y = position.y.max(23);
            }
        }
    }

    // The bat reuses its previous heading during its flight commitment.
    pub(super) fn update_bat(&mut self) {
        self.world.bat.animation = self.world.bat.animation.wrapping_add(1);
        if self.world.bat.animation == 8 {
            self.world.bat.animation = 0;
        }
        if self.world.bat.flight_timer != 0 {
            self.world.bat.flight_timer = self.world.bat.flight_timer.wrapping_add(1);
            self.move_object_through_world(RecordId::Bat, self.world.bat.direction, 3);
        } else {
            let chosen =
                self.move_creature(Creature::Bat, &BAT_REACTIONS, self.world.bat.carried, 3);
            if let Some(chosen) = chosen {
                let target = self.world.position(chosen.reaction.to);
                let bat = self.world.bat.position;
                if target.room == bat.room
                    && target.x.wrapping_sub(bat.x).wrapping_add(4) & 0xf8 == 0
                    && target.y.wrapping_sub(bat.y).wrapping_add(4) & 0xf8 == 0
                {
                    self.world.bat.carried = Some(chosen.reaction.to);
                    self.world.bat.flight_timer = 16;
                }
            }
        }
        if let Some(cargo) = self.world.bat.carried {
            let bat = self.world.bat.position;
            *self.world.position_mut(cargo) = Position::new(bat.room, bat.x.wrapping_add(8), bat.y);
            if OBJECT_CATALOG[self.world.held as usize].position_matches(cargo) {
                self.world.held = ObjectId::Empty;
            }
        }
    }
}
