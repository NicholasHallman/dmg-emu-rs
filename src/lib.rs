
pub mod mem;
pub mod cpu;
pub mod ppu;
pub mod debug;
pub mod screen;
pub mod io;

use std::{fs, time::Instant};

use cpu::{Cpu};
use io::Button;
use mem::{Mem};
use ppu::{Ppu};
use debug::{DebugEmu};

pub struct Emu {
    pub cpu: Cpu,
    pub mem: Mem,
    pub ppu: Ppu,

    pub debugging: bool,
    pub debug: DebugEmu,

}

impl Emu {
    pub fn new(debugging: bool) -> Self {

        let debug = DebugEmu::new();
        debug.clear_term().expect("Failed to clear terminal");
        debug.enable_trace();

        Self {
            cpu: Cpu::new(),
            mem: Mem::new(),
            ppu: Ppu::new(),
            debugging,
            debug,
        }
    }

    pub fn write_mem(&mut self, values: &[u8; 50]) {
        let mut i: u16 = 0;
        for b in values {
            self.mem.set(i, *b);
            i += 1;
        }
    }

    pub fn load_boot_rom(&mut self) {
        let boot_rom = &fs::read("./resources/DMG_ROM.bin").expect("File Not Found");
        let mut i: u16 = 0;
        for instruction in boot_rom {
            self.mem.set(i, *instruction);
            i += 1;
        }
        self.load_nintendo_logo();

        self.debug.load_assembly(&self.mem, false);
    }

    pub fn load_nintendo_logo(&mut self) {
        let logo: [u8; 48] = [0xce, 0xed, 0x66, 0x66, 0xcc, 0x0d, 0x00, 0x0b, 0x03, 0x73, 0x00, 0x83, 0x00, 0x0c, 0x00, 0x0d, 0x00, 0x08, 0x11, 0x1f, 0x88, 0x89, 0x00, 0x0e, 0xdc, 0xcc, 0x6e, 0xe6, 0xdd, 0xdd, 0xd9, 0x99, 0xbb, 0xbb, 0x67, 0x63, 0x6e, 0x0e, 0xec, 0xcc, 0xdd, 0xdc, 0x99, 0x9f, 0xbb, 0xb9, 0x33, 0x3e];
        
        for i in 0x104..0x133 {
            self.mem.mem[i as usize] = logo[(i - 0x104) as usize];
        }
    }

    pub fn load_rom<S: Into<String>>(&mut self, name: S) {
        let boot_rom = &fs::read(format!("./resources/{}", name.into())).expect("File Not Found");
        let mut i: u16 = 0;
        for instruction in boot_rom {
            self.mem.set(i, *instruction);
            i += 1;
        }

        self.debug.load_assembly(&self.mem, false);
    }

    pub fn check_for_break(&mut self) {
        if self.debug.is_blocked || (self.cpu.current_cycle == 1 && self.debug.is_breakpoint(self.cpu.PC))  {
            if !self.debug.is_blocked {
                self.debug.print_registers(self).expect("Register Print Error");
                self.debug.print_ins(&self.cpu, &self.mem).expect("Ins Print Error");
                self.debug.print_mem(&self.mem).expect("Mem Debug Error");
                self.debug.print_stat(&self.mem).expect("Stat Debug Error");
            }
            self.debug.block(self.cpu.PC, &self.mem, &self.cpu).expect("Problem with block");
        }
    }

    pub fn tick(&mut self) {
        if self.ppu.wait_for_frame {
            let elapsed = Instant::now() - self.ppu.start_time;
            if elapsed.as_millis() >= 16 {
                self.ppu.wait_for_frame = false;
                self.ppu.start_time = Instant::now();
            }
        } else {
            self.check_for_break();
            //self.debug.trace_to_file(&self.mem, &self.cpu);
            if !self.debug.is_blocked {
                self.cpu.tick(&mut self.mem);
                self.ppu.tick(&mut self.mem);
            }
        }
    }

    pub fn press_button<B>(&mut self, button: B, value: bool) where B: Into<Button> {
        self.mem.button(button.into(), value);
    }
}