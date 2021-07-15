
use crate::io::{Button, Joypad, P1_ADDR, SB_ADDR, SC_ADDR, Serial, Timer, DIV_ADDR, TIMA_ADDR, TMA_ADDR, TAC_ADDR};
use wasm_bindgen::prelude::*;

extern crate web_sys;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

#[wasm_bindgen]

pub struct Mem {
    mem: Vec<u8>,
    // INTERUPT
    iflag: u8,
    ienable: u8,
    // PPU
    LCDControl: u8,
    LCDStatus: u8,
    scrolly: u8,
    scrollx: u8,
    ly: u8,
    lyc: u8,
    dma: u8,
    bgp: u8,
    obp0: u8,
    obp1: u8,
    wy: u8,
    wx: u8,
    // DMA
    pub transfering: bool,
    transfer_count: u16,
    // IO
    serial: Serial,
    joypad: Joypad,
    timer: Timer,

    pub ppu_access: bool,
    rom_lock: bool

}

impl Mem {
    pub fn new() -> Self {
        Self {
            mem: vec![0; 0xFFFF],
            iflag: 0,
            ienable: 0x1F,

            LCDControl: 0,
            LCDStatus: 0,
            scrolly: 0,
            scrollx: 0,
            ly: 0,
            lyc: 0,
            dma: 0,
            bgp: 0,
            obp0: 0,
            obp1: 0,
            wy: 0,
            wx: 0,

            transfering: false,
            transfer_count: 0,

            serial: Serial::new(),
            joypad: Joypad::new(),
            timer: Timer::new(),

            ppu_access: false,
            rom_lock: false
        }
    }

    pub fn reset(&mut self) {
        self.mem = vec![0; 0xFFFF];
    }

    pub fn get_mem(&mut self) -> &mut Vec<u8> {
        &mut self.mem
    }

    pub fn get_serial(&self) -> &Serial {
        &self.serial
    }

    pub fn lock_rom(&mut self, lock: bool) {
        self.rom_lock = lock;
    }

    pub fn get(&self, addr: u16) -> u8 {
        if self.transfering && !self.ppu_access { // can only access high ram
            return match addr {
                0xFF80..=0xFFFE => self.mem[addr as usize],
                0xFF46 => self.dma,
                _ => 0xFF
            }
        }
        match addr {
            P1_ADDR => self.joypad.read(),
            0xFFFF => self.ienable,
            0xFF0F => self.iflag,

            0xFF40 => self.LCDControl,
            0xFF41 => self.LCDStatus,
            0xFF42 => self.scrolly,
            0xFF43 => self.scrollx,
            0xFF44 => self.ly,
            0xFF45 => self.lyc,
            0xFF46 => self.dma,
            0xFF47 => self.bgp,
            0xFF48 => self.obp0,
            0xFF49 => self.obp1,
            0xFF4A => self.wy,
            0xFF4B => self.wx,
            
            DIV_ADDR | TIMA_ADDR | TMA_ADDR | TAC_ADDR => self.timer.read(addr),
            SB_ADDR | SC_ADDR => self.serial.read(addr),
            0xE000 ..= 0xFDFF => self.mem[(addr - 0x200) as usize],
            0x0000 ..= 0xFFFE => self.mem[addr as usize],
            _ => 0xFF
        }
    }

    pub fn set_ly(&mut self, value: u8) {
        self.ly = value;
        if value == 255 {
            println!("???");
        }
        if self.ly == self.lyc {
            self.set_lcd_stat(2, 1);
            self.iflag |= 0x2;
        } else {
            self.set_lcd_stat(2, 0);
        }
    }

    pub fn set(&mut self, addr: u16, value: u8) {

        match addr {
            P1_ADDR => self.joypad.write(value),
            0xFF0F => self.iflag = value,
            
            0xFF40 => self.LCDControl = value,
            0xFF41 => self.LCDStatus = value,
            0xFF42 => self.scrolly = value,
            0xFF43 => self.scrollx = value,
            0xFF45 => {
                self.lyc = value;
                if self.ly == self.lyc {
                    self.set_lcd_stat(2, 1);
                } else {
                    self.set_lcd_stat(2, 0);
                }
            },
            0xFF46 => {
                self.dma = value;
                unsafe { log!("DMA < {:X}", value); }
                if value <= 0xDF { self.dma_transfer() }
            },
            0xFF47 => self.bgp = value,
            0xFF48 => self.obp0 = value,
            0xFF49 => self.obp1 = value,
            0xFF4A => self.wy = value,
            0xFF4B => self.wx = value,

            DIV_ADDR | TIMA_ADDR | TMA_ADDR | TAC_ADDR => self.timer.write(addr, value),
            SB_ADDR | SC_ADDR => self.serial.write(addr, value),
            0xFFFF => self.ienable = value,
            0x0..=0x7FFF => if self.rom_lock {} else {self.mem[addr as usize] = value},
            _ => self.mem[addr as usize] = value,
        };
    }

    pub fn dma_transfer(&mut self) {
        if self.transfer_count == 160 {
            self.transfer_count = 0;
            self.transfering = false;
            return;
        }

        let start_addr = 0xFE00;
        let cur_addr = start_addr + self.transfer_count;

        let from_addr = (self.dma as u16) << 8;
        let cur_from = from_addr + self.transfer_count;
        self.ppu_access = true;
        let value = self.get(cur_from);
        self.ppu_access = false;
        self.set(
            cur_addr,
            value
        );
        if cur_addr == 0xFE0A {
            unsafe { log!("transfer {:X} > {:X} {:X} | ", cur_from, cur_addr, value); }
        }
        self.transfer_count += 1;
        self.transfering = true;
    }

    pub fn set_lcd_stat(&mut self, bit: u8, v: u8) {
        if bit == 0 || bit == 1 {
            self.LCDStatus = self.LCDStatus & 0b11111100;
            self.LCDStatus |= v;
        } else {
            self.LCDStatus = self.LCDStatus & !(1 << bit);
            self.LCDStatus |= v << bit;
        }
    }

    pub fn button(&mut self, b: Button, v: bool) {
        self.joypad.set(b, v);
    }

    pub fn get_action_buttons(&self) -> u8 {
        self.joypad.get_action()
    }

    pub fn get_arrow_buttons(&self) -> u8 {
        self.joypad.get_arrow()
    }

    pub fn tick(&mut self) {
        let overflowed = self.timer.tick();
        if overflowed {
            self.mem[0xFF0F as usize] = self.mem[0xFF0F as usize] & 4;
        }
    }
}