//! Complete-picture layer composition and independent geometric contacts.

use std::ops::Range;

pub const WIDTH: usize = 160;
pub const HEIGHT: usize = 194;

// Three words cover one screen row without conflating overlapping layers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Mask([u64; 3]);

impl Mask {
    fn insert(&mut self, x: usize) {
        self.0[x / 64] |= 1 << (x % 64);
    }

    fn overlaps(self, other: Self) -> bool {
        self.0.into_iter().zip(other.0).any(|(a, b)| a & b != 0)
    }

    fn union(self, other: Self) -> Self {
        Self(std::array::from_fn(|i| self.0[i] | other.0[i]))
    }

    fn without(self, other: Self) -> Self {
        Self(std::array::from_fn(|i| self.0[i] & !other.0[i]))
    }

    // Visit only covered pixels; the unused high bits of the last word stay clear.
    fn paint(self, pixels: &mut [u8], color: u8) {
        for (word, mut bits) in self.0.into_iter().enumerate() {
            while bits != 0 {
                let bit = bits.trailing_zeros() as usize;
                pixels[word * 64 + bit] = color;
                bits &= bits - 1;
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct GeometryRow {
    walls: Mask,
    player: Mask,
    objects: [Mask; 2],
    barriers: [Mask; 2],
}

/// Complete geometric layers, independent of colors, priority, and pixel painting.
pub struct Geometry {
    rows: [GeometryRow; HEIGHT],
    source: Option<Picture>,
}

impl Default for Geometry {
    fn default() -> Self {
        Self {
            rows: [GeometryRow::default(); HEIGHT],
            source: None,
        }
    }
}

impl Geometry {
    pub fn prepare(picture: &Picture) -> Self {
        let mut geometry = Self::default();
        geometry.update(picture);
        geometry
    }

    /// Rebuild affected coverage rows while retaining unchanged geometry, including hidden layers.
    pub fn update(&mut self, picture: &Picture) {
        let appearance = picture.appearance;
        let mut player = Mask::default();
        for span in wrapped_spans(
            usize::from(picture.player_x),
            usize::from(appearance.player_width),
        ) {
            for x in span {
                player.insert(x);
            }
        }
        let barriers = std::array::from_fn(|slot| {
            let mut mask = Mask::default();
            if appearance.barriers[slot] {
                let x = [11, 148][slot];
                mask.insert(x);
                mask.insert(x + 1);
            }
            mask
        });

        // Consecutive wall bands share their expansion. Sprites retain independent footprints.
        let mut cached_walls = None;
        let mut walls = Mask::default();
        for y in 0..HEIGHT {
            if self
                .source
                .as_ref()
                .is_some_and(|previous| previous.same_geometry_row(picture, y))
            {
                continue;
            }
            if cached_walls != Some(picture.walls[y]) {
                walls = Mask::default();
                for x in 0..WIDTH {
                    if picture.walls[y].covers(x, appearance.mirrored) {
                        walls.insert(x);
                    }
                }
                cached_walls = Some(picture.walls[y]);
            }
            let objects = std::array::from_fn(|slot| {
                let object = &picture.objects[slot];
                let scale = match object.line.width {
                    SpriteWidth::Normal => 1,
                    SpriteWidth::Wide => 4,
                };
                let mut mask = Mask::default();
                let mut bits = object.rows[y];
                while bits != 0 {
                    let bit = bits.leading_zeros() as usize;
                    bits &= !(0x80 >> bit);
                    for offset in 0..scale {
                        mask.insert((usize::from(object.line.x) + bit * scale + offset) % WIDTH);
                    }
                }
                mask
            });
            let mut row = GeometryRow {
                walls,
                player: if picture.player_visible[y] {
                    player
                } else {
                    Mask::default()
                },
                objects,
                barriers,
            };
            // Blanking removes geometry as well as color, so off-picture overlap cannot latch.
            if usize::from(picture.visible_widths[y]) < WIDTH {
                let mut hidden = Mask::default();
                for x in usize::from(picture.visible_widths[y])..WIDTH {
                    hidden.insert(x);
                }
                row.walls = row.walls.without(hidden);
                row.player = row.player.without(hidden);
                row.objects = row.objects.map(|mask| mask.without(hidden));
                row.barriers = row.barriers.map(|mask| mask.without(hidden));
            }
            self.rows[y] = row;
        }
        self.source = Some(*picture);
    }

    /// Detect every overlap, including objects hidden by foreground priority.
    pub fn contacts(&self) -> Contacts {
        let mut contacts = Contacts::default();
        for row in &self.rows {
            contacts.player_walls |= row.player.overlaps(row.walls);
            for slot in 0..2 {
                contacts.player_objects[slot] |= row.player.overlaps(row.objects[slot]);
                contacts.player_barriers[slot] |= row.player.overlaps(row.barriers[slot]);
            }
            contacts.objects_touch |= row.objects[0].overlaps(row.objects[1]);
        }
        contacts
    }
}

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
    /// Visible prefix of each row; the remainder is blanked black and cannot collide.
    pub visible_widths: [u8; HEIGHT],
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
            visible_widths: [WIDTH as u8; HEIGHT],
        }
    }
}

impl Picture {
    // Positions and widths matter only on occupied rows; colors never change coverage.
    fn same_geometry_row(&self, other: &Self, y: usize) -> bool {
        self.visible_widths[y] == other.visible_widths[y]
            && self.appearance.barriers == other.appearance.barriers
            && self.appearance.mirrored == other.appearance.mirrored
            && self.walls[y] == other.walls[y]
            && self.player_visible[y] == other.player_visible[y]
            && (!self.player_visible[y]
                || self.player_x == other.player_x
                    && self.appearance.player_width == other.appearance.player_width)
            && self
                .objects
                .iter()
                .zip(&other.objects)
                .all(|(first, second)| {
                    first.rows[y] == second.rows[y]
                        && (first.rows[y] == 0
                            || first.line.x == second.line.x
                                && first.line.width == second.line.width)
                })
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
    /// Rows recomposed by the latest paint, including changes hidden by priority.
    pub changed_rows: Vec<Range<usize>>,
}

impl Default for Frame {
    fn default() -> Self {
        Self {
            pixels: vec![0; WIDTH * HEIGHT],
            painted_lines: 0,
            changed_rows: Vec::new(),
        }
    }
}

#[derive(Default)]
pub struct Video {
    pub frame: Frame,
    pub contacts: Contacts,
    retained: Option<(Picture, Contacts)>,
    composed: Vec<Option<PaintedRow>>,
}

// Appearance only includes values that affect composition; geometry already encodes widths and mirroring.
#[derive(Clone, Copy, PartialEq, Eq)]
struct PaintedRow {
    geometry: GeometryRow,
    colors: [u8; 4],
    foreground_priority: bool,
    visible_width: u8,
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
            self.frame.changed_rows.clear();
            return;
        }
        let geometry = Geometry::prepare(picture);
        self.contacts = geometry.contacts();
        self.compose(picture, &geometry);
        self.retained = Some((*picture, self.contacts));
    }

