#[cfg(test)]

#[macro_use]
extern crate cascade;

extern crate test_case;
mod cpu_assert;

mod blargg_cpu_test {

    use dmg_emu::{Emu, cpu::{self, HalfReg, Reg}}; 
    use crate::cpu_assert::{EmuTester, EmuTestHelpers};
    use test_case::test_case;
    use std::env;

    fn init(name: &str) -> Emu {
        let mut emu = Emu::new(false);
        emu.load_rom(name);
        emu.cpu.PC = 0x0100;
        emu.cpu.SP = 0xFFFE;
        emu.cpu.AF = 0x1180;
        emu
    }

    fn run_rom(emu: &mut Emu) {
        let mut passed = false;
        let mut failed = false;
        while !passed && !failed {
            emu.tick();
            passed = emu.get_serial().contains("Passed");
            failed = emu.get_serial().contains("Failed");
        }

        assert_eq!(passed, true);
    }

    #[test_case("01-special.gb"             ;  "01-special.gb")]
    #[test_case("02-interrupts.gb"          ;  "02-interrupts.gb")]
    #[test_case("03-op sp,hl.gb"            ;  "03-op sp,hl.gb")]
    #[test_case("04-op r,imm.gb"            ;  "04-op r,imm.gb")]
    #[test_case("05-op rp.gb"               ;  "05-op rp.gb")]
    #[test_case("06-ld r,r.gb"              ;  "06-ld r,r.gb")]
    #[test_case("07-jr,jp,call,ret,rst.gb"  ;  "07-jr,jp,call,ret,rst.gb")]
    #[test_case("08-misc instrs.gb"         ;  "08-misc instrs.gb")]
    #[test_case("09-op r,r.gb"              ;  "09-op r,r.gb")]
    #[test_case("10-bit ops.gb"             ;  "10-bit ops.gb")]
    #[test_case("11-op a,(hl).gb"           ;  "11-op a,(hl).gb")]
    fn blargg_cpu_test(name: &str) {
        let mut rom = name.to_string();
        if let Ok(path) = env::var("TEST_ROM_PATH") {
            rom = path + rom.as_str();
        }

        let mut emu = init(rom.as_str());
        run_rom(&mut emu);  
    }

}

