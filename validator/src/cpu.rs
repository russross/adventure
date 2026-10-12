//! CPU execution from the cartridge's reset vector, with actual stack bytes and cycle accounting.

use crate::bus::Bus;

pub struct Cpu {
    pub bus: Bus,
    pub pc: u16,
    a: u8,
    x: u8,
    y: u8,
    sp: u8,
    carry: bool,
    zero: bool,
    negative: bool,
    overflow: bool,
    crossed: bool,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Imp,
    Imm,
    Zp,
    Zpx,
    Zpy,
    Abs,
    Absx,
    Absy,
    Indx,
    Indy,
    Ind,
    Rel,
}

#[derive(Clone, Copy)]
enum Op {
    Adc,
    And,
    Cmp,
    Eor,
    Lda,
    Ora,
    Sbc,
    Sta,
    Asl,
    Lsr,
    Rol,
    Ror,
    Inc,
    Dec,
    Ldx,
    Ldy,
    Stx,
    Sty,
    Bit,
    Cpx,
    Cpy,
    Jmp,
    Jsr,
    Bcc,
    Bcs,
    Beq,
    Bmi,
    Bne,
    Bpl,
    Bvc,
    Bvs,
    Brk,
    Clc,
    Cld,
    Cli,
    Clv,
    Dex,
    Dey,
    Inx,
    Iny,
    Nop,
    Pha,
    Php,
    Pla,
    Plp,
    Rti,
    Rts,
    Sec,
    Sed,
    Sei,
    Tax,
    Tay,
    Tsx,
    Txa,
    Txs,
    Tya,
}

impl Cpu {
    pub fn new(bus: Bus) -> Self {
        let pc = u16::from_le_bytes([bus.memory[0xfffc], bus.memory[0xfffd]]);
        Self {
            bus,
            pc,
            a: 0,
            x: 0,
            y: 0,
            sp: 0,
            carry: false,
            zero: false,
            negative: false,
            overflow: false,
            crossed: false,
        }
    }
    fn indexed(&mut self, base: u16, offset: u8) -> u16 {
        let address = base.wrapping_add(u16::from(offset));
        self.crossed = base & 0xff00 != address & 0xff00;
        address
    }
    fn push(&mut self, value: u8) {
        self.bus.write(0x100 + u16::from(self.sp), value);
        self.sp = self.sp.wrapping_sub(1);
    }
    fn pop(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        self.bus.peek(0x100 + u16::from(self.sp))
    }
    // Operand fetches preserve cartridge addresses; indexed data accesses retain page-cross information.
    fn fetch(&mut self) -> u8 {
        let byte = self.bus.memory[usize::from(self.pc)];
        self.pc = self.pc.wrapping_add(1);
        byte
    }

    fn word(&mut self) -> u16 {
        u16::from(self.fetch()) | u16::from(self.fetch()) << 8
    }

    fn address(&mut self, mode: Mode) -> u16 {
        match mode {
            Mode::Imp => 0,
            Mode::Imm | Mode::Rel => {
                let address = self.pc;
                self.pc += 1;
                address
            }
            Mode::Zp => u16::from(self.fetch()),
            Mode::Zpx => u16::from(self.fetch().wrapping_add(self.x)),
            Mode::Zpy => u16::from(self.fetch().wrapping_add(self.y)),
            Mode::Abs => self.word(),
            Mode::Absx => {
                let base = self.word();
                self.indexed(base, self.x)
            }
            Mode::Absy => {
                let base = self.word();
                self.indexed(base, self.y)
            }
            Mode::Indx => {
                let pointer = self.fetch().wrapping_add(self.x);
                u16::from(self.bus.memory[usize::from(pointer)])
                    | u16::from(self.bus.memory[usize::from(pointer.wrapping_add(1))]) << 8
            }
            Mode::Indy => {
                let pointer = self.fetch();
                let address = u16::from(self.bus.memory[usize::from(pointer)])
                    | u16::from(self.bus.memory[usize::from(pointer.wrapping_add(1))]) << 8;
                self.indexed(address, self.y)
            }
            Mode::Ind => {
                let pointer = self.word();
                let high = (pointer & 0xff00) | (pointer.wrapping_add(1) & 255);
                u16::from(self.bus.memory[usize::from(pointer)])
                    | u16::from(self.bus.memory[usize::from(high)]) << 8
            }
        }
    }

