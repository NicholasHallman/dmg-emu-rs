#[cfg(test)]

extern crate test_case;
mod cpu_assert;

mod blargg_cpu_test {

    use dmg_emu::{Emu}; 
    use test_case::test_case;
    use std::env;

    fn init(name: &str) -> Emu {
        let mut emu = Emu::new();
        emu.load_rom(name);
        emu.cpu().PC = 0x0100;
        emu.cpu().SP = 0xFFFE;
        emu.cpu().AF = 0x1180;
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

    #[test_case("basic.gb";  "dma_basic.gb")]
    fn dma_tests(name: &str) {
        let mut rom = name.to_string();
        if let Ok(path) = env::var("TEST_ROM_PATH") {
            rom = path + rom.as_str();
        }

        let mut emu = init(rom.as_str());
        run_rom(&mut emu);  
    }

}

