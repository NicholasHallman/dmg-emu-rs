
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
        if debugging {
            debug.clear_term().expect("Failed to clear terminal");
            debug.enable_trace();
        }
        

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
        let logo = &fs::read_to_string("./resources/nintendo_logo.txt").expect("File Not Found");
        let logo = logo.split(',');
        let mut curr_addr:usize = 0x104;
        for byte in logo {
            self.mem.mem[curr_addr] = u8::from_str_radix(byte.trim_start_matches("0x"), 16).unwrap();
            curr_addr += 1;
        }
    }

    pub fn load_rom<S: Into<String>>(&mut self, name: S) {
        let path = format!("./resources/{}", &name.into());
        let boot_rom = &fs::read(path).expect("File Not Found");
        let mut i: u16 = 0;
        for instruction in boot_rom {
            self.mem.set(i, *instruction);
            i += 1;
        }

        self.debug.load_assembly(&self.mem, false);
    }

    pub fn get_serial(&self) -> String {
        self.mem.serial.string_buffer.clone()
    }

    pub fn check_for_break(&mut self) {
        if !self.debugging {return}

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
            if !self.debug.is_blocked {
                if self.mem.transfering {
                    self.mem.dma_transfer();
                }
                self.cpu.tick(&mut self.mem);
                self.ppu.tick(&mut self.mem);
                self.mem.tick();
            }
        }
    }

    pub fn press_button<B>(&mut self, button: B, value: bool) where B: Into<Button> {
        self.mem.button(button.into(), value);
    }

    pub fn button_states(&mut self, buttons: u8) {
        let up = buttons & 1 == 1;
        let down = buttons >> 1 & 1 == 1; 
        let left = buttons >> 2 & 1 == 1; 
        let right = buttons >> 3 & 1 == 1; 
        let a = buttons >> 4 & 1 == 1; 
        let b = buttons >> 5 & 1 == 1; 
        let start = buttons >> 6 & 1 == 1; 
        let select = buttons >> 7 & 1 == 1;
        
        self.mem.button(Button::A, !a);
        self.mem.button(Button::B, !b);
        self.mem.button(Button::Start, !start);
        self.mem.button(Button::Select, !select);
        self.mem.button(Button::Up, !up);
        self.mem.button(Button::Down, !down);
        self.mem.button(Button::Left, !left);
        self.mem.button(Button::Right, !right);
    }
}