    // Arithmetic keeps the processor flags needed by the original control flow.
    fn flags(&mut self, value: u8) {
        self.zero = value == 0;
        self.negative = value & 128 != 0;
    }

    fn compare(&mut self, left: u8, right: u8) {
        self.carry = left >= right;
        self.flags(left.wrapping_sub(right));
    }

    fn add(&mut self, right: u8) {
        let result = u16::from(self.a) + u16::from(right) + u16::from(self.carry);
        self.overflow = (!(self.a ^ right) & (self.a ^ result as u8) & 128) != 0;
        self.a = result as u8;
        self.carry = result > 255;
        self.flags(self.a);
    }

    // Game observation boundaries precede instruction execution; bus effects follow instruction timing.
    pub fn step(&mut self) -> Result<bool, String> {
        if !self.bus.before(self.pc, self.x, self.y)? {
            return Ok(false);
        }
        let start = self.pc;
        self.bus.pc = start;
        let opcode = self.fetch();
        let (operation, mode) = decode(opcode)?;
        self.crossed = false;
        let address = self.address(mode);
        let writes = matches!(operation, Op::Sta | Op::Stx | Op::Sty);
        let rmw = matches!(
            operation,
            Op::Asl | Op::Lsr | Op::Rol | Op::Ror | Op::Inc | Op::Dec
        ) && mode != Mode::Imp;
        let mut cycles = match mode {
            Mode::Imp | Mode::Imm | Mode::Rel => 2,
            Mode::Zp => 3,
            Mode::Zpx | Mode::Zpy | Mode::Abs => 4,
            Mode::Absx | Mode::Absy => {
                if writes {
                    5
                } else {
                    4 + u64::from(self.crossed)
                }
            }
            Mode::Indx => 6,
            Mode::Indy => {
                if writes {
                    6
                } else {
                    5 + u64::from(self.crossed)
                }
            }
            Mode::Ind => 5,
        };
        if rmw {
            cycles = match mode {
                Mode::Zp => 5,
                Mode::Zpx => 6,
                Mode::Abs => 6,
                _ => 7,
            };
        }
        if matches!(operation, Op::Jsr | Op::Rts) {
            cycles = 6;
        }
        if matches!(operation, Op::Jmp) && mode == Mode::Abs {
            cycles = 3;
        }
        self.bus.advance(cycles);
        let value = if mode == Mode::Imp {
            self.a
        } else if writes || matches!(operation, Op::Jmp | Op::Jsr) {
            0
        } else {
            self.bus.read(address)
        };
        if rmw {
            self.bus.write(address, value);
        }
        match operation {
            Op::Lda => {
                self.a = value;
                self.flags(value);
            }
            Op::Ldx => {
                self.x = value;
                self.flags(value);
            }
            Op::Ldy => {
                self.y = value;
                self.flags(value);
            }
            Op::Sta => self.bus.write(address, self.a),
            Op::Stx => self.bus.write(address, self.x),
            Op::Sty => self.bus.write(address, self.y),
            Op::Adc => self.add(value),
            Op::Sbc => self.add(!value),
            Op::And => {
                self.a &= value;
                self.flags(self.a);
            }
            Op::Ora => {
                self.a |= value;
                self.flags(self.a);
            }
            Op::Eor => {
                self.a ^= value;
                self.flags(self.a);
            }
            Op::Cmp => self.compare(self.a, value),
            Op::Cpx => self.compare(self.x, value),
            Op::Cpy => self.compare(self.y, value),
            Op::Bit => {
                self.zero = self.a & value == 0;
                self.negative = value & 128 != 0;
                self.overflow = value & 64 != 0;
            }
            Op::Asl | Op::Lsr | Op::Rol | Op::Ror => {
                let previous_carry = u8::from(self.carry);
                let result = match operation {
                    Op::Asl => {
                        self.carry = value & 128 != 0;
                        value << 1
                    }
                    Op::Lsr => {
                        self.carry = value & 1 != 0;
                        value >> 1
                    }
                    Op::Rol => {
                        self.carry = value & 128 != 0;
                        value << 1 | previous_carry
                    }
                    Op::Ror => {
                        self.carry = value & 1 != 0;
                        value >> 1 | previous_carry << 7
                    }
                    _ => unreachable!(),
                };
                if mode == Mode::Imp {
                    self.a = result;
                } else {
                    self.bus.write(address, result);
                }
                self.flags(result);
            }
            Op::Inc | Op::Dec => {
                let result = if matches!(operation, Op::Inc) {
                    value.wrapping_add(1)
                } else {
                    value.wrapping_sub(1)
                };
                self.bus.write(address, result);
                self.flags(result);
            }
            Op::Inx => {
                self.x = self.x.wrapping_add(1);
                self.flags(self.x);
            }
            Op::Iny => {
                self.y = self.y.wrapping_add(1);
                self.flags(self.y);
            }
            Op::Dex => {
                self.x = self.x.wrapping_sub(1);
                self.flags(self.x);
            }
            Op::Dey => {
                self.y = self.y.wrapping_sub(1);
                self.flags(self.y);
            }
            Op::Tax => {
                self.x = self.a;
                self.flags(self.x);
            }
            Op::Tay => {
                self.y = self.a;
                self.flags(self.y);
            }
            Op::Txa => {
                self.a = self.x;
                self.flags(self.a);
            }
            Op::Tya => {
                self.a = self.y;
                self.flags(self.a);
            }
            Op::Clc => self.carry = false,
            Op::Sec => self.carry = true,
            Op::Clv => self.overflow = false,
            Op::Jmp => self.pc = address,
            Op::Jsr => {
                let ret = self.pc.wrapping_sub(1);
                self.push((ret >> 8) as u8);
                self.push(ret as u8);
                self.pc = address;
            }
            Op::Rts => {
                let lo = self.pop();
                let hi = self.pop();
                self.pc = u16::from_le_bytes([lo, hi]).wrapping_add(1);
            }
            Op::Bcc | Op::Bcs | Op::Beq | Op::Bmi | Op::Bne | Op::Bpl | Op::Bvc | Op::Bvs => {
                let take = match operation {
                    Op::Bcc => !self.carry,
                    Op::Bcs => self.carry,
                    Op::Beq => self.zero,
                    Op::Bmi => self.negative,
                    Op::Bne => !self.zero,
                    Op::Bpl => !self.negative,
                    Op::Bvc => !self.overflow,
                    Op::Bvs => self.overflow,
                    _ => unreachable!(),
                };
                if take {
                    let target = self.pc.wrapping_add_signed(i16::from(value as i8));
                    self.bus
                        .advance(1 + u64::from(target & 0xff00 != self.pc & 0xff00));
                    self.pc = target;
                }
            }
            Op::Txs => self.sp = self.x,
            Op::Sei | Op::Cli | Op::Cld | Op::Nop => {}
            _ => return Err(format!("unsupported execution at {start:04x}")),
        }
        self.bus.after(start, self.x)?;
        self.bus
            .session
            .check()
            .map_err(|e| format!("{e}; {}", self.bus.diagnostics()))?;
        Ok(true)
    }
}

