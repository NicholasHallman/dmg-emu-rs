#[cfg(test)]

#[macro_use]
extern crate cascade;

extern crate test_case;
mod cpu_assert;

mod cpu_tests {
    use dmg_emu::{Emu, cpu::{self, HalfReg, Reg}}; 
    use crate::cpu_assert::{EmuTester, EmuTestHelpers};
    use test_case::test_case;

    fn before(op_codes: [u8; 3]) -> Emu {
        let mut emu = Emu::new(false);
        let mut mem: [u8; 50] = [0; 50];
        mem[0] = op_codes[0];
        mem[1] = op_codes[1];
        mem[2] = op_codes[2];
        emu.write_mem(&mem);
        return emu
    }

    #[test]
    fn noop() {
        let mut emu = before([0,0,0]);
        emu.tick_till_done();
        cascade! {
            emu.assert();
            ..reg(Reg::PC);..equals(0x1u16);
        };
    }

    #[test_case(Reg::BC ;  "Register BC stored 0x3020")]
    #[test_case(Reg::DE ;  "Register DE stored 0x3020")]
    #[test_case(Reg::HL ;  "Register HL stored 0x3020")]
    #[test_case(Reg::SP ;  "Register SP stored 0x3020")]
    fn ld_rr_d16(reg: Reg) {
        let op;
        match reg {
            Reg::BC => op = 0x01,
            Reg::DE => op = 0x11,
            Reg::HL => op = 0x21,
            Reg::SP => op = 0x31,
            _ => panic!("Can't store at AF or PC")
        }
        let mut emu = before([op, 0x20, 0x30]);
        emu.tick_till_done();
        cascade! {
            emu.assert();
            ..reg(reg);..equals(0x3020u16);
        };
    }

    #[test_case(Reg::BC ;  "Address in BC stored A")]
    #[test_case(Reg::DE ;  "Address in DE stored A")]
    fn ld_ar_a(reg: Reg) {
        let op;
        match reg {
            Reg::BC => op = 0x02,
            Reg::DE => op = 0x12,
            _ => panic!("Can't store here")
        }
        let mut emu = before([op, 0, 0]);
        emu.cpu.set_byte_reg(&HalfReg::A, 0x20);
        emu.cpu.set_word_reg(&reg, 0x3020);
        emu.tick_till_done();
        cascade! {
            emu.assert();
            ..mem(0x3020);..equals(0x20u8);
        };
    }

    #[test]
    fn cp_d8_should_be_equal() {
        let mut emu = before([0xFE, 0x90, 0x00]);

        emu.cpu.set_byte_reg(&cpu::HalfReg::A, 0x90);
        emu.tick_till_done();

        cascade! {
            emu.assert();
            ..flag(cpu::Flag::N);..equals(true);
            ..flag(cpu::Flag::Z);..equals(true);
            ..flag(cpu::Flag::C);..equals(false);
        };
    }
    #[test]
    fn cp_d8_should_not_be_equal() {
        let mut emu = before([0xFE, 0x90, 0x00]);

        emu.cpu.set_byte_reg(&cpu::HalfReg::A, 0x00);
        emu.tick_till_done();

        cascade! {
            emu.assert();
            ..flag(cpu::Flag::N);..equals(true);
            ..flag(cpu::Flag::Z);..equals(false);
            ..flag(cpu::Flag::C);..equals(true);
        };
    }

}