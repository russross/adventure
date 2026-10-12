use super::Adventure;
use super::assets::{OBJECT_CATALOG, ROOMS};
use super::model::{
    Bitmap, ColorChoice, ImageChoice, OBJECT_IDS, ObjectId, Position, PositionSource, Room,
    RoomDefinition, StateSource,
};
use crate::video::{Appearance, ObjectLine, Scanline, SpriteWidth, WIDTH};

impl Adventure {
    // Each logical row emits the retained line followed by the newly prepared line.
    pub(super) fn draw_room(&mut self) {
        let mut line = Scanline {
            appearance: self.display.appearance,
            walls: self.display.room_walls[0],
            objects: self.display.objects,
            player_x: screen_x(positioned_object_x(self.world.player.x) - 1),
            player_visible: self.display.player_visible,
        };
        line.objects[0].x = object_screen_x(self.display.first_position.x, line.objects[0].width);
        line.objects[1].x = object_screen_x(self.display.second_position.x, line.objects[1].width);
        line.objects[0].bits = 0;
        line.objects[1].bits = 0;
        self.video.begin_picture();
        let player_y = self.world.player.y.wrapping_sub(4);
        let mut cursors = [0; 2];
        let mut staged_second = 0;
        let mut band = 1;

        // The first retained line skips the second image fetch. Zero terminators stop cursors.
        for row in (8u8..=104).rev() {
            if row != 104 && below(row, self.display.second_position.y) {
                staged_second = self.display.second_image.row(cursors[1]);
                cursors[1] += usize::from(staged_second != 0);
            }
            let first = if below(row, self.display.first_position.y) {
                let bits = self.display.first_image.row(cursors[0]);
                cursors[0] += usize::from(bits != 0);
                bits
            } else {
                0
            };
            self.video.paint(&line);

            // Commit both object rows together; wall changes affect the second line only.
            line.objects[0].bits = first;
            line.objects[1].bits = staged_second;
            line.player_visible = row.wrapping_sub(player_y) & 0xfc == 0;
            if row & 15 == 0 {
                line.walls = self.display.room_walls[band];
                band += 1;
            }
            self.video.paint(&line);
        }
        self.display.player_visible = line.player_visible;
    }

    // Appearance and images are a snapshot, reused by the following two pictures.
    pub(super) fn prepare_room_display(&mut self) {
        let room = self.find_room_record(self.world.player.room);
        self.display.room_walls = room.walls;
        self.display.appearance = Appearance {
            foreground: self.resolve_display_color(if self.controls.color {
                room.color
            } else {
                room.monochrome
            }),
            background: self.resolve_display_color(ColorChoice::Fixed(8)),
            mirrored: room.mirrored,
            walls_in_front: room.foreground_priority,
            player_width: room.player_width,
            barriers: [room.left_barrier, room.right_barrier],
        };
        self.select_visible_objects();
        if self.world.first_visible == ObjectId::Light
            || self.world.first_visible == ObjectId::Bridge
                && self.world.second_visible != ObjectId::Light
        {
            std::mem::swap(
                &mut self.world.first_visible,
                &mut self.world.second_visible,
            );
        }

        // Both slots use the same catalog resolution; priority follows their selected order.
        for (slot, object) in [self.world.first_visible, self.world.second_visible]
            .into_iter()
            .enumerate()
        {
            let descriptor = OBJECT_CATALOG[object as usize];
            let position = self.find_object_position(object);
            let image =
                self.select_object_image(descriptor.images, self.image_state(descriptor.state));
            self.display.objects[slot] = ObjectLine {
                x: object_screen_x(position.x, descriptor.style),
                color: self.resolve_display_color(if self.controls.color {
                    descriptor.color
                } else {
                    descriptor.monochrome
                }),
                width: descriptor.style,
                bits: 0,
            };
            if slot == 0 {
                self.display.first_position = position;
                self.display.first_image = image;
            } else {
                self.display.second_position = position;
                self.display.second_image = image;
            }
        }
    }

    // State pointers become a closed selection of named game fields and fixed values.
    pub(super) fn image_state(&self, source: StateSource) -> u8 {
        match source {
            StateSource::Fixed(value) => value,
            StateSource::Dragon(dragon) => self.world.dragon(dragon).state.value(),
            StateSource::Gate(gate) => self.world.gates.get(gate).value(),
            StateSource::GameNumber => (self.world.game.index() * 2) as u8,
            StateSource::BatAnimation => self.world.bat.animation,
        }
    }

    // Rotate over the real inhabitants, retaining at most two in this room.
    pub(super) fn select_visible_objects(&mut self) {
        self.world.first_visible = ObjectId::Empty;
        self.world.second_visible = ObjectId::Empty;
        let mut candidate = self.world.last_candidate;
        loop {
            candidate = OBJECT_IDS[(candidate as usize + 1) % 18];
            if self.find_object_position(candidate).room == self.world.player.room {
                if self.world.first_visible == ObjectId::Empty {
                    self.world.first_visible = candidate;
                } else {
                    self.world.second_visible = candidate;
                    break;
                }
            }
            if candidate == self.world.last_candidate {
                break;
            }
        }
        self.world.last_candidate = candidate;
    }

    // Room identities index the immutable map catalog.
    pub(super) fn find_room_record(&self, room: Room) -> &'static RoomDefinition {
        &ROOMS[room as usize]
    }

    // The first inclusive ceiling selects the object's current bitmap.
    pub(super) fn select_object_image(&self, choices: &'static [ImageChoice], state: u8) -> Bitmap {
        choices
            .iter()
            .find(|choice| state <= choice.ceiling)
            .expect("image ranges end at 255")
            .bitmap
    }

    // Resolve literal or counter-driven colors, then apply the idle transform.
    pub(super) fn resolve_display_color(&self, color: ColorChoice) -> u8 {
        let mut value = match color {
            ColorChoice::Fixed(value) => value >> 1,
            ColorChoice::Cycling => self.world.idle_ticks,
        };
        if self.world.idle_periods & 0x80 != 0 {
            value = (value ^ self.world.idle_periods) & 0xfb;
        }
        value.wrapping_shl(1)
    }

    // Position sources distinguish live records from immutable scenery.
    pub(super) fn find_object_position(&self, object: ObjectId) -> Position {
        match OBJECT_CATALOG[object as usize].position {
            PositionSource::Record(record) => self.world.position(record),
            PositionSource::Fixed(position) => position,
        }
    }
}

// Coordinates retain byte wrapping before their signed vertical comparison.
fn below(row: u8, y: u8) -> bool {
    (row.wrapping_sub(y) as i8) < 0
}

// Game coordinates encode the cartridge's calibrated placement, including line crossings.
fn positioned_object_x(x: u8) -> i16 {
    match x {
        0..=164 => i16::from(x) - 1,
        165..=239 => i16::from(x % 15) - 4,
        240..=255 => i16::from(x) - 229,
    }
}

fn object_screen_x(x: u8, width: SpriteWidth) -> u8 {
    let expansion_shift = i16::from(matches!(width, SpriteWidth::Wide));
    screen_x(positioned_object_x(x) + expansion_shift)
}

// Normalize after the signed correction so the left edge wraps through pixel 159.
fn screen_x(x: i16) -> u8 {
    x.rem_euclid(WIDTH as i16) as u8
}