fn decode(opcode: u8) -> Result<(Op, Mode), String> {
    Ok(match opcode {
        0x00 => (Op::Brk, Mode::Imp),
        0x01 => (Op::Ora, Mode::Indx),
        0x05 => (Op::Ora, Mode::Zp),
        0x06 => (Op::Asl, Mode::Zp),
        0x08 => (Op::Php, Mode::Imp),
        0x09 => (Op::Ora, Mode::Imm),
        0x0a => (Op::Asl, Mode::Imp),
        0x0d => (Op::Ora, Mode::Abs),
        0x0e => (Op::Asl, Mode::Abs),
        0x10 => (Op::Bpl, Mode::Rel),
        0x11 => (Op::Ora, Mode::Indy),
        0x15 => (Op::Ora, Mode::Zpx),
        0x16 => (Op::Asl, Mode::Zpx),
        0x18 => (Op::Clc, Mode::Imp),
        0x19 => (Op::Ora, Mode::Absy),
        0x1d => (Op::Ora, Mode::Absx),
        0x1e => (Op::Asl, Mode::Absx),
        0x20 => (Op::Jsr, Mode::Abs),
        0x21 => (Op::And, Mode::Indx),
        0x24 => (Op::Bit, Mode::Zp),
        0x25 => (Op::And, Mode::Zp),
        0x26 => (Op::Rol, Mode::Zp),
        0x28 => (Op::Plp, Mode::Imp),
        0x29 => (Op::And, Mode::Imm),
        0x2a => (Op::Rol, Mode::Imp),
        0x2c => (Op::Bit, Mode::Abs),
        0x2d => (Op::And, Mode::Abs),
        0x2e => (Op::Rol, Mode::Abs),
        0x30 => (Op::Bmi, Mode::Rel),
        0x31 => (Op::And, Mode::Indy),
        0x35 => (Op::And, Mode::Zpx),
        0x36 => (Op::Rol, Mode::Zpx),
        0x38 => (Op::Sec, Mode::Imp),
        0x39 => (Op::And, Mode::Absy),
        0x3d => (Op::And, Mode::Absx),
        0x3e => (Op::Rol, Mode::Absx),
        0x40 => (Op::Rti, Mode::Imp),
        0x41 => (Op::Eor, Mode::Indx),
        0x45 => (Op::Eor, Mode::Zp),
        0x46 => (Op::Lsr, Mode::Zp),
        0x48 => (Op::Pha, Mode::Imp),
        0x49 => (Op::Eor, Mode::Imm),
        0x4a => (Op::Lsr, Mode::Imp),
        0x4c => (Op::Jmp, Mode::Abs),
        0x4d => (Op::Eor, Mode::Abs),
        0x4e => (Op::Lsr, Mode::Abs),
        0x50 => (Op::Bvc, Mode::Rel),
        0x51 => (Op::Eor, Mode::Indy),
        0x55 => (Op::Eor, Mode::Zpx),
        0x56 => (Op::Lsr, Mode::Zpx),
        0x58 => (Op::Cli, Mode::Imp),
        0x59 => (Op::Eor, Mode::Absy),
        0x5d => (Op::Eor, Mode::Absx),
        0x5e => (Op::Lsr, Mode::Absx),
        0x60 => (Op::Rts, Mode::Imp),
        0x61 => (Op::Adc, Mode::Indx),
        0x65 => (Op::Adc, Mode::Zp),
        0x66 => (Op::Ror, Mode::Zp),
        0x68 => (Op::Pla, Mode::Imp),
        0x69 => (Op::Adc, Mode::Imm),
        0x6a => (Op::Ror, Mode::Imp),
        0x6c => (Op::Jmp, Mode::Ind),
        0x6d => (Op::Adc, Mode::Abs),
        0x6e => (Op::Ror, Mode::Abs),
        0x70 => (Op::Bvs, Mode::Rel),
        0x71 => (Op::Adc, Mode::Indy),
        0x75 => (Op::Adc, Mode::Zpx),
        0x76 => (Op::Ror, Mode::Zpx),
        0x78 => (Op::Sei, Mode::Imp),
        0x79 => (Op::Adc, Mode::Absy),
        0x7d => (Op::Adc, Mode::Absx),
        0x7e => (Op::Ror, Mode::Absx),
        0x81 => (Op::Sta, Mode::Indx),
        0x84 => (Op::Sty, Mode::Zp),
        0x85 => (Op::Sta, Mode::Zp),
        0x86 => (Op::Stx, Mode::Zp),
        0x88 => (Op::Dey, Mode::Imp),
        0x8a => (Op::Txa, Mode::Imp),
        0x8c => (Op::Sty, Mode::Abs),
        0x8d => (Op::Sta, Mode::Abs),
        0x8e => (Op::Stx, Mode::Abs),
        0x90 => (Op::Bcc, Mode::Rel),
        0x91 => (Op::Sta, Mode::Indy),
        0x94 => (Op::Sty, Mode::Zpx),
        0x95 => (Op::Sta, Mode::Zpx),
        0x96 => (Op::Stx, Mode::Zpy),
        0x98 => (Op::Tya, Mode::Imp),
        0x99 => (Op::Sta, Mode::Absy),
        0x9a => (Op::Txs, Mode::Imp),
        0x9d => (Op::Sta, Mode::Absx),
        0xa0 => (Op::Ldy, Mode::Imm),
        0xa1 => (Op::Lda, Mode::Indx),
        0xa2 => (Op::Ldx, Mode::Imm),
        0xa4 => (Op::Ldy, Mode::Zp),
        0xa5 => (Op::Lda, Mode::Zp),
        0xa6 => (Op::Ldx, Mode::Zp),
        0xa8 => (Op::Tay, Mode::Imp),
        0xa9 => (Op::Lda, Mode::Imm),
        0xaa => (Op::Tax, Mode::Imp),
        0xac => (Op::Ldy, Mode::Abs),
        0xad => (Op::Lda, Mode::Abs),
        0xae => (Op::Ldx, Mode::Abs),
        0xb0 => (Op::Bcs, Mode::Rel),
        0xb1 => (Op::Lda, Mode::Indy),
        0xb4 => (Op::Ldy, Mode::Zpx),
        0xb5 => (Op::Lda, Mode::Zpx),
        0xb6 => (Op::Ldx, Mode::Zpy),
        0xb8 => (Op::Clv, Mode::Imp),
        0xb9 => (Op::Lda, Mode::Absy),
        0xba => (Op::Tsx, Mode::Imp),
        0xbc => (Op::Ldy, Mode::Absx),
        0xbd => (Op::Lda, Mode::Absx),
        0xbe => (Op::Ldx, Mode::Absy),
        0xc0 => (Op::Cpy, Mode::Imm),
        0xc1 => (Op::Cmp, Mode::Indx),
        0xc4 => (Op::Cpy, Mode::Zp),
        0xc5 => (Op::Cmp, Mode::Zp),
        0xc6 => (Op::Dec, Mode::Zp),
        0xc8 => (Op::Iny, Mode::Imp),
        0xc9 => (Op::Cmp, Mode::Imm),
        0xca => (Op::Dex, Mode::Imp),
        0xcc => (Op::Cpy, Mode::Abs),
        0xcd => (Op::Cmp, Mode::Abs),
        0xce => (Op::Dec, Mode::Abs),
        0xd0 => (Op::Bne, Mode::Rel),
        0xd1 => (Op::Cmp, Mode::Indy),
        0xd5 => (Op::Cmp, Mode::Zpx),
        0xd6 => (Op::Dec, Mode::Zpx),
        0xd8 => (Op::Cld, Mode::Imp),
        0xd9 => (Op::Cmp, Mode::Absy),
        0xdd => (Op::Cmp, Mode::Absx),
        0xde => (Op::Dec, Mode::Absx),
        0xe0 => (Op::Cpx, Mode::Imm),
        0xe1 => (Op::Sbc, Mode::Indx),
        0xe4 => (Op::Cpx, Mode::Zp),
        0xe5 => (Op::Sbc, Mode::Zp),
        0xe6 => (Op::Inc, Mode::Zp),
        0xe8 => (Op::Inx, Mode::Imp),
        0xe9 => (Op::Sbc, Mode::Imm),
        0xea => (Op::Nop, Mode::Imp),
        0xec => (Op::Cpx, Mode::Abs),
        0xed => (Op::Sbc, Mode::Abs),
        0xee => (Op::Inc, Mode::Abs),
        0xf0 => (Op::Beq, Mode::Rel),
        0xf1 => (Op::Sbc, Mode::Indy),
        0xf5 => (Op::Sbc, Mode::Zpx),
        0xf6 => (Op::Inc, Mode::Zpx),
        0xf8 => (Op::Sed, Mode::Imp),
        0xf9 => (Op::Sbc, Mode::Absy),
        0xfd => (Op::Sbc, Mode::Absx),
        0xfe => (Op::Inc, Mode::Absx),
        _ => return Err(format!("unsupported opcode {opcode:02x}")),
    })
}

#[cfg(test)]
mod tests {
    use super::{Bus, Cpu};
    use adventure_trace::Session;

    // Execute the cartridge's own positioning loop; indexed strobe stores affect the resulting origin.
    #[test]
    fn cartridge_positioning_matches_calibrated_origins() {
        for (id, x, expected) in [(0, 80, 79), (4, 80, 78), (2, 13, 11), (3, 150, 148)] {
            let mut recording = Session::record();
            recording.finish().unwrap();
            let mut cpu = Cpu::new(Bus::new(recording.records().to_vec()).unwrap());
            cpu.pc = 0xf0d2;
            cpu.a = x;
            cpu.x = id;
            cpu.sp = 255;
            cpu.push(0xef);
            cpu.push(0xff);
            for _ in 0..1000 {
                assert!(cpu.step().unwrap());
                if cpu.pc == 0xf000 {
                    break;
                }
            }
            assert_eq!(cpu.pc, 0xf000);
            // The routine leaves fine motion for the subsequent shared HMOVE strobe.
            cpu.bus.write(0x2a, 0);
            assert_eq!(cpu.bus.drawable_position(usize::from(id)), expected);
            assert_eq!(cpu.sp, 255);
        }
    }
}
