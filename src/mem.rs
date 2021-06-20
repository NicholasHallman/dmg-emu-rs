
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
    // DMA
    pub transfering: bool,
    transfer_count: u16,
    // IO
    pub serial: Serial,
    joypad: Joypad,

    pub ppu_access: bool,

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

            transfering: false,
            transfer_count: 0,

            serial: Serial::new(),
            joypad: Joypad::new(),

            ppu_access: false
        }
    }

    pub fn get(&self, addr: u16) -> u8 {
        if self.transfering && !self.ppu_access { // can only access high ram
            return match addr {
                0xFF80..=0xFFFE => self.mem[addr as usize],
                _ => 0xFF
            }
        }
        match addr {
            P1_ADDR => 0xFF,
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
                self.dma_transfer();
            },
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

    pub fn dma_transfer(&mut self) {
        if self.transfer_count == 159 {
            self.transfer_count = 0;
            self.transfering = false;
            return;
        }

        let start_addr = 0xFE00;
        let cur_addr = start_addr + self.transfer_count;

        let from_addr = (self.dma as u16) << 8;
        let cur_from = from_addr + self.transfer_count;

        self.set(
            cur_addr,
            self.get(cur_from)
        );
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
}