    /// Paint visual geometry without changing the game's contact observations.
    pub fn compose(&mut self, picture: &Picture, geometry: &Geometry) {
        self.retained = None;
        let appearance = picture.appearance;
        self.composed.resize(HEIGHT, None);
        self.frame.changed_rows.clear();
        let mut changed_start = None;
        for (y, (row, pixels)) in geometry
            .rows
            .iter()
            .zip(self.frame.pixels.chunks_exact_mut(WIDTH))
            .enumerate()
        {
            let current = PaintedRow {
                geometry: *row,
                colors: [
                    appearance.background,
                    appearance.foreground,
                    picture.objects[0].line.color,
                    picture.objects[1].line.color,
                ],
                foreground_priority: appearance.walls_in_front,
                visible_width: picture.visible_widths[y],
            };
            if self.composed[y] == Some(current) {
                if let Some(first) = changed_start.take() {
                    self.frame.changed_rows.push(first..y);
                }
                continue;
            }
            changed_start.get_or_insert(y);
            pixels.fill(appearance.background);
            let foreground = row.walls.union(row.player);
            foreground.paint(pixels, appearance.foreground);

            // The first object wins over the second; foreground priority applies to both.
            for slot in [1, 0] {
                let mut mask = row.objects[slot].union(row.barriers[slot]);
                if appearance.walls_in_front {
                    mask = mask.without(foreground);
                }
                mask.paint(pixels, picture.objects[slot].line.color);
            }
            pixels[usize::from(current.visible_width).min(WIDTH)..].fill(0);
            self.composed[y] = Some(current);
        }
        if let Some(first) = changed_start {
            self.frame.changed_rows.push(first..HEIGHT);
        }
        self.frame.painted_lines = HEIGHT;
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
