
pub mod mem;
pub mod cpu;
pub mod ppu;
pub mod io;

use std::{fs};
use wasm_bindgen::prelude::*;

use cpu::{Cpu, DebugCpu};
use io::Button;
use mem::{Mem};
use ppu::{Ppu};

#[wasm_bindgen]
pub struct Emu {
    cpu: Cpu,
    mem: Mem,
    ppu: Ppu,
}

#[wasm_bindgen]
impl Emu {
    pub fn new() -> Self { 
        Self {
            cpu: Cpu::new(),
            mem: Mem::new(),
            ppu: Ppu::new(),
        }
    }

    pub fn init(&mut self) {
        self.cpu.PC = 0x100;
        self.cpu.SP = 0xFFFE;
        self.cpu.AF = 0x1180;
    }

    pub fn tick(&mut self) {
        self.cycle();
        while self.cpu.get_cycle() != 1 {
            self.cycle();
        }
    }

    pub fn tick_till_frame_done(&mut self) -> u16 {
        self.cycle();
        while !self.ppu.ready {
            self.cycle();
        }
        self.cpu.PC
    }

    pub fn get_buffer(&mut self) -> Vec<u8> {
        self.ppu.get_buffer().clone().to_vec()
    }

    pub fn load_rom_data(&mut self, rom: Vec<u8>) {
        let mut i = 0;
        for inst in rom {
            self.mem.set(i, inst);
            i += 1;
        }
    }

    pub fn get_cpu_state(&self) -> DebugCpu {
        self.cpu.get_state()
    }

    pub fn get_mem_state(&self) -> Vec<u8> {
        let mut clone: Vec<u8> = vec![0; 0xFFFF];
        for i in 0..0xFFFF{
            clone[i as usize] = self.mem.get(i);
        }
        clone
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

impl Emu {
    
    pub fn cpu(&mut self) -> &mut Cpu {
        &mut self.cpu
    }

    pub fn mem(&mut self) -> &mut Mem {
        &mut self.mem
    }

    pub fn ppu(&mut self) -> &mut Ppu {
        &mut self.ppu
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
    }

    pub fn load_nintendo_logo(&mut self) {
        let logo = &fs::read_to_string("./resources/nintendo_logo.txt").expect("File Not Found");
        let logo = logo.split(',');
        let mut curr_addr:usize = 0x104;
        for byte in logo {
            self.mem.get_mem()[curr_addr] = u8::from_str_radix(byte.trim_start_matches("0x"), 16).unwrap();
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
    }

    pub fn get_serial(&self) -> String {
        self.mem.get_serial().get_buffer().clone()
    }

    pub fn cycle(&mut self) {
        if self.mem.transfering {
            self.mem.dma_transfer();
        }
        self.cpu.tick(&mut self.mem);
        self.ppu.tick(&mut self.mem);
        self.mem.tick();
    }
   
    pub fn press_button<B>(&mut self, button: B, value: bool) where B: Into<Button> {
        self.mem.button(button.into(), value);
    }
}