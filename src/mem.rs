
use crate::io::{Button, Joypad, P1_ADDR, SB_ADDR, SC_ADDR, Serial};

pub struct Mem {
    pub mem: [u8; 0xFFFF],
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

    serial: Serial,
    joypad: Joypad

}

impl Mem {
    pub fn new() -> Self {
        Self {
            mem: [0; 0xFFFF],
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

            serial: Serial::new(),
            joypad: Joypad::new(),
        }
    }

    pub fn get(&self, addr: u16) -> u8 {
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

            SB_ADDR | SC_ADDR => self.serial.read(addr),
            0xE000 ..= 0xFDFF => self.mem[(addr - 0x200) as usize],
            0x0000 ..= 0xFFFE => self.mem[addr as usize],
            _ => 0xFF
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
            0xFF44 => {
                self.ly = value;
                if self.ly == self.lyc {
                    self.set_lcd_stat(2, 1);
                    self.iflag |= 0x2;
                } else {
                    self.set_lcd_stat(2, 0);
                }
            },
            0xFF45 => {
                self.lyc = value;
                if self.ly == self.lyc {
                    self.set_lcd_stat(2, 1);
                } else {
                    self.set_lcd_stat(2, 0);
                }
            },
            0xFF46 => self.dma = value,
            0xFF47 => self.bgp = value,
            0xFF48 => self.obp0 = value,
            0xFF49 => self.obp1 = value,
            0xFF4A => self.wy = value,
            0xFF4B => self.wx = value,
            
            SB_ADDR | SC_ADDR => self.serial.write(addr, value),
            0xFFFF => self.ienable = value,
            _ => self.mem[addr as usize] = value,
        };
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
}