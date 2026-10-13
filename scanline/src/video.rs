//! Game-specific scanline composition and accumulated geometric contacts.

pub const WIDTH: usize = 160;
pub const HEIGHT: usize = 194;

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

#[derive(Clone, Copy, Debug, Default)]
pub enum SpriteWidth {
    #[default]
    Normal,
    Wide,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ObjectLine {
    pub x: u8,
    pub color: u8,
    pub width: SpriteWidth,
    pub bits: u8,
}

impl ObjectLine {
    // Coordinates are the actual left edge; artwork wraps at the drawable screen edge.
    fn covers(self, x: usize) -> bool {
        let scale = match self.width {
            SpriteWidth::Normal => 1,
            SpriteWidth::Wide => 4,
        };
        let relative = (x as i16 - i16::from(self.x)).rem_euclid(WIDTH as i16);
        relative < scale * 8 && self.bits & (0x80 >> (relative / scale)) != 0
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Appearance {
    pub background: u8,
    pub foreground: u8,
    pub mirrored: bool,
    pub walls_in_front: bool,
    pub player_width: u8,
    pub barriers: [bool; 2],
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Scanline {
    pub appearance: Appearance,
    pub walls: PlayfieldRow,
    pub objects: [ObjectLine; 2],
    pub player_x: u8,
    pub player_visible: bool,
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
}

impl Video {
    pub fn begin_picture(&mut self) {
        self.frame.painted_lines = 0;
        self.contacts = Contacts::default();
    }

    // A complete line has no staged rows or register side effects. Hidden layers still collide.
    pub fn paint(&mut self, line: &Scanline) {
        let row = self.frame.painted_lines;
        assert!(row < HEIGHT, "picture exceeds its scanline budget");
        let appearance = line.appearance;
        for x in 0..WIDTH {
            let walls = line.walls.covers(x, appearance.mirrored);
            let player = line.player_visible && solid(x, line.player_x, appearance.player_width);
            let first = line.objects[0].covers(x);
            let second = line.objects[1].covers(x);
            let left = appearance.barriers[0] && solid(x, 11, 2);
            let right = appearance.barriers[1] && solid(x, 148, 2);
            self.contacts.player_walls |= player && walls;
            self.contacts.player_barriers[0] |= player && left;
            self.contacts.player_barriers[1] |= player && right;
            self.contacts.player_objects[0] |= player && first;
            self.contacts.player_objects[1] |= player && second;
            self.contacts.objects_touch |= first && second;

            // Walls and the player share a color; each barrier shares its object's color.
            let foreground = walls || player;
            self.frame.pixels[row * WIDTH + x] = if appearance.walls_in_front && foreground {
                appearance.foreground
            } else if first || left {
                line.objects[0].color
            } else if second || right {
                line.objects[1].color
            } else if foreground {
                appearance.foreground
            } else {
                appearance.background
            };
        }
        self.frame.painted_lines += 1;
    }
}

fn solid(pixel: usize, x: u8, width: u8) -> bool {
    (pixel as i16 - i16::from(x)).rem_euclid(WIDTH as i16) < i16::from(width)
}

#[cfg(test)]
#[path = "tests/video.rs"]
mod tests;
