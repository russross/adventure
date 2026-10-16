//! Complete-picture layer composition and independent geometric contacts.

use std::ops::Range;

pub const WIDTH: usize = 160;
pub const HEIGHT: usize = 194;

// Coverage bits retain geometric layers even when priority hides their colors.
const WALL_MASK: u8 = 1;
const PLAYER_MASK: u8 = 2;
const FOREGROUND_MASK: u8 = WALL_MASK | PLAYER_MASK;
const OBJECT_MASKS: [u8; 2] = [4, 8];
const BARRIER_MASKS: [u8; 2] = [16, 32];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PlayfieldRow {
    cells: [bool; 20],
}

impl PlayfieldRow {
    // Decode packed artwork once; rendering works with twenty four-pixel wall cells.
    pub const fn new(edge: u8, middle: u8, inner: u8) -> Self {
        let mut cells = [false; 20];
        let mut cell = 0;
        while cell < 20 {
            cells[cell] = if cell < 4 {
                edge & (1 << (cell + 4)) != 0
            } else if cell < 12 {
                middle & (1 << (11 - cell)) != 0
            } else {
                inner & (1 << (cell - 12)) != 0
            };
            cell += 1;
        }
        Self { cells }
    }

    fn covers(self, x: usize, mirrored: bool) -> bool {
        let cell = x / 4;
        self.cells[if cell < 20 {
            cell
        } else if mirrored {
            39 - cell
        } else {
            cell - 20
        }]
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SpriteWidth {
    #[default]
    Normal,
    Wide,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ObjectLine {
    pub x: u8,
    pub color: u8,
    pub width: SpriteWidth,
    pub bits: u8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Appearance {
    pub background: u8,
    pub foreground: u8,
    pub mirrored: bool,
    pub walls_in_front: bool,
    pub player_width: u8,
    pub barriers: [bool; 2],
}

// Each drawable owns its complete vertical coverage before composition begins.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Object {
    pub line: ObjectLine,
    pub rows: [u8; HEIGHT],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Picture {
    pub appearance: Appearance,
    pub walls: [PlayfieldRow; HEIGHT],
    pub objects: [Object; 2],
    pub player_x: u8,
    pub player_visible: [bool; HEIGHT],
}

impl Default for Picture {
    fn default() -> Self {
        Self {
            appearance: Appearance::default(),
            walls: [PlayfieldRow::default(); HEIGHT],
            objects: [Object {
                line: ObjectLine::default(),
                rows: [0; HEIGHT],
            }; 2],
            player_x: 0,
            player_visible: [false; HEIGHT],
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Contacts {
    pub player_walls: bool,
    pub player_barriers: [bool; 2],
    pub player_objects: [bool; 2],
    pub objects_touch: bool,
}

pub struct Frame {
    pub pixels: Vec<u8>,
    pub painted_lines: usize,
}

impl Default for Frame {
    fn default() -> Self {
        Self {
            pixels: vec![0; WIDTH * HEIGHT],
            painted_lines: 0,
        }
    }
}

#[derive(Default)]
pub struct Video {
    pub frame: Frame,
    pub contacts: Contacts,
    retained: Option<(Picture, Contacts)>,
}

impl Video {
    // Geometry remains separate from color so every hidden overlap still contributes contacts.
    pub fn paint(&mut self, picture: &Picture) {
        // An identical picture has identical hidden geometry as well as visible pixels.
        if let Some((previous, contacts)) = &self.retained
            && previous == picture
        {
            self.contacts = *contacts;
            self.frame.painted_lines = HEIGHT;
            return;
        }
        let appearance = picture.appearance;
        let mut coverage = vec![0u8; WIDTH * HEIGHT];
        self.contacts = Contacts::default();

        // Expand each distinct wall band once and copy its complete rows into the picture.
        let mut cached_walls = None;
        let mut wall_coverage = [0; WIDTH];
        let mut wall_pixels = [appearance.background; WIDTH];
        let player_spans = wrapped_spans(
            usize::from(picture.player_x),
            usize::from(appearance.player_width),
        );
        for (y, (masks, pixels)) in coverage
            .chunks_exact_mut(WIDTH)
            .zip(self.frame.pixels.chunks_exact_mut(WIDTH))
            .enumerate()
        {
            let walls = picture.walls[y];
            if cached_walls != Some(walls) {
                wall_coverage.fill(0);
                wall_pixels.fill(appearance.background);
                for (cell, (masks, pixels)) in wall_coverage
                    .chunks_exact_mut(4)
                    .zip(wall_pixels.chunks_exact_mut(4))
                    .enumerate()
                {
                    if walls.covers(cell * 4, appearance.mirrored) {
                        masks.fill(WALL_MASK);
                        pixels.fill(appearance.foreground);
                    }
                }
                cached_walls = Some(walls);
            }
            masks.copy_from_slice(&wall_coverage);
            pixels.copy_from_slice(&wall_pixels);

            // The player covers at most two spans, split only at the screen seam.
            if picture.player_visible[y] {
                for span in player_spans.clone() {
                    for (mask, pixel) in masks[span.clone()].iter_mut().zip(&mut pixels[span]) {
                        self.contacts.player_walls |= *mask & WALL_MASK != 0;
                        *mask |= PLAYER_MASK;
                        *pixel = appearance.foreground;
                    }
                }
            }
        }

        // The second slot is underneath the first, including their fixed thin barriers.
        for slot in [1, 0] {
            let object = picture.objects[slot];
            let scale = match object.line.width {
                SpriteWidth::Normal => 1,
                SpriteWidth::Wide => 4,
            };
            let origin = usize::from(object.line.x) % WIDTH;
            for (y, bits) in object.rows.into_iter().enumerate() {
                let row = y * WIDTH;
                if appearance.barriers[slot] {
                    let x = [11, 148][slot];
                    let span = row + x..row + x + 2;
                    for (mask, pixel) in coverage[span.clone()]
                        .iter_mut()
                        .zip(&mut self.frame.pixels[span])
                    {
                        self.contacts.player_barriers[slot] |= *mask & PLAYER_MASK != 0;
                        *mask |= BARRIER_MASKS[slot];
                        if !appearance.walls_in_front || *mask & FOREGROUND_MASK == 0 {
                            *pixel = object.line.color;
                        }
                    }
                }

                // Set bits accumulate hidden contacts before painting their scaled footprints.
                let mut remaining = bits;
                while remaining != 0 {
                    let bit = remaining.leading_zeros() as usize;
                    remaining &= !(0x80 >> bit);
                    for offset in 0..scale {
                        let x = origin + bit * scale + offset;
                        let x = if x >= WIDTH { x - WIDTH } else { x };
                        let index = row + x;
                        let mask = &mut coverage[index];
                        self.contacts.player_objects[slot] |= *mask & PLAYER_MASK != 0;
                        self.contacts.objects_touch |= *mask & OBJECT_MASKS[1 - slot] != 0;
                        *mask |= OBJECT_MASKS[slot];
                        if !appearance.walls_in_front || *mask & FOREGROUND_MASK == 0 {
                            self.frame.pixels[index] = object.line.color;
                        }
                    }
                }
            }
        }

        self.frame.painted_lines = HEIGHT;
        self.retained = Some((*picture, self.contacts));
    }
}

// Byte coordinates normalize to the screen; widths exceeding it cover every pixel once.
fn wrapped_spans(x: usize, width: usize) -> [Range<usize>; 2] {
    let start = x % WIDTH;
    let width = width.min(WIDTH);
    let first_width = width.min(WIDTH - start);
    [start..start + first_width, 0..width - first_width]
}

#[cfg(test)]
#[path = "tests/video.rs"]
mod tests;
