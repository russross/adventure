//! Functional scanline composition and contacts; no CPU clocks or bus registers.

pub const WIDTH: usize = 160;
pub const HEIGHT: usize = 194;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PlayfieldRow {
    pub edge: u8,
    pub middle: u8,
    pub inner: u8,
}

impl PlayfieldRow {
    // The stored twenty cells have three bit orders; each cell covers four pixels.
    fn covers(self, x: usize, mirrored: bool) -> bool {
        let mut cell = x / 4;
        if cell >= 20 {
            cell = if mirrored { 39 - cell } else { cell - 20 };
        }
        match cell {
            0..=3 => self.edge & (1 << (cell + 4)) != 0,
            4..=11 => self.middle & (1 << (11 - cell)) != 0,
            _ => self.inner & (1 << (cell - 12)) != 0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum Scale {
    Normal = 1,
    Quadruple = 4,
}

#[derive(Clone, Copy, Debug)]
pub struct Sprite {
    x: i16,
    pub color: u8,
    row: u8,
    staged_row: u8,
    delayed: bool,
    scale: Scale,
}

impl Default for Sprite {
    fn default() -> Self {
        Self {
            x: 0,
            color: 0,
            row: 0,
            staged_row: 0,
            delayed: false,
            scale: Scale::Normal,
        }
    }
}

impl Sprite {
    pub fn reposition(&mut self, x: u8) {
        self.x = positioned_object_x(x);
    }
    pub fn set_color(&mut self, color: u8) {
        self.color = color;
    }
    pub fn scale(&mut self, scale: Scale) {
        self.scale = scale;
    }
    pub fn delay_rows(&mut self, delayed: bool) {
        self.delayed = delayed;
    }

    // The second object stages its next row; committing the first row also commits that stage.
    pub fn set_row(&mut self, row: u8) {
        self.staged_row = row;
        if !self.delayed {
            self.row = row;
        }
    }

    fn commit_row(&mut self) {
        self.row = self.staged_row;
    }

    fn covers(self, x: usize) -> bool {
        // Enlarged TIA sprites start one pixel later; apply that shift before screen wrapping.
        let shift = match self.scale {
            Scale::Normal => 0,
            Scale::Quadruple => 1,
        };
        let relative = (x as i16 - self.x - shift).rem_euclid(WIDTH as i16);
        relative < self.scale as i16 * 8 && self.row & (0x80 >> (relative / self.scale as i16)) != 0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SolidObject {
    x: i16,
    width: u8,
    enabled: bool,
}

impl SolidObject {
    pub fn new(width: u8) -> Self {
        Self {
            x: 0,
            width,
            enabled: false,
        }
    }
    pub fn reposition(&mut self, x: u8) {
        self.x = positioned_object_x(x) - 1;
    }
    pub fn set_width(&mut self, width: u8) {
        self.width = width;
    }
    pub fn enable(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn covers(self, x: usize) -> bool {
        self.enabled && (x as i16 - self.x).rem_euclid(WIDTH as i16) < i16::from(self.width)
    }
}

// Adventure's positioning routine gives sprites a different origin from solid drawables.
// Large byte coordinates move the reset into horizontal blanking or the following line.
fn positioned_object_x(x: u8) -> i16 {
    match x {
        0..=164 => i16::from(x) - 1,
        165..=239 => i16::from(x % 15) - 4,
        240..=255 => i16::from(x) - 229,
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

pub struct Tia {
    pub first_object: Sprite,
    pub second_object: Sprite,
    pub player: SolidObject,
    pub left_barrier: SolidObject,
    pub right_barrier: SolidObject,
    pub contacts: Contacts,
    pub frame: Frame,
    walls: PlayfieldRow,
    background: u8,
    foreground: u8,
    mirrored: bool,
    walls_in_front: bool,
    blanked: bool,
}

impl Default for Tia {
    fn default() -> Self {
        Self {
            first_object: Sprite::default(),
            second_object: Sprite::default(),
            player: SolidObject::new(4),
            left_barrier: SolidObject::new(2),
            right_barrier: SolidObject::new(2),
            contacts: Contacts::default(),
            frame: Frame::default(),
            walls: PlayfieldRow::default(),
            background: 0,
            foreground: 0,
            mirrored: true,
            walls_in_front: false,
            blanked: true,
        }
    }
}

impl Tia {
    pub fn background(&mut self, color: u8) {
        self.background = color;
    }
    pub fn foreground(&mut self, color: u8) {
        self.foreground = color;
    }
    pub fn background_pattern(&mut self, walls: PlayfieldRow) {
        self.walls = walls;
    }
    pub fn mirror_walls(&mut self, mirrored: bool) {
        self.mirrored = mirrored;
    }
    pub fn foreground_priority(&mut self, in_front: bool) {
        self.walls_in_front = in_front;
    }
    pub fn blank(&mut self, blanked: bool) {
        self.blanked = blanked;
    }
    pub fn begin_picture(&mut self) {
        self.frame.painted_lines = 0;
    }

    pub fn first_object_row(&mut self, row: u8) {
        self.first_object.set_row(row);
        self.second_object.commit_row();
    }

    // Horizontal sync paints the accumulated line. Contacts accumulate even when a layer is hidden.
    pub fn horizontal_sync(&mut self) {
        let line = self.frame.painted_lines;
        if line >= HEIGHT {
            return;
        }
        for x in 0..WIDTH {
            let walls = !self.blanked && self.walls.covers(x, self.mirrored);
            let player = !self.blanked && self.player.covers(x);
            let first = !self.blanked && self.first_object.covers(x);
            let second = !self.blanked && self.second_object.covers(x);
            let left = !self.blanked && self.left_barrier.covers(x);
            let right = !self.blanked && self.right_barrier.covers(x);
            self.contacts.player_walls |= player && walls;
            self.contacts.player_barriers[0] |= player && left;
            self.contacts.player_barriers[1] |= player && right;
            self.contacts.player_objects[0] |= player && first;
            self.contacts.player_objects[1] |= player && second;
            self.contacts.objects_touch |= first && second;

            // Resolve the foreground group and object groups in the original priority order.
            let foreground = walls || player;
            let color = if self.blanked {
                0
            } else if self.walls_in_front && foreground {
                self.foreground
            } else if first || left {
                self.first_object.color
            } else if second || right {
                self.second_object.color
            } else if foreground {
                self.foreground
            } else {
                self.background
            };
            self.frame.pixels[line * WIDTH + x] = color;
        }
        self.frame.painted_lines += 1;
    }
}
