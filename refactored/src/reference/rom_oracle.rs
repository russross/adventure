//! Test-only execution of the supplied ROM to compare persistent game behavior.

use adventure_port::tia::{Contacts, PlayfieldRow, Scale, Tia};

pub struct Raster {
    pub tia: Tia,
    walls: PlayfieldRow,
    visible: bool,
}

impl Raster {
    pub fn new(memory: &[u8; 65536]) -> Self {
        let mut tia = Tia::default();
        tia.foreground(memory[8]);
        tia.background(memory[9]);
        tia.mirror_walls(memory[10] & 1 != 0);
        tia.foreground_priority(memory[10] & 4 != 0);
        tia.player.set_width(1 << ((memory[10] >> 4) & 3));
        tia.left_barrier.enable(memory[0x1d] & 2 != 0);
        tia.right_barrier.enable(memory[0x1e] & 2 != 0);
        tia.first_object.set_color(memory[6]);
        tia.second_object.set_color(memory[7]);
        tia.first_object.scale(if memory[4] & 7 == 7 {
            Scale::Quadruple
        } else {
            Scale::Normal
        });
        tia.second_object.scale(if memory[5] & 7 == 7 {
            Scale::Quadruple
        } else {
            Scale::Normal
        });
        Self {
            tia,
            walls: PlayfieldRow::default(),
            visible: false,
        }
    }

    // Translate original output writes into the same functional hardware boundary used by the port.
    fn write(&mut self, address: u16, value: u8) {
        match address {
            1 => {
                let visible = value & 2 == 0;
                if self.visible && !visible {
                    self.tia.horizontal_sync();
                }
                if !self.visible && visible {
                    self.tia.begin_picture();
                }
                self.visible = visible;
                self.tia.blank(!visible);
            }
            2 => {
                if self.visible {
                    self.tia.horizontal_sync();
                }
            }
            0x0d => {
                self.walls.edge = value;
                self.tia.background_pattern(self.walls);
            }
            0x0e => {
                self.walls.middle = value;
                self.tia.background_pattern(self.walls);
            }
            0x0f => {
                self.walls.inner = value;
                self.tia.background_pattern(self.walls);
            }
            0x1b => self.tia.first_object_row(value),
            0x1c => self.tia.second_object.set_row(value),
            0x1f => self.tia.player.enable(value & 2 != 0),
            0x26 => self.tia.second_object.delay_rows(value & 1 != 0),
            0x2c => self.tia.contacts = Contacts::default(),
            _ => {}
        }
    }

    fn position(&mut self, drawable: u8, x: u8) {
        match drawable {
            0 => self.tia.first_object.reposition(x),
            1 => self.tia.second_object.reposition(x),
            2 => self.tia.left_barrier.reposition(x),
            3 => self.tia.right_barrier.reposition(x),
            4 => self.tia.player.reposition(x),
            _ => panic!("bad reference drawable"),
        }
    }
}

pub struct Cpu {
    pub memory: Box<[u8; 65536]>,
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub raster: Option<Raster>,
    pc: u16,
    carry: bool,
    zero: bool,
    negative: bool,
    overflow: bool,
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
    pub fn new() -> Self {
        let mut memory = Box::new([0; 65536]);
        memory[0xf000..].copy_from_slice(include_bytes!("../../../target/refactored-oracle.bin"));
        Self {
            memory,
            a: 0,
            x: 0,
            y: 0,
            raster: None,
            pc: 0,
            carry: false,
            zero: false,
            negative: false,
            overflow: false,
        }
    }

    fn fetch(&mut self) -> u8 {
        let byte = self.memory[usize::from(self.pc)];
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
            Mode::Absx => self.word().wrapping_add(u16::from(self.x)),
            Mode::Absy => self.word().wrapping_add(u16::from(self.y)),
            Mode::Indx => {
                let pointer = self.fetch().wrapping_add(self.x);
                u16::from(self.memory[usize::from(pointer)])
                    | u16::from(self.memory[usize::from(pointer.wrapping_add(1))]) << 8
            }
            Mode::Indy => {
                let pointer = self.fetch();
                let address = u16::from(self.memory[usize::from(pointer)])
                    | u16::from(self.memory[usize::from(pointer.wrapping_add(1))]) << 8;
                address.wrapping_add(u16::from(self.y))
            }
            Mode::Ind => {
                let pointer = self.word();
                let high = (pointer & 0xff00) | (pointer.wrapping_add(1) & 255);
                u16::from(self.memory[usize::from(pointer)])
                    | u16::from(self.memory[usize::from(high)]) << 8
            }
        }
    }

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

    fn write(&mut self, address: u16, value: u8) {
        if address >= 0xf000 {
            return;
        }
        let value = match address {
            0x15 | 0x16 | 0x19 | 0x1a => value & 15,
            0x17 | 0x18 => value & 31,
            _ => value,
        };
        self.memory[usize::from(address)] = value;
        if let Some(raster) = &mut self.raster {
            raster.write(address, value);
        }
    }

    // Calls use a real nested continuation stack; the first RTS completes this subroutine.
    pub fn call(&mut self, entry: u16) {
        self.pc = entry;
        let mut calls = Vec::new();
        for _ in 0..200_000 {
            let opcode = self.fetch();
            let (operation, mode) = decode(opcode);
            let address = self.address(mode);
            let value = if mode == Mode::Imp {
                self.a
            } else {
                self.memory[usize::from(address)]
            };
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
                Op::Sta => self.write(address, self.a),
                Op::Stx => self.write(address, self.x),
                Op::Sty => self.write(address, self.y),
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
                        self.write(address, result);
                    }
                    self.flags(result);
                }
                Op::Inc | Op::Dec => {
                    let result = if matches!(operation, Op::Inc) {
                        value.wrapping_add(1)
                    } else {
                        value.wrapping_sub(1)
                    };
                    self.write(address, result);
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
                    if address == 0xf0d2
                        && let Some(raster) = &mut self.raster
                    {
                        raster.position(self.x, self.a);
                        continue;
                    }
                    calls.push(self.pc);
                    self.pc = address;
                }
                Op::Rts => {
                    if let Some(continuation) = calls.pop() {
                        self.pc = continuation;
                    } else {
                        return;
                    }
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
                        self.pc = self.pc.wrapping_add_signed(i16::from(value as i8));
                    }
                }
                Op::Sei | Op::Cli | Op::Cld | Op::Nop => {}
                Op::Brk
                | Op::Sed
                | Op::Pha
                | Op::Php
                | Op::Pla
                | Op::Plp
                | Op::Rti
                | Op::Tsx
                | Op::Txs => panic!(
                    "unexpected stack/decimal operation in reference routine at {:04x}",
                    self.pc
                ),
            }
        }
        panic!("reference routine {entry:04x} did not return");
    }
}

fn decode(opcode: u8) -> (Op, Mode) {
    match opcode {
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
        _ => panic!("unsupported reference opcode {opcode:02x}"),
    }
}
