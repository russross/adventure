//! Collision-only TIA: register strobes and pixel clocks produce latches without a framebuffer.

pub struct Tia {
    registers: [u8; 0x30],
    positions: [i16; 5],
    delayed: u8,
    contacts: [u8; 8],
    pub cycles: u64,
}
impl Default for Tia {
    fn default() -> Self {
        Self {
            registers: [0; 0x30],
            positions: [0; 5],
            delayed: 0,
            contacts: [0; 8],
            cycles: 0,
        }
    }
}
impl Tia {
    pub fn diagnostics(&self) -> String {
        format!(
            "positions={:?}, contacts={:?}, clock={}",
            self.positions, self.contacts, self.cycles
        )
    }
    pub fn contact(&self, register: usize) -> u8 {
        self.contacts[register]
    }

    // Evaluate hidden overlapping pixels as well as visible ones; priority and color cannot affect contact.
    pub fn advance(&mut self, count: u64) {
        for _ in 0..count {
            let clock = (self.cycles % 76) as i16 * 3;
            if self.registers[1] & 2 == 0 {
                for x in clock - 68..clock - 65 {
                    if !(0..160).contains(&x) {
                        continue;
                    }
                    let p0 = self.sprite(0, x);
                    let p1 = self.sprite(1, x);
                    let m0 = self.solid(
                        2,
                        x,
                        self.registers[0x1d] & 2 != 0,
                        1 << (self.registers[4] >> 4 & 3),
                    );
                    let m1 = self.solid(
                        3,
                        x,
                        self.registers[0x1e] & 2 != 0,
                        1 << (self.registers[5] >> 4 & 3),
                    );
                    let ball = self.solid(
                        4,
                        x,
                        self.registers[0x1f] & 2 != 0,
                        1 << (self.registers[10] >> 4 & 3),
                    );
                    let wall = self.wall(x);
                    self.contacts[2] |= u8::from(p0 && wall) << 7 | u8::from(p0 && ball) << 6;
                    self.contacts[3] |= u8::from(p1 && wall) << 7 | u8::from(p1 && ball) << 6;
                    self.contacts[4] |= u8::from(m0 && wall) << 7 | u8::from(m0 && ball) << 6;
                    self.contacts[5] |= u8::from(m1 && wall) << 7 | u8::from(m1 && ball) << 6;
                    self.contacts[6] |= u8::from(ball && wall) << 7;
                    self.contacts[7] |= u8::from(p0 && p1) << 7 | u8::from(m0 && m1) << 6;
                }
            }
            self.cycles += 1;
        }
    }
    fn solid(&self, id: usize, x: i16, enabled: bool, width: i16) -> bool {
        enabled && (x - self.positions[id]).rem_euclid(160) < width
    }
    fn sprite(&self, id: usize, x: i16) -> bool {
        let width = if self.registers[4 + id] & 7 == 7 {
            4
        } else {
            1
        };
        let offset = (x - self.positions[id] - i16::from(width == 4)).rem_euclid(160);
        if offset >= width * 8 {
            return false;
        }
        let row = if id == 1 && self.registers[0x26] & 1 != 0 {
            self.delayed
        } else {
            self.registers[0x1b + id]
        };
        row & (0x80 >> (offset / width)) != 0
    }
    #[cfg(test)]
    pub fn position(&self, id: usize) -> i16 {
        self.positions[id]
    }
    fn wall(&self, x: i16) -> bool {
        let mut cell = x / 4;
        if cell >= 20 {
            cell = if self.registers[10] & 1 != 0 {
                39 - cell
            } else {
                cell - 20
            };
        }
        match cell {
            0..=3 => self.registers[0x0d] & (1 << (4 + cell)) != 0,
            4..=11 => self.registers[0x0e] & (1 << (11 - cell)) != 0,
            _ => self.registers[0x0f] & (1 << (cell - 12)) != 0,
        }
    }

    // Reset strobes establish coarse positions; HMOVE applies the signed high-nibble adjustment.
    pub fn write(&mut self, register: usize, value: u8) {
        if register >= self.registers.len() {
            return;
        }
        self.registers[register] = value;
        match register {
            2 => {
                let rest = (76 - self.cycles % 76) % 76;
                self.advance(rest);
            }
            0x10..=0x14 => {
                let id = register - 0x10;
                let delay = if id < 2 { 5 } else { 4 };
                self.positions[id] = ((self.cycles % 76) as i16 * 3 - 68 + delay).rem_euclid(160);
            }
            0x1b => self.delayed = self.registers[0x1c],
            0x2a => {
                for id in 0..5 {
                    let motion = (self.registers[0x20 + id] as i8 >> 4) as i16;
                    self.positions[id] = (self.positions[id] - motion).rem_euclid(160);
                }
            }
            0x2b => self.registers[0x20..=0x24].fill(0),
            0x2c => self.contacts.fill(0),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Tia;

    // Layer priority can hide a sprite, but overlapping player pixels must still latch a contact.
    #[test]
    fn hidden_contacts_persist_until_cleared() {
        let mut tia = Tia::default();
        tia.write(1, 2);
        tia.advance(47);
        tia.write(0x10, 0);
        tia.write(0x14, 0);
        tia.write(0x20, 32);
        tia.write(0x24, 32);
        tia.write(0x2a, 0);
        tia.write(0x1b, 128);
        tia.write(0x1f, 2);
        tia.write(0x0a, 0x24);
        tia.write(0x0d, 0xf0);
        tia.write(0x0e, 0xff);
        tia.write(0x0f, 0xff);
        tia.write(2, 0);
        tia.write(1, 0);
        tia.advance(76);
        assert_ne!(tia.contact(2) & 64, 0);
        assert_ne!(tia.contact(6) & 128, 0);
        tia.write(1, 2);
        tia.advance(76);
        assert_ne!(tia.contact(2) & 64, 0);
        tia.write(0x2c, 0);
        assert_eq!(tia.contact(2), 0);
    }

    #[test]
    fn second_sprite_delay_commits_on_first_sprite_write() {
        let mut tia = Tia::default();
        tia.write(1, 2);
        tia.advance(47);
        tia.write(0x11, 0);
        tia.write(0x14, 0);
        tia.write(0x21, 32);
        tia.write(0x24, 32);
        tia.write(0x2a, 0);
        tia.write(0x1f, 2);
        tia.write(0x0a, 0x20);
        tia.write(0x26, 1);
        tia.write(0x1c, 128);
        tia.write(2, 0);
        tia.write(1, 0);
        tia.advance(76);
        assert_eq!(tia.contact(3) & 64, 0);
        tia.write(0x1b, 0);
        tia.advance(76);
        assert_ne!(tia.contact(3) & 64, 0);
    }
}

#[cfg(test)]
mod wide_tests {
    use super::Tia;

    // The independently captured wide-sprite seam places its first expanded pixel after the reset origin.
    #[test]
    fn wide_sprite_seam_preserves_its_delayed_origin() {
        let mut tia = Tia::default();
        tia.positions[0] = 159;
        tia.positions[4] = 159;
        tia.write(4, 7);
        tia.write(0x1b, 128);
        tia.write(0x1f, 2);
        tia.advance(76);
        assert_eq!(tia.contact(2) & 64, 0);
        tia.positions[4] = 0;
        tia.write(0x2c, 0);
        tia.advance(76);
        assert_ne!(tia.contact(2) & 64, 0);
    }
}
