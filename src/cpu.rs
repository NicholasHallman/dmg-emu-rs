use crate::mem::Mem;

pub enum Reg {
    AF,
    BC,
    DE,
    HL,
    SP,
    PC
}

impl From<&Reg> for String {
    fn from(r: &Reg) -> Self {
        match r {
            Reg::AF => "AF".to_string(),
            Reg::BC => "BC".to_string(),
            Reg::DE => "DE".to_string(),
            Reg::HL => "HL".to_string(),
            Reg::SP => "SP".to_string(),
            Reg::PC => "PC".to_string(),
        }
    }
}

pub enum HalfReg {
    A,
    F,
    B,
    C,
    D,
    E,
    H,
    L,
}

pub enum Flag {
    Z,
    N,
    H,
    C
}

pub struct Cpu {
    // registers
    pub AF: u16, 
    pub BC: u16, 
    pub DE: u16, 
    pub HL: u16, 
    pub SP: u16, 
    pub PC: u16,
    pub ime: bool,
    // cycle tracking
    pub current_cycle: i32,
    pub current_op: u8,
    store: [u8; 5],

    pub is_halt: bool,
    pub is_stop: bool,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            AF: 0,
            BC: 0,
            DE: 0,
            HL: 0,
            SP: 0,
            PC: 0,
            current_cycle: 1,
            current_op: 0,
            store: [0; 5],
            is_halt: false,
            is_stop: false,
            ime: false
        }
    }

    pub fn get_op(&mut self, mem: &Mem) -> u8 {
        let pc = self.PC;
        self.PC += 1;
        mem.get(pc)
    }

    pub fn reset(&mut self) {
        self.current_cycle = 0;
    }

    pub fn interupt(&mut self, mem: &mut Mem, isr: u16) {
        let pc = self.PC;
        mem.set(self.SP - 1, (pc >> 8) as u8);
        mem.set(self.SP - 2, (pc) as u8);
        self.SP -= 2;
        self.PC = isr;
    }

    pub fn check_interupts(&mut self, mem: &mut Mem) {
        // check for interupts
        if !self.ime {return}

        let enabled = mem.get(0xFFFF); // is the interupt enabled
        let flags = mem.get(0xFF0F); // was the interupt triggered

        if flags & 1 == 1 && enabled & 1 == 1 { // vblank
            self.interupt(mem, 0x40);
            self.ime = false;
            mem.set(0xFF0F, flags & !1);

        } else if flags & 2 == 2 && enabled & 2 == 2 { // lcd stat
            self.interupt(mem, 0x48);
            self.ime = false;
            mem.set(0xFF0F, flags & !2);

        } else if flags & 4 == 4 && enabled & 4 == 4 { // timer
            self.interupt(mem, 0x50);
            self.ime = false;
            mem.set(0xFF0F, flags & !4);

        } else if flags & 8 == 8 && enabled & 8 == 8 { // serial
            self.interupt(mem, 0x58);
            self.ime = false;
            mem.set(0xFF0F, flags & !8);

        } else if flags & 16 == 16 && enabled & 16 == 16 { // joypad
            self.interupt(mem, 0x60);
            self.ime = false;
            mem.set(0xFF0F, flags & !16);
        }
    }

    pub fn tick(&mut self, mem: &mut Mem) {
        
        match self.current_cycle { 
            1 => { // start decoding new op
                self.current_op = self.get_op(mem);
                self.execute(mem);
            },
            _ => { // continue executing
                self.execute(mem);
            }
        }

        if self.current_cycle == 0 { // finished execution
            self.check_interupts(mem); 
        }
        self.current_cycle += 1;
    }

    pub fn set_word_reg(&mut self, reg: &Reg, value: u16) {
        match reg {
            Reg::AF => self.AF = value,
            Reg::BC => self.BC = value,
            Reg::DE => self.DE = value,
            Reg::HL => self.HL = value,
            Reg::SP => self.SP = value,
            Reg::PC => self.PC = value,
        };
    }

    pub fn get_word_reg(&self, reg: &Reg) -> u16 {
        match reg {
            Reg::AF => self.AF,
            Reg::BC => self.BC,
            Reg::DE => self.DE,
            Reg::HL => self.HL,
            Reg::SP => self.SP,
            Reg::PC => self.PC
        }
    }

    pub fn clear_reg(&mut self, reg: &HalfReg) {
        match reg {
            HalfReg::A => self.AF = (self.AF << 8) >> 8,
            HalfReg::F => self.AF = (self.AF >> 8) << 8,
            HalfReg::B => self.BC = (self.BC << 8) >> 8,
            HalfReg::C => self.BC = (self.BC >> 8) << 8,
            HalfReg::D => self.DE = (self.DE << 8) >> 8,
            HalfReg::E => self.DE = (self.DE >> 8) << 8,
            HalfReg::H => self.HL = (self.HL << 8) >> 8,
            HalfReg::L => self.HL = (self.HL >> 8) << 8,
        }
    }

    pub fn set_byte_reg(&mut self, reg: &HalfReg, value: u8) {
        self.clear_reg(reg);
        match reg {
            HalfReg::A => self.AF |= (value as u16) << 8,
            HalfReg::F => self.AF |= value as u16,
            HalfReg::B => self.BC |= (value as u16) << 8,
            HalfReg::C => self.BC |= value as u16,
            HalfReg::D => self.DE |= (value as u16) << 8,
            HalfReg::E => self.DE |= value as u16,
            HalfReg::H => self.HL |= (value as u16) << 8,
            HalfReg::L => self.HL |= value as u16,
        }
    }

    pub fn get_byte_reg(&self, reg: &HalfReg) -> u8 {
        match reg {
            HalfReg::A => (self.AF >> 8) as u8,
            HalfReg::F => self.AF as u8,
            HalfReg::B => (self.BC >> 8) as u8,
            HalfReg::C => self.BC as u8,
            HalfReg::D => (self.DE >> 8) as u8,
            HalfReg::E => self.DE as u8,
            HalfReg::H => (self.HL >> 8) as u8,
            HalfReg::L => self.HL as u8,
        }
    }

    pub fn get_flag(&self, flag: Flag) -> bool {
        match flag {
            Flag::Z => self.AF & 0b0000000010000000 == 0x80,
            Flag::N => self.AF & 0b0000000001000000 == 0x40,
            Flag::H => self.AF & 0b0000000000100000 == 0x20,
            Flag::C => self.AF & 0b0000000000010000 == 0x10,
        }
    }

    pub fn set_flag(&mut self, flag: Flag, value: bool) {
        match value {
            true => {
                match flag {
                    Flag::Z => self.AF |= 0b0000000010000000,
                    Flag::N => self.AF |= 0b0000000001000000,
                    Flag::H => self.AF |= 0b0000000000100000,
                    Flag::C => self.AF |= 0b0000000000010000,
                }
            },
            false => {
                match flag {
                    Flag::Z => self.AF &= !0b0000000010000000,
                    Flag::N => self.AF &= !0b0000000001000000,
                    Flag::H => self.AF &= !0b0000000000100000,
                    Flag::C => self.AF &= !0b0000000000010000,
                }
            }
        }
    }

    pub fn set_bhca(&mut self, a: u8, b: u8) -> bool {
        if ((a & 0xF) + (b & 0xF)) & 0x10 == 0x10 {
            self.set_flag(Flag::H, true);
            return true;
        }
        self.set_flag(Flag::H, false);
        false
    }

    pub fn set_bhcs(&mut self, a: u8, b: u8) -> bool {
        if (a & 0xF).overflowing_sub(b & 0xF).0 & 0x10 == 0x10 {
            self.set_flag(Flag::H, true);
            return true;
        }
        self.set_flag(Flag::H, false);
        return false;
    }

    pub fn set_4hca(&mut self, a: u16, b: u16) -> bool {
        if ((a & 0xF) + (b & 0xF)) & 0x10 == 0x10 {
            self.set_flag(Flag::H, true);
            return true;
        }
        self.set_flag(Flag::H, false);
        return false;
    }

    pub fn set_4hcs(&mut self, a: u16, b: u16) -> bool {
        if (a & 0xF).overflowing_sub(b & 0xF).0 & 0x10 == 0x10 {
            self.set_flag(Flag::H, true);
            return true;
        }
        self.set_flag(Flag::H, false);
        return false;
    }

    pub fn set_12hca(&mut self, a: u16, b: u16) -> bool {
        if ((a & 0x0FFF) + (b & 0x0FFF)) & 0x1000 == 0x1000 {
            self.set_flag(Flag::H, true);
            return true;
        }
        self.set_flag(Flag::H, false);
        return false;
    }

    pub fn set_12hcs(&mut self, a: u16, b: u16) -> bool {
        if (a & 0x0FFF).overflowing_sub(b & 0x0FFF).0 & 0x1000 == 0x1000 {
            self.set_flag(Flag::H, true);
            return true;
        }
        self.set_flag(Flag::H, false);
        return false;
    }

    pub fn execute(&mut self, mem: &mut Mem) {
        match self.current_op {
            0x00 => self.noop(),
            0x01 => self.ld_rr_d16(mem, &Reg::BC),
            0x02 => self.ld_ar_a(&Reg::BC, mem),
            0x03 => self.inc_rr(&Reg::BC),
            0x04 => self.inc_r(&HalfReg::B),
            0x05 => self.dec_r(&HalfReg::B),
            0x06 => self.ld_r_d8(mem, &HalfReg::B),
            0x07 => self.rlca(),
            0x08 => self.ld_a16_sp(mem),
            0x09 => self.add_rr_rr(&Reg::HL, &Reg::BC),
            0x0A => self.ld_a_rr(mem, &Reg::BC),
            0x0B => self.dec_rr(&Reg::BC),
            0x0C => self.inc_r(&HalfReg::C),
            0x0D => self.dec_r(&HalfReg::C),
            0x0E => self.ld_r_d8(mem, &HalfReg::C),
            0x0F => self.rrca(),

            0x10 => self.stop(),
            0x11 => self.ld_rr_d16(mem, &Reg::DE),
            0x12 => self.ld_ar_a(&Reg::DE, mem),
            0x13 => self.inc_rr(&Reg::DE),
            0x14 => self.inc_r(&HalfReg::D),
            0x15 => self.dec_r(&HalfReg::D),
            0x16 => self.ld_r_d8(mem, &HalfReg::D),
            0x17 => self.rla(),
            0x18 => self.jr(mem),
            0x19 => self.add_rr_rr(&Reg::HL, &Reg::DE),
            0x1A => self.ld_a_rr(mem, &Reg::DE),
            0x1B => self.dec_rr(&Reg::DE),
            0x1C => self.inc_r(&HalfReg::E),
            0x1D => self.dec_r(&HalfReg::E),
            0x1E => self.ld_r_d8(mem, &HalfReg::E),
            0x1F => self.rra(),

            0x20 => self.jr_con(mem, Flag::Z, false),
            0x21 => self.ld_rr_d16(mem, &Reg::HL),
            0x22 => self.ld_hl_inc_a(mem),
            0x23 => self.inc_rr(&Reg::HL),
            0x24 => self.inc_r(&HalfReg::H),
            0x25 => self.dec_r(&HalfReg::H),
            0x26 => self.ld_r_d8(mem, &HalfReg::H),
            0x27 => self.daa(),
            0x28 => self.jr_con(mem, Flag::Z, true),
            0x29 => self.add_rr_rr(&Reg::HL, &Reg::HL),
            0x2A => self.ld_a_hl_inc(mem),
            0x2B => self.dec_rr(&Reg::HL),
            0x2C => self.inc_r(&HalfReg::L),
            0x2D => self.dec_r(&HalfReg::L),
            0x2E => self.ld_r_d8(mem, &HalfReg::L),
            0x2F => self.cpl(),
            
            0x30 => self.jr_con(mem, Flag::C, false),
            0x31 => self.ld_rr_d16(mem, &Reg::SP),
            0x32 => self.ld_hl_dec_a(mem),
            0x33 => self.inc_rr(&Reg::SP),
            0x34 => self.inc_hl(mem),
            0x35 => self.dec_hl(mem),
            0x36 => self.ld_hl_d8(mem),
            0x37 => self.scf(),
            0x38 => self.jr_con(mem, Flag::C, true),
            0x39 => self.add_rr_rr(&Reg::HL, &Reg::SP),
            0x3A => self.ld_a_hl_dec(mem),
            0x3B => self.dec_rr(&Reg::SP),
            0x3C => self.inc_r(&HalfReg::A),
            0x3D => self.dec_r(&HalfReg::A),
            0x3E => self.ld_r_d8(mem, &HalfReg::A),
            0x3F => self.ccf(),

            0x40 => self.ld_r_r( &HalfReg::B, &HalfReg::B),
            0x41 => self.ld_r_r( &HalfReg::B, &HalfReg::C),
            0x42 => self.ld_r_r( &HalfReg::B, &HalfReg::D),
            0x43 => self.ld_r_r( &HalfReg::B, &HalfReg::E),
            0x44 => self.ld_r_r( &HalfReg::B, &HalfReg::H),
            0x45 => self.ld_r_r( &HalfReg::B, &HalfReg::L),
            0x46 => self.ld_r_hl(&HalfReg::B, mem),
            0x47 => self.ld_r_r( &HalfReg::B, &HalfReg::A),
            0x48 => self.ld_r_r( &HalfReg::C, &HalfReg::B),
            0x49 => self.ld_r_r( &HalfReg::C, &HalfReg::C),
            0x4A => self.ld_r_r( &HalfReg::C, &HalfReg::D),
            0x4B => self.ld_r_r( &HalfReg::C, &HalfReg::E),
            0x4C => self.ld_r_r( &HalfReg::C, &HalfReg::H),
            0x4D => self.ld_r_r( &HalfReg::C, &HalfReg::L),
            0x4E => self.ld_r_hl(&HalfReg::C, mem),
            0x4F => self.ld_r_r( &HalfReg::C, &HalfReg::A),

            0x50 => self.ld_r_r( &HalfReg::D, &HalfReg::B),
            0x51 => self.ld_r_r( &HalfReg::D, &HalfReg::C),
            0x52 => self.ld_r_r( &HalfReg::D, &HalfReg::D),
            0x53 => self.ld_r_r( &HalfReg::D, &HalfReg::E),
            0x54 => self.ld_r_r( &HalfReg::D, &HalfReg::H),
            0x55 => self.ld_r_r( &HalfReg::D, &HalfReg::L),
            0x56 => self.ld_r_hl(&HalfReg::D, mem),
            0x57 => self.ld_r_r( &HalfReg::D, &HalfReg::A),
            0x58 => self.ld_r_r( &HalfReg::E, &HalfReg::B),
            0x59 => self.ld_r_r( &HalfReg::E, &HalfReg::C),
            0x5A => self.ld_r_r( &HalfReg::E, &HalfReg::D),
            0x5B => self.ld_r_r( &HalfReg::E, &HalfReg::E),
            0x5C => self.ld_r_r( &HalfReg::E, &HalfReg::H),
            0x5D => self.ld_r_r( &HalfReg::E, &HalfReg::L),
            0x5E => self.ld_r_hl(&HalfReg::E, mem),
            0x5F => self.ld_r_r( &HalfReg::E, &HalfReg::A),

            0x60 => self.ld_r_r( &HalfReg::H, &HalfReg::B),
            0x61 => self.ld_r_r( &HalfReg::H, &HalfReg::C),
            0x62 => self.ld_r_r( &HalfReg::H, &HalfReg::D),
            0x63 => self.ld_r_r( &HalfReg::H, &HalfReg::E),
            0x64 => self.ld_r_r( &HalfReg::H, &HalfReg::H),
            0x65 => self.ld_r_r( &HalfReg::H, &HalfReg::L),
            0x66 => self.ld_r_hl(&HalfReg::H, mem),
            0x67 => self.ld_r_r( &HalfReg::H, &HalfReg::A),
            0x68 => self.ld_r_r( &HalfReg::L, &HalfReg::B),
            0x69 => self.ld_r_r( &HalfReg::L, &HalfReg::C),
            0x6A => self.ld_r_r( &HalfReg::L, &HalfReg::D),
            0x6B => self.ld_r_r( &HalfReg::L, &HalfReg::E),
            0x6C => self.ld_r_r( &HalfReg::L, &HalfReg::H),
            0x6D => self.ld_r_r( &HalfReg::L, &HalfReg::L),
            0x6E => self.ld_r_hl(&HalfReg::L, mem),
            0x6F => self.ld_r_r( &HalfReg::L, &HalfReg::A),

            0x70 => self.ld_hl_r(mem, &HalfReg::B),
            0x71 => self.ld_hl_r(mem, &HalfReg::C),
            0x72 => self.ld_hl_r(mem, &HalfReg::D),
            0x73 => self.ld_hl_r(mem, &HalfReg::E),
            0x74 => self.ld_hl_r(mem, &HalfReg::H),
            0x75 => self.ld_hl_r(mem, &HalfReg::L),
            0x76 => self.halt(),
            0x77 => self.ld_hl_r(mem, &HalfReg::A),
            0x78 => self.ld_r_r( &HalfReg::A, &HalfReg::B),
            0x79 => self.ld_r_r( &HalfReg::A, &HalfReg::C),
            0x7A => self.ld_r_r( &HalfReg::A, &HalfReg::D),
            0x7B => self.ld_r_r( &HalfReg::A, &HalfReg::E),
            0x7C => self.ld_r_r( &HalfReg::A, &HalfReg::H),
            0x7D => self.ld_r_r( &HalfReg::A, &HalfReg::L),
            0x7E => self.ld_r_hl(&HalfReg::A, mem),
            0x7F => self.ld_r_r( &HalfReg::A, &HalfReg::A),

            0x80 => self.add_a(&HalfReg::B),
            0x81 => self.add_a(&HalfReg::C),
            0x82 => self.add_a(&HalfReg::D),
            0x83 => self.add_a(&HalfReg::E),
            0x84 => self.add_a(&HalfReg::H),
            0x85 => self.add_a(&HalfReg::L),
            0x86 => self.add_m(mem),
            0x87 => self.add_a(&HalfReg::A),
            0x88 => self.addc_a(&HalfReg::B),
            0x89 => self.addc_a(&HalfReg::C),
            0x8A => self.addc_a(&HalfReg::D),
            0x8B => self.addc_a(&HalfReg::E),
            0x8C => self.addc_a(&HalfReg::H),
            0x8D => self.addc_a(&HalfReg::L),
            0x8E => self.addc_m(mem),
            0x8F => self.addc_a(&HalfReg::A),

            0x90 => self.sub_a(&HalfReg::B),
            0x91 => self.sub_a(&HalfReg::C),
            0x92 => self.sub_a(&HalfReg::D),
            0x93 => self.sub_a(&HalfReg::E),
            0x94 => self.sub_a(&HalfReg::H),
            0x95 => self.sub_a(&HalfReg::L),
            0x96 => self.sub_m(mem),
            0x97 => self.sub_a(&HalfReg::A),
            0x98 => self.subc_a(&HalfReg::B),
            0x99 => self.subc_a(&HalfReg::C),
            0x9A => self.subc_a(&HalfReg::D),
            0x9B => self.subc_a(&HalfReg::E),
            0x9C => self.subc_a(&HalfReg::H),
            0x9D => self.subc_a(&HalfReg::L),
            0x9E => self.subc_m(mem),
            0x9F => self.subc_a(&HalfReg::A),

            0xA0 => self.and_a(&HalfReg::B),
            0xA1 => self.and_a(&HalfReg::C),
            0xA2 => self.and_a(&HalfReg::D),
            0xA3 => self.and_a(&HalfReg::E),
            0xA4 => self.and_a(&HalfReg::H),
            0xA5 => self.and_a(&HalfReg::L),
            0xA6 => self.and_m(mem),
            0xA7 => self.and_a(&HalfReg::A),
            0xA8 => self.xor_a(&HalfReg::B),
            0xA9 => self.xor_a(&HalfReg::C),
            0xAA => self.xor_a(&HalfReg::D),
            0xAB => self.xor_a(&HalfReg::E),
            0xAC => self.xor_a(&HalfReg::H),
            0xAD => self.xor_a(&HalfReg::L),
            0xAE => self.xor_m(mem),
            0xAF => self.xor_a(&HalfReg::A),

            0xB0 => self.or_a(&HalfReg::B),
            0xB1 => self.or_a(&HalfReg::C),
            0xB2 => self.or_a(&HalfReg::D),
            0xB3 => self.or_a(&HalfReg::E),
            0xB4 => self.or_a(&HalfReg::H),
            0xB5 => self.or_a(&HalfReg::L),
            0xB6 => self.or_m(mem),
            0xB7 => self.or_a(&HalfReg::A),
            0xB8 => self.cp_a(&HalfReg::B),
            0xB9 => self.cp_a(&HalfReg::C),
            0xBA => self.cp_a(&HalfReg::D),
            0xBB => self.cp_a(&HalfReg::E),
            0xBC => self.cp_a(&HalfReg::H),
            0xBD => self.cp_a(&HalfReg::L),
            0xBE => self.cp_m(mem),
            0xBF => self.cp_a(&HalfReg::A),

            0xC0 => self.ret_con(mem, Flag::Z, false),
            0xC1 => self.pop(&Reg::BC, mem),
            0xC2 => self.jp_con(mem, Flag::Z, false),
            0xC3 => self.jp(mem),
            0xC4 => self.call_con(mem, Flag::Z, false),
            0xC5 => self.push(&Reg::BC, mem),
            0xC6 => self.add_d8(mem),
            0xC7 => self.rst(mem, 0),
            0xC8 => self.ret_con(mem, Flag::Z, true),
            0xC9 => self.ret(mem),
            0xCA => self.jp_con(mem, Flag::Z, true),
            0xCB => self.extended(mem),
            0xCC => self.call_con(mem, Flag::Z, true),
            0xCD => self.call(mem),
            0xCE => self.addc_d8(mem),
            0xCF => self.rst(mem, 0x08),

            0xD0 => self.ret_con(mem, Flag::C, false),
            0xD1 => self.pop(&Reg::DE, mem),
            0xD2 => self.jp_con(mem, Flag::C, false),
            0xD4 => self.call_con(mem, Flag::C, false),
            0xD5 => self.push(&Reg::DE, mem),
            0xD6 => self.sub_d8(mem),
            0xD7 => self.rst(mem, 0x10),
            0xD8 => self.ret_con(mem, Flag::C, true),
            0xD9 => self.reti(mem),
            0xDA => self.jp_con(mem, Flag::C, true),
            0xDC => self.call_con(mem, Flag::C, true),
            0xDE => self.subc_d8(mem),
            0xDF => self.rst(mem, 0x18),

            0xE0 => self.ld_d8_a(mem),
            0xE1 => self.pop(&Reg::HL, mem),
            0xE2 => self.ld_c_a(mem),
            0xE5 => self.push(&Reg::HL, mem),
            0xE6 => self.and_d8(mem),
            0xE7 => self.rst(mem, 0x20),
            0xE8 => self.add_sp_s8(mem),
            0xE9 => self.jp_hl(),
            0xEA => self.ld_a16_a(mem),
            0xEE => self.xor_d8(mem),
            0xEF => self.rst(mem, 0x28),

            0xF0 => self.ld_a_d8(mem),
            0xF1 => self.pop(&Reg::AF, mem),
            0xF2 => self.ld_a_c(mem),
            0xF3 => self.di(mem),
            0xF5 => self.push(&Reg::AF, mem),
            0xF6 => self.or_d8(mem),
            0xF7 => self.rst(mem, 0x30),
            0xF8 => self.ld_hl_sp_s8(mem),
            0xF9 => self.ld_sp_hl(),
            0xFA => self.ld_a_a16(mem),
            0xFB => self.ei(mem),
            0xFE => self.cp_d8(mem),
            0xFF => self.rst(mem, 0x38),

            _ => ()
        }
    }

    // OP CODES

    // SPECIAL

    pub fn noop(&mut self) {
        self.reset();
    }

    fn stop(&mut self) {
        self.is_stop = true;
        self.reset();
    }
    fn halt(&mut self) {
        self.is_halt = true;
        self.reset();
    }

    fn di(&mut self, mem: &mut Mem) {
        self.ime = false;
        self.reset();
    }

    fn ei(&mut self, mem: &mut Mem) {
        self.ime = true;
        self.reset();
    }

    // LOAD

    pub fn ld_rr_d16(&mut self, mem: &Mem, reg: &Reg) {
        match self.current_cycle {
            2 => {
                self.store[0] = self.get_op(mem);
            },
            3 => {
                self.store[1] = self.get_op(mem);
                self.set_word_reg(reg, (self.store[0] as u16) | (self.store[1] as u16) << 8);
                self.reset();
            }
            _ => ()
        }
    }

    fn ld_r_r(&mut self, to: &HalfReg, from: &HalfReg) {
        self.set_byte_reg(
            to, 
            self.get_byte_reg(from)
        );
        self.reset();
    }

    fn ld_r_hl(&mut self, to: &HalfReg, mem: &Mem, ) {
        match self.current_cycle {
            2 => {
                self.set_byte_reg(
                    to,
                    mem.get(self.HL) 
                );
                self.reset();
            },
            _ => ()
        }
    }

    fn ld_r_d8(&mut self,  mem: &Mem, to: &HalfReg) {
        match self.current_cycle {
            2 => {
                let value = self.get_op(mem);
                self.set_byte_reg(&to, value);
                self.reset();
            },
            _ => ()
        }
    }

    pub fn ld_a_hl_inc(&mut self, mem: &Mem) {
        match self.current_cycle {
            2 => {
                let addr = self.HL;
                let value = mem.get(addr);
                self.set_byte_reg(&HalfReg::A, value);
                self.HL += 1;
                self.reset();
            },
            _ => ()
        }
    }

    pub fn ld_a_hl_dec(&mut self, mem: &Mem) {
        match self.current_cycle {
            2 => {
                let addr = self.HL;
                let value = mem.get(addr);
                self.set_byte_reg(&HalfReg::A, value);
                self.HL -= 1;
                self.reset();
            },
            _ => (),
        }
    }

    pub fn ld_ar_a(&mut self, to_reg: &Reg, mem: &mut Mem) {
        match self.current_cycle {
            2 => {
                let value = self.get_byte_reg(&HalfReg::A);
                let addr = self.get_word_reg(to_reg);
                mem.set(addr, value);
                self.reset();
            },
            _ => ()
        }
    }

    fn ld_a16_sp(&mut self, mem: &mut Mem) {
        match self.current_cycle {
            2 => self.store[0] = self.get_op(mem),
            3 => self.store[1] = self.get_op(mem),
            4 => {
                let addr = (self.store[1] as u16) << 8 | (self.store[0] as u16);
                mem.set(addr, self.SP as u8);
            },
            5 => {
                let addr = (self.store[1] as u16) << 8 | (self.store[0] as u16);
                mem.set(addr + 1, (self.SP >> 8) as u8);
                self.reset();
            },
            _ => ()
        }
    }

    fn ld_d8_a(&mut self, mem: &mut Mem) {
        match self.current_cycle {
            2 => {
                self.store[0] = self.get_op(mem);
            },
            3 => {
                mem.set(
                    0xFF00 | (self.store[0] as u16), 
                    self.get_byte_reg(&HalfReg::A)
                );
                self.reset();
            },
            _ => ()
        }
    }

    fn ld_a_d8(&mut self, mem: &mut Mem) {
        match self.current_cycle {
            2 => {
                self.store[0] = self.get_op(mem);
            },
            3 => {
                self.set_byte_reg(
                    &HalfReg::A, 
                    mem.get(0xFF00 | (self.store[0] as u16))
                );
                self.reset();
            },
            _ => ()
        }
    }

    fn ld_c_a(&mut self, mem: &mut Mem) {
        match self.current_cycle {
            2 => {
                mem.set(
                    0xFF00 | (self.get_byte_reg(&HalfReg::C) as u16), 
                    self.get_byte_reg(&HalfReg::A)
                );
                self.reset();
            },
            _ => ()
        }
    }

    fn ld_a_c(&mut self, mem: &mut Mem) {
        match self.current_cycle {
            2 => {
                self.set_byte_reg(
                    &HalfReg::A, 
                    mem.get(0xFF00 | (self.get_byte_reg(&HalfReg::C) as u16))
                );
                self.reset();
            },
            _ => ()
        }
    }

    
    fn ld_a_rr(&mut self, mem: &Mem, from: &Reg) {
        match self.current_cycle {
            2 => {
                let addr = self.get_word_reg(from);
                let value = mem.get(addr);
                self.set_byte_reg(&HalfReg::A, value);
                self.reset();
            },
            _ => ()
        }
    }

    fn ld_hl_inc_a(&mut self, mem: &mut Mem) {
        match self.current_cycle {
            2 => {
                let addr = self.HL;
                let value = self.get_byte_reg(&HalfReg::A);
                mem.set(addr, value);
                self.HL += 1;
                self.reset();
            },
            _ => ()
        }
    }

    fn ld_hl_dec_a(&mut self, mem: &mut Mem) {
        match self.current_cycle {
            2 => {
                let addr = self.HL;
                let value = self.get_byte_reg(&HalfReg::A);
                mem.set(addr, value);
                self.HL -= 1;
                self.reset();
            },
            _ => ()
        }
    }

    fn ld_hl_d8(&mut self, mem: &mut Mem) {
        match self.current_cycle {
            2 => {
                self.store[0] = self.get_op(mem); // d8
            },  
            3 => {
                mem.set(self.HL, self.store[0]);
                self.reset();
            },
            _ => (),
        }
    }

    fn ld_hl_r(&mut self, mem: &mut Mem, from: &HalfReg) {
        match self.current_cycle {
            2 => {
                let value = self.get_byte_reg(from);
                mem.set(self.HL, value);
                self.reset();
            },
            _ => (),
        }
    }

    fn ld_a16_a(&mut self, mem: &mut Mem) {
        match self.current_cycle {
            2 => {
                self.store[0] = self.get_op(mem);
            },
            3 => {
                self.store[1] = self.get_op(mem);
            }, 
            4 => {
                mem.set(
                    (self.store[1] as u16) << 8 | self.store[0] as u16,
                     self.get_byte_reg(&HalfReg::A)
                );
                self.reset();
            },
            _ => ()
        }
    }

    fn ld_a_a16(&mut self, mem: &Mem) {
        match self.current_cycle {
            2 => {
                self.store[0] = self.get_op(mem);
            },
            3 => {
                self.store[1] = self.get_op(mem);
            }, 
            4 => {
                let value = mem.get(
                    ((self.store[1] as u16) << 8) | (self.store[0] as u16),
                );
                self.set_byte_reg(&HalfReg::A, value);
                self.reset();
            },
            _ => ()
        }
    }

    fn ld_hl_sp_s8(&mut self, mem: &Mem) {
        match self.current_cycle {
            2 => {
                self.store[0] = self.get_op(mem);
            },
            3 => {
                let sp = self.SP;
                let s8 = i16::from(self.store[0] as i8) as u16; 
                let result: (u16, bool) = sp.overflowing_add(s8);

                self.HL = result.0;
                self.set_flag(Flag::N, false);
                self.set_flag(Flag::Z, false);
                self.set_flag(Flag::C, (sp & 0xFF) + (s8 & 0xFF) > 0xFF);
                self.set_flag(Flag::H, (sp & 0xF) + (s8 & 0xF) > 0xF);
                self.reset();
            },
            _ => ()
        }
    }

    fn ld_sp_hl(&mut self) {
        match self.current_cycle {
            2 => {
                self.SP = self.HL;
                self.reset();
            },
            _ => (),
        }
    }

    // INC DEC

    pub fn inc_rr(&mut self, reg: &Reg) {
        match self.current_cycle {
            2 => {
                let value = self.get_word_reg(reg);
                let result = value.wrapping_add(1);

                self.set_word_reg(reg, result);
                self.reset()
            },
            _ => ()
        }
    }

    pub fn dec_rr(&mut self, reg: &Reg) {
        match self.current_cycle {
            2 => {
                let value = self.get_word_reg(reg);
                let (result, _) = value.overflowing_sub(1);
                self.set_word_reg(reg, result);
                self.reset()
            },
            _ => ()
        }
    }

    pub fn inc_r(&mut self, reg: &HalfReg) {
        let value = self.get_byte_reg(reg);
        let (result, _) = value.overflowing_add(1);
        self.set_byte_reg(reg, result);

        self.set_flag(Flag::Z, result == 0);
        self.set_flag(Flag::N, false);
        self.set_bhca(value, 1);
        self.reset();
    }

    pub fn dec_r(&mut self, reg: &HalfReg) {
        let value = self.get_byte_reg(reg);
        let (result, _) = value.overflowing_sub(1);
        self.set_byte_reg(reg, result);
        self.set_flag(Flag::Z, result == 0);
        self.set_flag(Flag::N, true);
        self.set_bhcs(value, 1);
        self.reset();
    }

    pub fn inc_hl(&mut self, mem: &mut Mem) {
        match self.current_cycle {
            2 => {
                self.store[0] = mem.get(self.HL).overflowing_add(1).0;
                self.set_bhca(mem.get(self.HL), 1);
                self.set_flag(Flag::N, false);
                self.set_flag(Flag::Z, self.store[0] == 0);
            },
            3 => {
                mem.set(self.HL, self.store[0]);
                self.reset();
            },
            _ => (),
        }
    }

    pub fn dec_hl(&mut self, mem: &mut Mem) {
        match self.current_cycle {
            2 => {
                self.store[0] = mem.get(self.HL).overflowing_sub(1).0;
                self.set_bhcs(mem.get(self.HL), 1);
                self.set_flag(Flag::N, true);
                self.set_flag(Flag::Z, self.store[0] == 0);
            },
            3 => {
                mem.set(self.HL, self.store[0]);
                self.reset();
            },
            _ => (),
        }
    }

    // ROTATES

    fn rlca(&mut self) {
        let value = self.get_byte_reg(&HalfReg::A).rotate_left(1);
        let b7 = value & 1;
        self.set_byte_reg(&HalfReg::A, value);
        self.set_flag(Flag::Z, false);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::H, false);
        self.set_flag(Flag::C, b7 == 1);
        self.reset();
    }

    fn rla(&mut self) {
        let mut value = self.get_byte_reg(&HalfReg::A);
        let b7 = (value >> 7) & 1;
        let c: u8 = if self.get_flag(Flag::C) {1} else {0};
        value = (value << 1) | c;
        self.set_flag(Flag::C, b7 == 1);
        self.set_flag(Flag::Z, false);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::H, false);

        self.set_byte_reg(&HalfReg::A, value);
        self.reset();
    }

    fn rrca(&mut self) {
        let value = self.get_byte_reg(&HalfReg::A).rotate_right(1);
        let b1 = (value >> 7) & 1;
        self.set_byte_reg(&HalfReg::A, value);
        self.set_flag(Flag::Z, false);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::H, false);
        self.set_flag(Flag::C, b1 == 1);
        self.reset();
    }

    fn rra(&mut self) {
        let mut value = self.get_byte_reg(&HalfReg::A);
        let b1 = value & 1;
        let c: u8 = if self.get_flag(Flag::C) {1} else {0};
        value = value >> 1;
        value |= c << 7;
        self.set_flag(Flag::C, b1 == 1);
        self.set_flag(Flag::Z, false);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::H, false);

        self.set_byte_reg(&HalfReg::A, value);
        self.reset();
    }

    // BCD

    fn daa(&mut self) {
        let n = self.get_flag(Flag::N);
        let c = self.get_flag(Flag::C);
        let h = self.get_flag(Flag::H);
        let mut a = self.get_byte_reg(&HalfReg::A);
        if !n {
            if c || a > 0x99 { a = a.overflowing_add(0x60).0; self.set_flag(Flag::C, true); }
            if h || (a & 0x0f) > 0x09 { a = a.overflowing_add(0x6).0; }
        }
        else {
            if c { a = a.overflowing_sub(0x60).0; }
            if h { a = a.overflowing_sub(0x6).0; }
        }
        self.set_flag(Flag::Z, a == 0);
        self.set_flag(Flag::H, false);

        self.set_byte_reg(&HalfReg::A, a);
        self.reset();
    }

    // FLAG FLIPS

    fn cpl(&mut self) {
        let value = self.get_byte_reg(&HalfReg::A);
        self.set_byte_reg(&HalfReg::A, !value);
        self.set_flag(Flag::N, true);
        self.set_flag(Flag::H, true);
        self.reset();
    }

    fn scf(&mut self) {
        self.set_flag(Flag::C, true);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::H, false);
        self.reset();
    }

    fn ccf(&mut self) {
        let c = self.get_flag(Flag::C);
        self.set_flag(Flag::C, !c);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::H, false);
        self.reset();
    }

    // JUMP

    fn jr(&mut self, mem: &Mem) {
        match self.current_cycle {
            2 => {
                self.store[0] = self.get_op(mem);
            },
            3 => {
                let mut pc = self.PC;
                let rel = self.store[0] as i8;
                if rel < 0 {
                    pc -= rel.abs() as u16;
                } else {
                    pc += rel as u16;
                }
                self.PC = pc;
                self.reset();
            },
            _ => (),
        }

    }

    fn jr_con(&mut self, mem: &Mem, con: Flag, value: bool) {
        match self.current_cycle {
            2 => {
                self.store[0] = self.get_op(mem); // s8
                let con = self.get_flag(con);
                if con != value {
                    self.reset()
                }
            },
            3 => {
                let mut pc = self.PC;
                let rel = self.store[0] as i8;
                if rel < 0 {
                    pc -= rel.abs() as u16;
                } else {
                    pc += rel as u16;
                }
                self.PC = pc;
                self.reset();
            },
            _ => ()
        }
    } 

    fn jp(&mut self, mem: &Mem) {
        match self.current_cycle {
            2 => {
                self.store[0] = self.get_op(mem);
            },
            3 => {
                self.store[1] = self.get_op(mem);
            },
            4 => {
                self.PC = (self.store[1] as u16) << 8 | (self.store[0] as u16);
                self.reset();
            },
            _ => ()
        }
    }

    fn jp_con(&mut self, mem: &Mem, flag: Flag, check: bool) {
        match self.current_cycle {
            2 => {
                self.store[0] = self.get_op(mem);
            },
            3 => {
                self.store[1] = self.get_op(mem);
                if self.get_flag(flag) != check {
                    self.reset();
                }
            },
            4 => {
                self.PC = (self.store[1] as u16) << 8 | (self.store[0] as u16);
                self.reset();
            },
            _ => ()
        }
    }

    fn jp_hl(&mut self) {
        self.PC = self.HL;
        self.reset();
    }

    // ALU Operations

    fn add_rr_rr(&mut self, to: &Reg, from: &Reg) {
        match self.current_cycle {
            2 => {
                let value_from = self.get_word_reg(from);
                let value_to = self.get_word_reg(to);
                let (result, overflow) = value_to.overflowing_add(value_from);

                self.set_flag(Flag::N, false);
                self.set_flag(Flag::C, overflow);
                self.set_12hca(value_to, value_from);
                self.set_word_reg(&to, result);
                self.reset();
            },
            _ => ()
        }
    }

    fn add_a(&mut self, from: &HalfReg) {
        let a = self.get_byte_reg(&HalfReg::A);
        let b = self.get_byte_reg(from);
        let (result, overflow) = a.overflowing_add(b);
        self.set_flag(Flag::C, overflow);
        self.set_flag(Flag::Z, result == 0);
        self.set_flag(Flag::N, false);
        self.set_bhca(a, b);

        self.set_byte_reg(&HalfReg::A, result);
        self.reset();
    }

    fn addc_a(&mut self, from: &HalfReg) {
        match self.current_cycle {
            2 => {
                let a = self.get_byte_reg(&HalfReg::A);
                let b = self.get_byte_reg(from);
                let c = if self.get_flag(Flag::C) {1} else {0};
                let (result1, overflow1) = a.overflowing_add(b);
                let (result, overflow2) = result1.overflowing_add(c);
                let hc1 = self.set_bhca(a, b);
                let hc2 = self.set_bhca(result1, c);
                self.set_flag(Flag::C, overflow1 || overflow2);
                self.set_flag(Flag::Z, result == 0);
                self.set_flag(Flag::N, false);
                self.set_flag(Flag::H, hc1 || hc2);
                
                self.set_byte_reg(&HalfReg::A, result);
                self.reset();
            },
            _ => {},
        }        
    }

    fn add_m(&mut self, mem: &Mem) {
        match self.current_cycle {
            2 => {
                let a = self.get_byte_reg(&HalfReg::A);
                let b = mem.get(self.HL);
                let (result, overflow) = a.overflowing_add(b);
                self.set_flag(Flag::C, overflow);
                self.set_flag(Flag::Z, result == 0);
                self.set_flag(Flag::N, false);
                self.set_bhca(a, b);

                self.set_byte_reg(&HalfReg::A, result);
                self.reset();
            },
            _ => ()
        }
    }

    fn add_d8(&mut self, mem: &Mem) {
        match self.current_cycle {
            2 => {
                let a = self.get_byte_reg(&HalfReg::A);
                let b = self.get_op(mem);
                let (result, overflow) = a.overflowing_add(b);
                self.set_flag(Flag::C, overflow);
                self.set_flag(Flag::Z, result == 0);
                self.set_flag(Flag::N, false);
                self.set_bhca(a, b);

                self.set_byte_reg(&HalfReg::A, result);
                self.reset();
            },
            _ => ()
        }
    }

    fn add_sp_s8(&mut self, mem: &Mem) {
        match self.current_cycle {
            2 => {
                self.store[0] = self.get_op(mem);
            },
            3 => {
                // do some 4 bit operation???
            },
            4 => {
                let sp = self.SP;
                let s8 = i16::from(self.store[0] as i8) as u16; 
                let result: (u16, bool) = sp.overflowing_add(s8);

                self.SP = result.0;
                self.set_flag(Flag::N, false);
                self.set_flag(Flag::Z, false);
                self.set_flag(Flag::C, (sp & 0xFF) + (s8 & 0xFF) > 0xFF);
                self.set_flag(Flag::H, (sp & 0xF) + (s8 & 0xF) > 0xF);
                self.reset();
            }
            _ => ()
        }
    }

    fn addc_m(&mut self, mem: &Mem) {
        match self.current_cycle {
            2 => {
                let a = self.get_byte_reg(&HalfReg::A);
                let b = mem.get(self.HL);
                let c = if self.get_flag(Flag::C) {1} else {0};
                let (result1, overflow1) = a.overflowing_add(b);
                let (result, overflow2) = result1.overflowing_add(c);
                let hc1 = self.set_bhca(a, b);
                let hc2 = self.set_bhca(result1, c);
                self.set_flag(Flag::C, overflow1 || overflow2);
                self.set_flag(Flag::Z, result == 0);
                self.set_flag(Flag::N, false);
                self.set_flag(Flag::H, hc1 || hc2);

                self.set_byte_reg(&HalfReg::A, result);
                self.reset();
            },
            _ => ()
        }
    }

    fn addc_d8(&mut self, mem: &Mem) {
        match self.current_cycle {
            2 => {
                let a = self.get_byte_reg(&HalfReg::A);
                let b = self.get_op(mem);
                let c = if self.get_flag(Flag::C) {1} else {0};
                let (result1, overflow1) = a.overflowing_add(b);
                let (result, overflow2) = result1.overflowing_add(c);
                let hc1 = self.set_bhca(a, b);
                let hc2 = self.set_bhca(result1, c);
                self.set_flag(Flag::C, overflow1 || overflow2);
                self.set_flag(Flag::Z, result == 0);
                self.set_flag(Flag::N, false);
                self.set_flag(Flag::H, hc1 || hc2);

                self.set_byte_reg(&HalfReg::A, result);
                self.reset();
            },
            _ => ()
        }
    }

    fn sub_a(&mut self, from: &HalfReg) {
        let a = self.get_byte_reg(&HalfReg::A);
        let b = self.get_byte_reg(from);
        let (result, overflow) = a.overflowing_sub(b);
        self.set_flag(Flag::C, overflow);
        self.set_flag(Flag::Z, result == 0);
        self.set_flag(Flag::N, true);
        self.set_bhcs(a, b);

        self.set_byte_reg(&HalfReg::A, result);
        self.reset();
    }

    fn subc_a(&mut self, from: &HalfReg) {
        let a = self.get_byte_reg(&HalfReg::A);
        let b = self.get_byte_reg(from);
        let c = if self.get_flag(Flag::C) {1} else {0};
        let (result1, overflow1) = a.overflowing_sub(b);
        let (result, overflow2) = result1.overflowing_sub(c);
        let hc1 = self.set_bhcs(a, b);
        let hc2 = self.set_bhcs(result1, c);
        self.set_flag(Flag::C, overflow1 || overflow2);
        self.set_flag(Flag::Z, result == 0);
        self.set_flag(Flag::N, true);
        self.set_flag(Flag::H, hc1 || hc2);

        self.set_byte_reg(&HalfReg::A, result);
        self.reset();
    }

    fn sub_m(&mut self, mem: &Mem) {
        match self.current_cycle {
            2 => {
                let a = self.get_byte_reg(&HalfReg::A);
                let b = mem.get(self.HL);
                let (result, overflow) = a.overflowing_sub(b);
                self.set_flag(Flag::C, overflow);
                self.set_flag(Flag::Z, result == 0);
                self.set_flag(Flag::N, true);
                self.set_bhcs(a, b);

                self.set_byte_reg(&HalfReg::A, result);
                self.reset();
            },
            _ => ()
        }
    }

    fn sub_d8(&mut self, mem: &Mem) {
        match self.current_cycle {
            2 => {
                let a = self.get_byte_reg(&HalfReg::A);
                let b = self.get_op(mem);
                let (result, overflow) = a.overflowing_sub(b);
                self.set_flag(Flag::C, overflow);
                self.set_flag(Flag::Z, result == 0);
                self.set_flag(Flag::N, true);
                self.set_bhcs(a, b);

                self.set_byte_reg(&HalfReg::A, result);
                self.reset();
            },
            _ => ()
        }
    }

    fn subc_m(&mut self, mem: &Mem) {
        match self.current_cycle {
            2 => {
                let a = self.get_byte_reg(&HalfReg::A);
                let b = mem.get(self.HL);
                let c = if self.get_flag(Flag::C) {1} else {0};
                let (result1, overflow1) = a.overflowing_sub(b);
                let (result, overflow2) = result1.overflowing_sub(c);
                let hc1 = self.set_bhcs(a, b);
                let hc2 = self.set_bhcs(result1, c);
                self.set_flag(Flag::C, overflow1 || overflow2);
                self.set_flag(Flag::Z, result == 0);
                self.set_flag(Flag::N, true);
                self.set_flag(Flag::H, hc1 || hc2);

                self.set_byte_reg(&HalfReg::A, result);
                self.reset();
            },
            _ => ()
        }
    }

    fn subc_d8(&mut self, mem: &Mem) {
        match self.current_cycle {
            2 => {
                let a = self.get_byte_reg(&HalfReg::A);
                let b = self.get_op(mem);
                let c = if self.get_flag(Flag::C) {1} else {0};
                let (result1, overflow1) = a.overflowing_sub(b);
                let (result, overflow2) = result1.overflowing_sub(c);
                let hc1 = self.set_bhcs(a, b);
                let hc2 = self.set_bhcs(result1, c);
                self.set_flag(Flag::C, overflow1 || overflow2);
                self.set_flag(Flag::Z, result == 0);
                self.set_flag(Flag::N, true);
                self.set_flag(Flag::H, hc1 || hc2);

                self.set_byte_reg(&HalfReg::A, result);
                self.reset();
            },
            _ => ()
        }
    }

    fn and_a(&mut self, from: &HalfReg) {
        let a = self.get_byte_reg(&HalfReg::A);
        let b = self.get_byte_reg(from);
        let result = a & b;
        self.set_flag(Flag::C, false);
        self.set_flag(Flag::Z, result == 0);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::H, true);

        self.set_byte_reg(&HalfReg::A, result);
        self.reset();
    }

    fn and_m(&mut self, mem: &Mem) {
        match self.current_cycle {
            2 => {
                let a = self.get_byte_reg(&HalfReg::A);
                let b = mem.get(self.HL);
                let result = a & b;
                self.set_flag(Flag::C, false);
                self.set_flag(Flag::Z, result == 0);
                self.set_flag(Flag::N, false);
                self.set_flag(Flag::H, true);

                self.set_byte_reg(&HalfReg::A, result);
                self.reset();
            },
            _ => ()
        }
    }

    fn and_d8(&mut self, mem: &Mem) {
        match self.current_cycle {
            2 => {
                let a = self.get_byte_reg(&HalfReg::A);
                let b = self.get_op(mem);
                let result = a & b;
                self.set_flag(Flag::C, false);
                self.set_flag(Flag::Z, result == 0);
                self.set_flag(Flag::N, false);
                self.set_flag(Flag::H, true);

                self.set_byte_reg(&HalfReg::A, result);
                self.reset();
            },
            _ => ()
        }
    }

    fn xor_a(&mut self, from: &HalfReg) {
        let a = self.get_byte_reg(&HalfReg::A);
        let b = self.get_byte_reg(from);
        let result = a ^ b;
        self.set_flag(Flag::C, false);
        self.set_flag(Flag::Z, result == 0);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::H, false);

        self.set_byte_reg(&HalfReg::A, result);
        self.reset();
    }

    fn xor_m(&mut self, mem: &Mem) {
        match self.current_cycle {
            2 => {
                let a = self.get_byte_reg(&HalfReg::A);
                let b = mem.get(self.HL);
                let result = a ^ b;
                self.set_flag(Flag::C, false);
                self.set_flag(Flag::Z, result == 0);
                self.set_flag(Flag::N, false);
                self.set_flag(Flag::H, false);

                self.set_byte_reg(&HalfReg::A, result);
                self.reset();
            },
            _ => ()
        }
    }

    fn xor_d8(&mut self, mem: &Mem) {
        match self.current_cycle {
            2 => {
                let a = self.get_byte_reg(&HalfReg::A);
                let b = self.get_op(mem);
                let result = a ^ b;
                self.set_flag(Flag::C, false);
                self.set_flag(Flag::Z, result == 0);
                self.set_flag(Flag::N, false);
                self.set_flag(Flag::H, false);

                self.set_byte_reg(&HalfReg::A, result);
                self.reset();
            },
            _ => ()
        }
    }

    fn or_a(&mut self, from: &HalfReg) {
        let a = self.get_byte_reg(&HalfReg::A);
        let b = self.get_byte_reg(from);
        let result = a | b;
        self.set_flag(Flag::C, false);
        self.set_flag(Flag::Z, result == 0);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::H, false);

        self.set_byte_reg(&HalfReg::A, result);
        self.reset();
    }

    fn or_m(&mut self, mem: &Mem) {
        match self.current_cycle {
            2 => {
                let a = self.get_byte_reg(&HalfReg::A);
                let b = mem.get(self.HL);
                let result = a | b;
                self.set_flag(Flag::C, false);
                self.set_flag(Flag::Z, result == 0);
                self.set_flag(Flag::N, false);
                self.set_flag(Flag::H, false);

                self.set_byte_reg(&HalfReg::A, result);
                self.reset();
            },
            _ => ()
        }
    }

    fn or_d8(&mut self, mem: &Mem) {
        match self.current_cycle {
            2 => {
                let a = self.get_byte_reg(&HalfReg::A);
                let b = self.get_op(mem);
                let result = a | b;
                self.set_flag(Flag::C, false);
                self.set_flag(Flag::Z, result == 0);
                self.set_flag(Flag::N, false);
                self.set_flag(Flag::H, false);

                self.set_byte_reg(&HalfReg::A, result);
                self.reset();
            },
            _ => ()
        }
    }

    fn cp_a(&mut self, from: &HalfReg) {
        let a = self.get_byte_reg(&HalfReg::A);
        let b = self.get_byte_reg(from);
        let (result, overflow) = a.overflowing_sub(b);
        self.set_flag(Flag::C, overflow);
        self.set_flag(Flag::Z, result == 0);
        self.set_flag(Flag::N, true);
        self.set_bhcs(a, b);
        self.reset();
    }

    fn cp_m(&mut self, mem: &Mem) {
        match self.current_cycle {
            2 => {
                let a = self.get_byte_reg(&HalfReg::A);
                let b = mem.get(self.HL);
                let (result, overflow) = a.overflowing_sub(b);
                self.set_flag(Flag::C, overflow);
                self.set_flag(Flag::Z, result == 0);
                self.set_flag(Flag::N, true);
                self.set_bhcs(a, b);
                self.reset();
            },
            _ => ()
        }
    }

    fn cp_d8(&mut self, mem: &Mem) {
        match self.current_cycle {
            2 => {
                let a = self.get_byte_reg(&HalfReg::A);
                let b = self.get_op(mem);
                let (result, overflow) = a.overflowing_sub(b);
                self.set_flag(Flag::C, overflow);
                self.set_flag(Flag::Z, result == 0);
                self.set_flag(Flag::N, true);
                self.set_bhcs(a, b);
                self.reset();
            },
            _ => ()
        }
    }

    // FUNCTION

    fn call(&mut self, mem: &mut Mem) {
        match self.current_cycle {
            2 => {
                self.store[0] = self.get_op(mem);
            },
            3 => {
                self.store[1] = self.get_op(mem);
            },
            4 => {
                self.SP -= 1;
                mem.set(self.SP, (self.PC >> 8) as u8);
            },
            5 => {
                self.SP -= 1;
                mem.set(self.SP, self.PC as u8);
            },
            6 => {
                self.PC = (self.store[1] as u16) << 8 | self.store[0] as u16;
                self.reset();
            },
            _ => ()
        }
    }

    fn call_con(&mut self, mem: &mut Mem, flag: Flag, check: bool) {
        match self.current_cycle {
            2 => {
                self.store[0] = self.get_op(mem);
            },
            3 => {
                self.store[1] = self.get_op(mem);
                if !self.get_flag(flag) == check {
                    self.reset();
                }
            },
            4 => {
                self.SP -= 1;
                mem.set(self.SP, (self.PC >> 8) as u8);
            },
            5 => {
                self.SP -= 1;
                mem.set(self.SP, self.PC as u8);
            },
            6 => {
                self.PC = (self.store[1] as u16) << 8 | self.store[0] as u16;
                self.reset();
            }
            _ => ()
        }
    }

    fn ret(&mut self, mem: &Mem) {
        match self.current_cycle {
            2 => {
                self.store[0] = mem.get(self.SP);
                self.SP += 1;
            },
            3 => {
                self.store[1] = mem.get(self.SP);
                self.SP += 1;
            },
            4 => {
                self.PC = (self.store[1] as u16) << 8 | self.store[0] as u16;
                self.reset();
            },
            _ => ()
        }
    }

    fn reti(&mut self, mem: &mut Mem) {
        match self.current_cycle {
            2 => {
                self.store[0] = mem.get(self.SP);
                self.SP += 1;
            },
            3 => {
                self.store[1] = mem.get(self.SP);
                self.SP += 1;
            },
            4 => {
                self.ime = true;
                self.PC = (self.store[1] as u16) << 8 | self.store[0] as u16;
                self.reset();
            },
            _ => ()
        }
    }

    fn ret_con(&mut self, mem: &Mem, flag: Flag, check: bool) {
        match self.current_cycle {
            2 => {
                if !(self.get_flag(flag) == check) {
                    self.reset();
                }
            },
            3 => {
                self.store[0] = mem.get(self.SP);
                self.SP += 1;
            },
            4 => {
                self.store[1] = mem.get(self.SP);
                self.SP += 1;
            },
            5 => {
                self.PC = (self.store[1] as u16) << 8 | self.store[0] as u16;
                self.reset();
            },
            _ => ()
        }
    }

    // INTERUPT

    fn rst(&mut self, mem: &mut Mem, counter: u16) {
        match self.current_cycle {
            2 => {
                mem.set(self.SP - 1, (self.PC >> 8) as u8);
            },
            3 => {
                mem.set(self.SP - 2, self.PC as u8);
            },
            4 => {
                self.SP -= 2;
                self.PC = 0x0000 | counter;
                self.reset();
            },
            _ => ()
        }
    }

    // STACK

    fn push(&mut self, reg: &Reg, mem: &mut Mem) {
        match self.current_cycle {
            2 => {
                let v = self.get_word_reg(reg);
                mem.set(self.SP - 1, (v >> 8) as u8)
            },
            3 => {
                let v = self.get_word_reg(reg);
                mem.set(self.SP - 2, v as u8)
            },
            4 => {
                self.SP -= 2;
                self.reset();
            },
            _ => ()
        }
    }

    fn pop(&mut self, reg: &Reg, mem: &mut Mem) {
        match self.current_cycle {
            2 => {
                self.store[0] = mem.get(self.SP);
            },
            3 => {
                self.store[1] = mem.get(self.SP + 1);
                self.SP += 2;

                match reg {
                    &Reg::AF => self.set_word_reg(reg, (self.store[1] as u16) << 8 | (self.store[0] & 0xf0) as u16),
                    _ => self.set_word_reg(reg, (self.store[1] as u16) << 8 | self.store[0] as u16)
                }
                self.reset();
            },
            _ => ()
        }
    }

    // EXTENDED

    fn extended(&mut self, mem: &mut Mem) {
        match self.current_cycle {
            1 => {
                self.store[0] = self.get_op(mem);
                self.execute_extended(mem);
            },
            _ => self.execute_extended(mem),
        }
    }

    fn execute_extended(&mut self, mem: &mut Mem) {
        match self.store[0] {
            0x00 => self.rlc(&HalfReg::B),
            0x01 => self.rlc(&HalfReg::C),
            0x02 => self.rlc(&HalfReg::D),
            0x03 => self.rlc(&HalfReg::E),
            0x04 => self.rlc(&HalfReg::H),
            0x05 => self.rlc(&HalfReg::L),
            0x06 => self.rlc_m(mem),
            0x07 => self.rlc(&HalfReg::A),
            0x08 => self.rrc(&HalfReg::B),
            0x09 => self.rrc(&HalfReg::C),
            0x0A => self.rrc(&HalfReg::D),
            0x0B => self.rrc(&HalfReg::E),
            0x0C => self.rrc(&HalfReg::H),
            0x0D => self.rrc(&HalfReg::L),
            0x0E => self.rrc_m(mem),
            0x0F => self.rrc(&HalfReg::A),

            0x10 => self.rl(&HalfReg::B),
            0x11 => self.rl(&HalfReg::C),
            0x12 => self.rl(&HalfReg::D),
            0x13 => self.rl(&HalfReg::E),
            0x14 => self.rl(&HalfReg::H),
            0x15 => self.rl(&HalfReg::L),
            0x16 => self.rl_m(mem),
            0x17 => self.rl(&HalfReg::A),
            0x18 => self.rr(&HalfReg::B),
            0x19 => self.rr(&HalfReg::C),
            0x1A => self.rr(&HalfReg::D),
            0x1B => self.rr(&HalfReg::E),
            0x1C => self.rr(&HalfReg::H),
            0x1D => self.rr(&HalfReg::L),
            0x1E => self.rr_m(mem),
            0x1F => self.rr(&HalfReg::A),

            0x20 => self.sla(&HalfReg::B),
            0x21 => self.sla(&HalfReg::C),
            0x22 => self.sla(&HalfReg::D),
            0x23 => self.sla(&HalfReg::E),
            0x24 => self.sla(&HalfReg::H),
            0x25 => self.sla(&HalfReg::L),
            0x26 => self.sla_m(mem),
            0x27 => self.sla(&HalfReg::A),
            0x28 => self.sra(&HalfReg::B),
            0x29 => self.sra(&HalfReg::C),
            0x2A => self.sra(&HalfReg::D),
            0x2B => self.sra(&HalfReg::E),
            0x2C => self.sra(&HalfReg::H),
            0x2D => self.sra(&HalfReg::L),
            0x2E => self.sra_m(mem),
            0x2F => self.sra(&HalfReg::A),

            0x30 => self.swap(&HalfReg::B),
            0x31 => self.swap(&HalfReg::C),
            0x32 => self.swap(&HalfReg::D),
            0x33 => self.swap(&HalfReg::E),
            0x34 => self.swap(&HalfReg::H),
            0x35 => self.swap(&HalfReg::L),
            0x36 => self.swap_m(mem),
            0x37 => self.swap(&HalfReg::A),
            0x38 => self.srl(&HalfReg::B),
            0x39 => self.srl(&HalfReg::C),
            0x3A => self.srl(&HalfReg::D),
            0x3B => self.srl(&HalfReg::E),
            0x3C => self.srl(&HalfReg::H),
            0x3D => self.srl(&HalfReg::L),
            0x3E => self.srl_m(mem),
            0x3F => self.srl(&HalfReg::A),

            0x40 => self.bit(&HalfReg::B,0),
            0x41 => self.bit(&HalfReg::C,0),
            0x42 => self.bit(&HalfReg::D,0),
            0x43 => self.bit(&HalfReg::E,0),
            0x44 => self.bit(&HalfReg::H,0),
            0x45 => self.bit(&HalfReg::L,0),
            0x46 => self.bit_m(mem,0),
            0x47 => self.bit(&HalfReg::A,0),
            0x48 => self.bit(&HalfReg::B,1),
            0x49 => self.bit(&HalfReg::C,1),
            0x4A => self.bit(&HalfReg::D,1),
            0x4B => self.bit(&HalfReg::E,1),
            0x4C => self.bit(&HalfReg::H,1),
            0x4D => self.bit(&HalfReg::L,1),
            0x4E => self.bit_m(mem,1),
            0x4F => self.bit(&HalfReg::A,1),

            0x50 => self.bit(&HalfReg::B,2),
            0x51 => self.bit(&HalfReg::C,2),
            0x52 => self.bit(&HalfReg::D,2),
            0x53 => self.bit(&HalfReg::E,2),
            0x54 => self.bit(&HalfReg::H,2),
            0x55 => self.bit(&HalfReg::L,2),
            0x56 => self.bit_m(mem,2),
            0x57 => self.bit(&HalfReg::A,2),
            0x58 => self.bit(&HalfReg::B,3),
            0x59 => self.bit(&HalfReg::C,3),
            0x5A => self.bit(&HalfReg::D,3),
            0x5B => self.bit(&HalfReg::E,3),
            0x5C => self.bit(&HalfReg::H,3),
            0x5D => self.bit(&HalfReg::L,3),
            0x5E => self.bit_m(mem,3),
            0x5F => self.bit(&HalfReg::A,3),

            0x60 => self.bit(&HalfReg::B,4),
            0x61 => self.bit(&HalfReg::C,4),
            0x62 => self.bit(&HalfReg::D,4),
            0x63 => self.bit(&HalfReg::E,4),
            0x64 => self.bit(&HalfReg::H,4),
            0x65 => self.bit(&HalfReg::L,4),
            0x66 => self.bit_m(mem,4),
            0x67 => self.bit(&HalfReg::A,4),
            0x68 => self.bit(&HalfReg::B,5),
            0x69 => self.bit(&HalfReg::C,5),
            0x6A => self.bit(&HalfReg::D,5),
            0x6B => self.bit(&HalfReg::E,5),
            0x6C => self.bit(&HalfReg::H,5),
            0x6D => self.bit(&HalfReg::L,5),
            0x6E => self.bit_m(mem,5),
            0x6F => self.bit(&HalfReg::A,5),

            0x70 => self.bit(&HalfReg::B,6),
            0x71 => self.bit(&HalfReg::C,6),
            0x72 => self.bit(&HalfReg::D,6),
            0x73 => self.bit(&HalfReg::E,6),
            0x74 => self.bit(&HalfReg::H,6),
            0x75 => self.bit(&HalfReg::L,6),
            0x76 => self.bit_m(mem,6),
            0x77 => self.bit(&HalfReg::A,6),
            0x78 => self.bit(&HalfReg::B,7),
            0x79 => self.bit(&HalfReg::C,7),
            0x7A => self.bit(&HalfReg::D,7),
            0x7B => self.bit(&HalfReg::E,7),
            0x7C => self.bit(&HalfReg::H,7),
            0x7D => self.bit(&HalfReg::L,7),
            0x7E => self.bit_m(mem,7),
            0x7F => self.bit(&HalfReg::A,7),

            0x80 => self.res(&HalfReg::B,0x1),
            0x81 => self.res(&HalfReg::C,0x1),
            0x82 => self.res(&HalfReg::D,0x1),
            0x83 => self.res(&HalfReg::E,0x1),
            0x84 => self.res(&HalfReg::H,0x1),
            0x85 => self.res(&HalfReg::L,0x1),
            0x86 => self.res_m(mem,0x1),
            0x87 => self.res(&HalfReg::A,0x1),
            0x88 => self.res(&HalfReg::B,0x2),
            0x89 => self.res(&HalfReg::C,0x2),
            0x8A => self.res(&HalfReg::D,0x2),
            0x8B => self.res(&HalfReg::E,0x2),
            0x8C => self.res(&HalfReg::H,0x2),
            0x8D => self.res(&HalfReg::L,0x2),
            0x8E => self.res_m(mem,0x2),
            0x8F => self.res(&HalfReg::A,0x2),

            0x90 => self.res(&HalfReg::B,0x4),
            0x91 => self.res(&HalfReg::C,0x4),
            0x92 => self.res(&HalfReg::D,0x4),
            0x93 => self.res(&HalfReg::E,0x4),
            0x94 => self.res(&HalfReg::H,0x4),
            0x95 => self.res(&HalfReg::L,0x4),
            0x96 => self.res_m(mem,0x4),
            0x97 => self.res(&HalfReg::A,0x4),
            0x98 => self.res(&HalfReg::B,0x8),
            0x99 => self.res(&HalfReg::C,0x8),
            0x9A => self.res(&HalfReg::D,0x8),
            0x9B => self.res(&HalfReg::E,0x8),
            0x9C => self.res(&HalfReg::H,0x8),
            0x9D => self.res(&HalfReg::L,0x8),
            0x9E => self.res_m(mem,0x8),
            0x9F => self.res(&HalfReg::A,0x8),

            0xA0 => self.res(&HalfReg::B,0x10),
            0xA1 => self.res(&HalfReg::C,0x10),
            0xA2 => self.res(&HalfReg::D,0x10),
            0xA3 => self.res(&HalfReg::E,0x10),
            0xA4 => self.res(&HalfReg::H,0x10),
            0xA5 => self.res(&HalfReg::L,0x10),
            0xA6 => self.res_m(mem,0x10),
            0xA7 => self.res(&HalfReg::A,0x10),
            0xA8 => self.res(&HalfReg::B,0x20),
            0xA9 => self.res(&HalfReg::C,0x20),
            0xAA => self.res(&HalfReg::D,0x20),
            0xAB => self.res(&HalfReg::E,0x20),
            0xAC => self.res(&HalfReg::H,0x20),
            0xAD => self.res(&HalfReg::L,0x20),
            0xAE => self.res_m(mem,0x20),
            0xAF => self.res(&HalfReg::A,0x20),

            0xB0 => self.res(&HalfReg::B,0x40),
            0xB1 => self.res(&HalfReg::C,0x40),
            0xB2 => self.res(&HalfReg::D,0x40),
            0xB3 => self.res(&HalfReg::E,0x40),
            0xB4 => self.res(&HalfReg::H,0x40),
            0xB5 => self.res(&HalfReg::L,0x40),
            0xB6 => self.res_m(mem,0x40),
            0xB7 => self.res(&HalfReg::A,0x40),
            0xB8 => self.res(&HalfReg::B,0x80),
            0xB9 => self.res(&HalfReg::C,0x80),
            0xBA => self.res(&HalfReg::D,0x80),
            0xBB => self.res(&HalfReg::E,0x80),
            0xBC => self.res(&HalfReg::H,0x80),
            0xBD => self.res(&HalfReg::L,0x80),
            0xBE => self.res_m(mem,0x80),
            0xBF => self.res(&HalfReg::A,0x80),

            0xC0 => self.set(&HalfReg::B,0x1),
            0xC1 => self.set(&HalfReg::C,0x1),
            0xC2 => self.set(&HalfReg::D,0x1),
            0xC3 => self.set(&HalfReg::E,0x1),
            0xC4 => self.set(&HalfReg::H,0x1),
            0xC5 => self.set(&HalfReg::L,0x1),
            0xC6 => self.set_m(mem,0x1),
            0xC7 => self.set(&HalfReg::A,0x1),
            0xC8 => self.set(&HalfReg::B,0x2),
            0xC9 => self.set(&HalfReg::C,0x2),
            0xCA => self.set(&HalfReg::D,0x2),
            0xCB => self.set(&HalfReg::E,0x2),
            0xCC => self.set(&HalfReg::H,0x2),
            0xCD => self.set(&HalfReg::L,0x2),
            0xCE => self.set_m(mem,0x2),
            0xCF => self.set(&HalfReg::A,0x2),

            0xD0 => self.set(&HalfReg::B,0x4),
            0xD1 => self.set(&HalfReg::C,0x4),
            0xD2 => self.set(&HalfReg::D,0x4),
            0xD3 => self.set(&HalfReg::E,0x4),
            0xD4 => self.set(&HalfReg::H,0x4),
            0xD5 => self.set(&HalfReg::L,0x4),
            0xD6 => self.set_m(mem,0x4),
            0xD7 => self.set(&HalfReg::A,0x4),
            0xD8 => self.set(&HalfReg::B,0x8),
            0xD9 => self.set(&HalfReg::C,0x8),
            0xDA => self.set(&HalfReg::D,0x8),
            0xDB => self.set(&HalfReg::E,0x8),
            0xDC => self.set(&HalfReg::H,0x8),
            0xDD => self.set(&HalfReg::L,0x8),
            0xDE => self.set_m(mem,0x8),
            0xDF => self.set(&HalfReg::A,0x8),

            0xE0 => self.set(&HalfReg::B,0x10),
            0xE1 => self.set(&HalfReg::C,0x10),
            0xE2 => self.set(&HalfReg::D,0x10),
            0xE3 => self.set(&HalfReg::E,0x10),
            0xE4 => self.set(&HalfReg::H,0x10),
            0xE5 => self.set(&HalfReg::L,0x10),
            0xE6 => self.set_m(mem,0x10),
            0xE7 => self.set(&HalfReg::A,0x10),
            0xE8 => self.set(&HalfReg::B,0x20),
            0xE9 => self.set(&HalfReg::C,0x20),
            0xEA => self.set(&HalfReg::D,0x20),
            0xEB => self.set(&HalfReg::E,0x20),
            0xEC => self.set(&HalfReg::H,0x20),
            0xED => self.set(&HalfReg::L,0x20),
            0xEE => self.set_m(mem,0x20),
            0xEF => self.set(&HalfReg::A,0x20),

            0xF0 => self.set(&HalfReg::B,0x40),
            0xF1 => self.set(&HalfReg::C,0x40),
            0xF2 => self.set(&HalfReg::D,0x40),
            0xF3 => self.set(&HalfReg::E,0x40),
            0xF4 => self.set(&HalfReg::H,0x40),
            0xF5 => self.set(&HalfReg::L,0x40),
            0xF6 => self.set_m(mem,0x40),
            0xF7 => self.set(&HalfReg::A,0x40),
            0xF8 => self.set(&HalfReg::B,0x80),
            0xF9 => self.set(&HalfReg::C,0x80),
            0xFA => self.set(&HalfReg::D,0x80),
            0xFB => self.set(&HalfReg::E,0x80),
            0xFC => self.set(&HalfReg::H,0x80),
            0xFD => self.set(&HalfReg::L,0x80),
            0xFE => self.set_m(mem,0x80),
            0xFF => self.set(&HalfReg::A,0x80),
        }
    }

    fn rlc(&mut self, reg: &HalfReg) {
        let value = self.get_byte_reg(reg);
        let b7 = value >> 7 & 1 == 1;
        let value = value.rotate_left(1);

        self.set_flag(Flag::C, b7);
        self.set_flag(Flag::Z, value == 0);
        self.set_flag(Flag::H, false);
        self.set_flag(Flag::N, false);

        self.set_byte_reg(reg, value);
        self.reset();
    }

    fn rlc_m(&mut self, mem: &mut Mem) {
        match self.current_cycle {
            3 => {
                self.store[1] = mem.get(self.HL);
            },
            4 => {
                let value = self.store[1];
                let b7 = value >> 7 & 1 == 1;
                let value = value.rotate_left(1);

                self.set_flag(Flag::C, b7);
                self.set_flag(Flag::Z, value == 0);
                self.set_flag(Flag::H, false);
                self.set_flag(Flag::N, false);
                mem.set(self.HL, value);

                self.reset();
            },
            _ => ()
        }
    }

    fn rrc(&mut self, reg: &HalfReg) {
        let value = self.get_byte_reg(reg);
        let b0 = value & 1 == 1;
        let value = value.rotate_right(1);

        self.set_flag(Flag::C, b0);
        self.set_flag(Flag::Z, value == 0);
        self.set_flag(Flag::H, false);
        self.set_flag(Flag::N, false);

        self.set_byte_reg(reg, value);
        self.reset();
    }

    fn rrc_m(&mut self, mem: &mut Mem) {
        match self.current_cycle {
            3 => {
                self.store[1] = mem.get(self.HL);
            },
            4 => {
                let value = self.store[1];
                let b0 = value & 1 == 1;
                let value = value.rotate_right(1);

                self.set_flag(Flag::C, b0);
                self.set_flag(Flag::Z, value == 0);
                self.set_flag(Flag::H, false);
                self.set_flag(Flag::N, false);
                mem.set(self.HL, value);

                self.reset();
            },
            _ => ()
        }
    }

    fn rl(&mut self, reg: &HalfReg) {
        let value = self.get_byte_reg(reg);
        let b7 = (value >> 7) & 1 == 1;
        let c = if self.get_flag(Flag::C) {1} else {0};
        let value = (value << 1) | c;

        self.set_flag(Flag::C, b7);
        self.set_flag(Flag::Z, value == 0);
        self.set_flag(Flag::H, false);
        self.set_flag(Flag::N, false);

        self.set_byte_reg(reg, value);
        self.reset();
    }

    fn rl_m(&mut self, mem: &mut Mem) {
        match self.current_cycle {
            3 => {
                self.store[1] = mem.get(self.HL);
            },
            4 => {
                let value = self.store[1];
                let b7 = value >> 7 & 1 == 1;
                let c = if self.get_flag(Flag::C) {1} else {0};
                let value = (value << 1) | c;
        
                self.set_flag(Flag::C, b7);
                self.set_flag(Flag::Z, value == 0);
                self.set_flag(Flag::H, false);
                self.set_flag(Flag::N, false);
        
                mem.set(self.HL, value);
                self.reset();
            },
            _ => (),
        }
    }

    fn rr(&mut self, reg: &HalfReg) {
        let value = self.get_byte_reg(reg);
        let b0 = value & 1 == 1;
        let c = if self.get_flag(Flag::C) {0x80} else {0};
        let value = (value >> 1) | c;

        self.set_flag(Flag::C, b0);
        self.set_flag(Flag::Z, value == 0);
        self.set_flag(Flag::H, false);
        self.set_flag(Flag::N, false);

        self.set_byte_reg(reg, value);
        self.reset();
    }

    fn rr_m(&mut self, mem: &mut Mem) {
        match self.current_cycle {
            3 => {
                self.store[1] = mem.get(self.HL);
            },
            4 => {
                let value = self.store[1];
                let b0 = value & 1 == 1;
                let c = if self.get_flag(Flag::C) {0x80} else {0};
                let value = (value >> 1) | c;
        
                self.set_flag(Flag::C, b0);
                self.set_flag(Flag::Z, value == 0);
                self.set_flag(Flag::H, false);
                self.set_flag(Flag::N, false);
        
                mem.set(self.HL, value);
                self.reset();
            },
            _ => (),
        }
    }

    fn sla(&mut self, reg: &HalfReg) {
        let value = self.get_byte_reg(reg);
        let b7 = value >> 7 & 1 == 1;
        let value = value << 1;

        self.set_flag(Flag::C, b7);
        self.set_flag(Flag::Z, value == 0);
        self.set_flag(Flag::H, false);
        self.set_flag(Flag::N, false);

        self.set_byte_reg(reg, value);
        self.reset();
    }

    fn sla_m(&mut self, mem: &mut Mem) {
        match self.current_cycle {
            3 => {
                self.store[1] = mem.get(self.HL);
            },
            4 => {
                let value = self.store[1];
                let b7 = value >> 7 & 1 == 1;
                let value = value << 1;
        
                self.set_flag(Flag::C, b7);
                self.set_flag(Flag::Z, value == 0);
                self.set_flag(Flag::H, false);
                self.set_flag(Flag::N, false);
        
                mem.set(self.HL, value);
                self.reset();
            },
            _ => (),
        }
    }

    fn sra(&mut self, reg: &HalfReg) {
        let value = self.get_byte_reg(reg);
        let b0 = value & 0x1 == 1;
        let b7 = value & 0x80;
        let value = (value >> 1) | b7;

        self.set_flag(Flag::C, b0);
        self.set_flag(Flag::Z, value == 0);
        self.set_flag(Flag::H, false);
        self.set_flag(Flag::N, false);

        self.set_byte_reg(reg, value);
        self.reset();
    }

    fn sra_m(&mut self, mem: &mut Mem) {
        match self.current_cycle {
            3 => {
                self.store[1] = mem.get(self.HL);
            },
            4 => {
                let value = self.store[1];
                let b0 = value & 1 == 1;
                let b7 = value & 0x80;
                let value = (value >> 1) | b7;
        
                self.set_flag(Flag::C, b0);
                self.set_flag(Flag::Z, value == 0);
                self.set_flag(Flag::H, false);
                self.set_flag(Flag::N, false);
        
                mem.set(self.HL, value);
                self.reset();
            },
            _ => (),
        }
    }

    fn swap(&mut self, reg: &HalfReg) {
        let value = self.get_byte_reg(reg);
        let value = (value >> 4) | (value << 4);

        self.set_byte_reg(reg, value);

        self.set_flag(Flag::C, false);
        self.set_flag(Flag::H, false);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::Z, value == 0);

        self.reset();
    }

    fn swap_m(&mut self, mem: &mut Mem) {
        match self.current_cycle {
            3 => {
                self.store[1] = mem.get(self.HL);
            },
            4 => {
                let value = self.store[1];
                let value = (value >> 4) | (value << 4);

                mem.set(self.HL, value);

                self.set_flag(Flag::C, false);
                self.set_flag(Flag::H, false);
                self.set_flag(Flag::N, false);
                self.set_flag(Flag::Z, value == 0);

                self.reset();
            },
            _ => ()
        }
    }

    fn srl(&mut self, reg: &HalfReg) {
        let value = self.get_byte_reg(reg);
        let b0 = value & 0x1;
        let value = value >> 1;

        self.set_flag(Flag::C, b0 == 0x1);
        self.set_flag(Flag::Z, value == 0);
        self.set_flag(Flag::H, false);
        self.set_flag(Flag::N, false);

        self.set_byte_reg(reg, value);
        self.reset();
    }

    fn srl_m(&mut self, mem: &mut Mem) {
        match self.current_cycle {
            3 => {
                self.store[1] = mem.get(self.HL);
            },
            4 => {
                let value = self.store[1];
                let b0 = value & 0x1;
                let value = value >> 1;
        
                self.set_flag(Flag::C, b0 == 0x1);
                self.set_flag(Flag::Z, value == 0);
                self.set_flag(Flag::H, false);
                self.set_flag(Flag::N, false);
        
                mem.set(self.HL, value);
                self.reset();
            },
            _ => (),
        }
    }

    /*
        Copy the complement of the contents of bit 0 in register B to the 
        Z flag of the program status word (PSW).
    */
    fn bit(&mut self, reg: &HalfReg, bit_num: u8) {
        let value = self.get_byte_reg(reg);
        let bit = (value >> bit_num) & 1 == 1;

        self.set_flag(Flag::Z, !bit);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::H, true);

        self.reset();
    }

    fn bit_m(&mut self, mem: &mut Mem, bit_num: u8) {
        match self.current_cycle {
            3 => {
                self.store[1] = mem.get(self.HL);
            },
            4 => {
                let value = self.store[1];
                let bit = (value >> bit_num) & 1 == 1;

                self.set_flag(Flag::Z, !bit);
                self.set_flag(Flag::N, false);
                self.set_flag(Flag::H, true);

                self.reset();
            },
            _ => ()
        }
    }

    fn res(&mut self, reg: &HalfReg, bit_mask: u8) {
        let mut value = self.get_byte_reg(reg);
        value &= !bit_mask;

        self.set_byte_reg(reg, value);
        self.reset();
    }

    fn res_m(&mut self, mem: &mut Mem, bit_mask: u8) {
        match self.current_cycle {
            3 => {
                self.store[1] = mem.get(self.HL);
            },
            4 => {
                let mut value = self.store[1];
                value &= !bit_mask;
        
                mem.set(self.HL, value);
                self.reset();
            },
            _ => ()
        }
    }

    fn set(&mut self, reg: &HalfReg, bit_mask: u8) {
        let mut value = self.get_byte_reg(reg);
        value |= bit_mask;

        self.set_byte_reg(reg, value);
        self.reset();
    }

    fn set_m(&mut self, mem: &mut Mem, bit_mask: u8) {
        match self.current_cycle {
            3 => {
                self.store[1] = mem.get(self.HL);
            },
            4 => {
                let mut value = self.store[1];
                value |= bit_mask;
        
                mem.set(self.HL, value);
                self.reset();
            },
            _ => ()
        }
    }

}