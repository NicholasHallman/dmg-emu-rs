use std::time::Instant;
use wasm_bindgen::prelude::*;

use crate::mem::Mem;

const WIDTH: u8 = 160;
const HEIGHT: u8 = 144;

const COLORS: [[u8; 4]; 4] = [
    [0xF3, 0xF0, 0xDE, 0xFF],
    [0x63, 0x91, 0xB0, 0xFF],
    [0x1E, 0x3A, 0x83, 0xFF],
    [0x3D, 0x17, 0x52, 0xFF],
    //3d1752
];
enum PPUMode {
    OAMSearch,
    PixelTransfer,
    HBlank,
    VBlank
}

#[wasm_bindgen]
pub struct Ppu {
    mode: PPUMode,
    cycles: usize,
    visible_sprites: [u16; 10],
    total_o: usize,
    fetcher: Fetcher,
    bw_fifo: Fifo,
    ob_fifo: Fifo,
    display_buffer: [u8; 160*144*4],
    x: u8,
    pub wait_for_frame: bool,
    pub ready: bool,

    current_o: u16,
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            mode: PPUMode::OAMSearch,
            cycles: 0,
            visible_sprites: [0; 10],
            fetcher: Fetcher::new(),
            bw_fifo: Fifo::new(false),
            ob_fifo: Fifo::new(true),
            display_buffer: [0; 160*144*4],
            x: 0,
            ready: false,
            wait_for_frame: false,
            current_o: 0,
            total_o: 0,
        }
    }

    pub fn tick(&mut self, mem: &mut Mem) {
        mem.ppu_access = true;
        match self.mode {
            PPUMode::OAMSearch => self.oam_search(mem),
            PPUMode::PixelTransfer => self.pixel_transfer(mem),
            PPUMode::HBlank => self.h_blank(mem),
            PPUMode::VBlank => self.v_blank(mem)
        };
        mem.ppu_access = false;
    }

    pub fn draw(&self, frame: &mut [u8]) {
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {

            let palet_number = self.display_buffer[i];

            let rgba: [u8; 4] = COLORS[palet_number as usize];

            pixel.copy_from_slice(&rgba);
        }
    }

    pub fn get_buffer(&self) -> &[u8; 160*144*4] {
        &self.display_buffer
    }

    fn prep_oam_search(&mut self, mem: &mut Mem) {
        self.current_o = 0;
        self.total_o = 0;
        self.visible_sprites = [0; 10];
        self.mode = PPUMode::OAMSearch;
        mem.set_lcd_stat(0, 2);
    }

    fn oam_search(&mut self, mem: &mut Mem) {
        mem.set_lcd_stat(0, 2);

        // search for visible sprites
        let oam_table_addr: u16 = 0xFE00;
        let curr_o_addr = oam_table_addr + (self.current_o * 4);
        let oa_y = mem.get(curr_o_addr);
        let oa_height :u8 = if (mem.get(0xFF40) & 4) == 4 {16} else {8};
        let y = mem.get(0xFF44);

        if oa_y >= 160 {
            self.current_o += 1;
            return;
        }

        let is_below_top = oa_y + oa_height >= 16;
        let is_on_current_ly = oa_y <= y + 16 && oa_y + oa_height > y + 16;

        if is_below_top && is_on_current_ly && self.total_o < 10 { 
            self.visible_sprites[self.total_o] = curr_o_addr;
            self.total_o += 1;
        }

        self.current_o += 1;

        self.cycles += 1;
        if self.cycles == 40 {
            self.bw_fifo.clear();
            self.ob_fifo.clear();
            self.fetcher.reset(mem);
            self.mode = PPUMode::PixelTransfer;
        }
    }

    fn pixel_transfer(&mut self, mem: &mut Mem) {
        mem.set_lcd_stat(0, 3);
        
        self.fetcher.tick(mem, self.cycles % 2, &mut self.bw_fifo, &mut self.ob_fifo, self.x, &self.visible_sprites);
        let maybe_bw_pixel = self.bw_fifo.tick();
        let maybe_ob_pixel = self.ob_fifo.tick();
        let ob_enabled = mem.get(0xFF40) >> 1 & 1 == 1;

        if let Some(bw_pixel) = maybe_bw_pixel {

            let y = mem.get(0xFF44) as usize;
            let x = self.x as usize;
            let pos: usize = ((x + (y * 160)) * 4) as usize;
            let pixel;

            let bw_pixel_num = (bw_pixel >> 2) & 3;
            let bw_pixel_color = bw_pixel & 3;

            if let Some(ob_pixel) = maybe_ob_pixel {
                let ob_pixel_pri = (ob_pixel >> 5) & 1;
                let ob_pixel_num = (ob_pixel >> 2) & 3;
                let ob_pixel_color = ob_pixel & 3;

                pixel = {
                    if ob_pixel_num == 0 || !ob_enabled{
                        bw_pixel_color
                    } else if ob_pixel_pri == 1 && bw_pixel_num > 0 {
                        bw_pixel_color
                    } else {
                        ob_pixel_color
                    }
                };

            } else {
                pixel = bw_pixel_color;
            }
            let rgba = COLORS[pixel as usize];
            self.display_buffer[pos] = rgba[0];
            self.display_buffer[pos+1] = rgba[1];
            self.display_buffer[pos+2] = rgba[2];
            self.display_buffer[pos+3] = rgba[3];
            
            self.x += 1;
        }

        self.cycles += 1;
        if self.x == 160 {
            self.x = 0;
            self.mode = PPUMode::HBlank
        }
    }

    fn h_blank(&mut self, mem: &mut Mem) {
        mem.set_lcd_stat(0, 0);

        self.cycles += 1;
        if self.cycles == 456 {
            self.cycles = 0;
            let ly = mem.get(0xFF44) + 1;
            mem.set_ly(ly);
            if ly == 144 {
                self.ready = true;
                self.mode = PPUMode::VBlank;
                let flags = mem.get(0xFF0F) | 1;
                mem.set(0xFF0F, flags); // interupt flag
            } else {
                self.prep_oam_search(mem);
            }
        }
    }

    fn v_blank(&mut self, mem: &mut Mem) {
        mem.set_lcd_stat(0, 1);

        self.ready = false;
        self.cycles += 1;
        if self.cycles == 456 {
            self.cycles = 0;
            let ly = mem.get(0xFF44) + 1;
            mem.set_ly(ly);
            if ly == 153 {
                self.fetcher.reset_win();
                mem.set_ly(0);
                self.prep_oam_search(mem);
            }
        }
    }
}

enum FetcherMode {
    ReadTile,
    Data0,
    Data1,
    Idle
}

enum BgWin {
    Background,
    Window
}

impl PartialEq for BgWin {
    fn eq(&self, other: &Self) -> bool {
        match self {
            &BgWin::Background => match other {
                &BgWin::Background => true,
                _ => false
            },
            &BgWin::Window => match other {
                &BgWin::Window => true,
                _ => false
            }
        }
    }
}

pub struct Fetcher {
    mode: FetcherMode,
    data0: u8,
    data1: u8,
    tile_num: u8,
    backup_tile: u16,
    curr_tile: u16,
    bg_win_on: bool,
    bg: BgWin,
    window_line: u16
}

impl Fetcher {

    pub fn new() -> Self {
        Self {
            mode: FetcherMode::ReadTile,
            data0: 0,
            data1: 0,
            tile_num: 0,
            curr_tile: 0,
            bg_win_on: true,
            bg: BgWin::Background,
            window_line: 0xFFFF,
            backup_tile: 0
        }
    }

    pub fn reset(&mut self, mem: &Mem) {
        self.mode = FetcherMode::ReadTile;
        self.data0 = 0;
        self.data1 = 0;
        self.tile_num = 0;
        let scx = mem.get(0xFF43);
        self.curr_tile = (scx / 8) as u16;
        self.bg_win_on = true;
        self.backup_tile = 0;
    }

    pub fn reset_win(&mut self) {
        self.window_line = 0xFFFF;
    }

    pub fn tick(&mut self, mem: &Mem, current_cycle: usize, bw_fifo: &mut Fifo, ob_fifo: &mut Fifo, x: u8, oams: &[u16; 10]) {
        self.check_oam(x, oams, ob_fifo, mem);
        self.win_or_back(mem, x, bw_fifo);
        if current_cycle == 1 {return}
        match self.mode {
            FetcherMode::ReadTile => self.read_tile(mem),
            FetcherMode::Data0 => self.get_data0(mem),
            FetcherMode::Data1 => self.get_data1(mem),
            FetcherMode::Idle => self.idle(bw_fifo, mem),
        }
    }

    pub fn check_oam(&mut self, current_x: u8, oams: &[u16; 10], ob_fifo: &mut Fifo, mem: &Mem) {
        // check if the x is the start of an oam
        // if it is, load the oas sprite into the ob_fifo
        for oa in oams {
            if oa.to_owned() == 0 {continue;}
            let o_y = mem.get(*oa) as u16;
            let o_x = mem.get(oa + 1);
            let o_a = mem.get(oa + 3);
            let x_flip = {
                o_a >> 5 & 1 == 1
            };
            let y_flip = {
                o_a >> 6 & 1 == 1
            };

            let ly = mem.get(0xFF44) as u16;
            let line_num = {
                let y = (ly + 16) - o_y;
                let height = self.get_obj_size(mem) as u16;
                if !y_flip {
                    y % height
                } else {
                    (height - 1) - y
                }
            };
            
            if o_x == current_x + 8 {
                // ob_fifo.clear();
                // get the sprite data
                let tile_num: u16 = {
                    if self.get_obj_size(mem) == 16 {
                        if line_num >= 8 {
                            (mem.get(oa + 2) as u16) & 0xFE
                        } else {
                            (mem.get(oa + 2) as u16) & 0xFE
                        }
                    } else {
                        mem.get(oa + 2) as u16
                    }
                };

                let tile_addr = 0x8000 + (tile_num * 16);
                let tile_line = tile_addr + (line_num * 2);
                
                let data0 = mem.get(tile_line);
                let data1 = mem.get(tile_line + 1);

                let palette_id = (o_a >> 4) & 1 ;
                let palette = if palette_id == 0 {mem.get(0xFF48)} else {mem.get(0xFF49)};

                let range: Vec<u8> = {
                    if x_flip {
                        (0..8).collect()
                    } else {
                        (0..8).rev().collect()
                    }
                };

                let priority = o_a >> 7 & 1;
                let mut j = 0;
                for i in range {
                    let high = (data1 >> i) & 1;
                    let low = (data0 >> i) & 1;
                    let color_num = (high << 1 | low) & 3;
                    
                    let color = (palette >> (color_num * 2)) & 3;
                
                    ob_fifo.mix(priority, color_num, color, j);
                    j+= 1;
                }
                break;
            }
        }
    }

    pub fn win_or_back(&mut self, mem: &Mem, x: u8, fifo: &mut Fifo) {
        let w_x = mem.get(0xFF4B);
        let w_y = mem.get(0xFF4A);

        if w_y > 143 || w_x > 166 {
            self.bg = BgWin::Background;
            return;
        }

        let w_on = self.get_win_enabled(mem) && w_x != 0;
        let ly = mem.get(0xFF44);

        let x_in_win = w_x <= (x + 7) && w_x + (WIDTH as u8) > (x + 7);
        let y_in_win = w_y <= ly && w_y + (HEIGHT as u8) > ly;

        if w_on && x_in_win && y_in_win {
            if self.bg == BgWin::Background {
                self.mode = FetcherMode::ReadTile;
                fifo.clear();
                if w_x == (x + 7) {
                    self.backup_tile = self.curr_tile;
                    self.curr_tile = 0;
                    self.window_line = self.window_line.overflowing_add(1).0;
                }
            } 
            self.bg = BgWin::Window;
        } else {
            if self.bg == BgWin::Window {
                self.mode = FetcherMode::ReadTile;
                self.curr_tile += self.backup_tile;
                fifo.clear();
            }
            self.bg = BgWin::Background;
        }
    }

    pub fn read_tile(&mut self, mem: &Mem) {
        self.bg_win_on = self.get_bg_win_enabled(&mem);
        
        let ly = if self.bg == BgWin::Background {
            self.get_ly_add_scy(&mem) as u16
        } else {
            self.window_line
            // let wy = mem.get(0xFF4A) as u16;
            // ly - wy
        };

        let line_num = ly / 8;
        let tile_map_row: u16 = {
            if self.bg == BgWin::Background {
                self.get_bg_map_addr(mem) 
            } else {
                self.get_win_map_addr(mem)
            }
        } + (line_num * 0x20);

        let addr = tile_map_row + self.curr_tile;

        self.tile_num = mem.get(addr);
        self.mode = FetcherMode::Data0;
    }

    pub fn get_data0(&mut self, mem: &Mem) {
        let ly = self.get_ly_add_scy(&mem) as u16;
        let line = ly % 8;
        let raw;
        let data_addr = self.get_bg_win_data_addr(mem);
        if data_addr == 0x8000 {
            let index: u16 = self.tile_num as u16 * 16;
            let offset: u16 = data_addr + index;

            raw = mem.get(offset + (line * 2));
        } else {
            let stile_num: i16 = self.tile_num as i16;
            let index = stile_num * 16;
            let offset: i32 = (data_addr as i32) + index as i32;
            let offset: u16 = offset as u16;
            raw = mem.get((offset + (line * 2)) as u16 );
        }

        self.data0 = 0;
        // for i in 0..=7 {
        //     self.data0 |= (raw << i) & 0x80;
        //     self.data0 = self.data0 >> 1;
        // }
        self.data0 = raw;
        self.mode = FetcherMode::Data1;
    }

    pub fn get_data1(&mut self, mem: &Mem) {
        let ly = self.get_ly_add_scy(&mem) as u16;
        let line = ly % 8;
        let raw;

        let data_addr = self.get_bg_win_data_addr(mem);
        if data_addr == 0x8000 {
            let index: u16 = self.tile_num as u16 * 16;
            let offset: u16 = data_addr + index;

            raw = mem.get(offset + (line * 2) + 1);
        } else {
            let stile_num: i16 = self.tile_num as i16;
            let index = stile_num * 16;
            let offset: i32 = (data_addr as i32) + index as i32;
            let offset: u16 = offset as u16;
            raw = mem.get((offset + (line * 2) + 1) as u16 );
        }

        self.data1 = 0;
        // for i in 0..=7 {
        //     self.data1 |= (raw << i) & 0x80;
        //     self.data1 = self.data0 >> 1;
        // }
        self.data1 = raw;
        self.mode = FetcherMode::Idle;
    }

    pub fn idle(&mut self, bw_fifo: &mut Fifo, mem: &Mem) {
        if bw_fifo.can_push() {
            if self.bg_win_on {
                for i in (0..8).rev() {
                    let high = (self.data1 >> i) & 1;
                    let low = (self.data0 >> i) & 1;
                    let color_num = (high << 1 | low) & 3;

                    let palette = mem.get(0xFF47);
                    let color = (palette >> (color_num * 2)) & 3;
                    bw_fifo.push(color_num << 2 | color);
                }
            } else {
                for _ in 0..8 {
                    bw_fifo.push(0);
                }
            }
            self.curr_tile = (self.curr_tile + 1) % 32;
            self.mode = FetcherMode::ReadTile;
        }
    }
}

pub struct Fifo {
    data: u128,
    tail: u8,
    is_for_sprite: bool,
}

impl Fifo {
    pub fn new(is_for_sprite: bool) -> Self {
        Self {
            data: 0,
            tail: 0,
            is_for_sprite,
        }
    }

    pub fn tick(&mut self) -> Option<u8> {
        if self.can_pop() {
            return Some(self.pop());
        }
        None
    }

    pub fn clear(&mut self) {
        self.data = 0;
        self.tail = 0;
    }
    
    pub fn can_pop(&self) -> bool {
        if self.is_for_sprite {
            self.tail > 0
        } else {
            self.tail >= 8
        }
    }

    pub fn can_push(&self) -> bool {
        self.tail < 8
    }

    pub fn push(&mut self, value: u8) {
        let value: u128 = (value as u128) & 0xFF;
        let dist = ((16 - self.tail) * 8) - 8;
        let value = value << dist;
        self.data |= value;
        self.tail += 1;
    }

    pub fn mix(&mut self, priority: u8, color_num: u8, color: u8, i: u8) {
        if i >= self.tail {
            self.push(priority << 5 | color_num << 2 | color);
        }
        let cur = self.peek(i);
        let cur_num = (cur >> 2) & 3;
        if cur_num == 0 {
            self.replace(i, priority << 5 | color_num << 2 | color );
        }
    }

    pub fn replace(&mut self, i: u8, value: u8) {
        let value: u128 = (value as u128) & 0xFF;
        let dist = ((16 - i) * 8) - 8;
        let mask: u128 = !0xFF << dist;
        let value: u128 = value << dist;
        self.data &= mask;
        self.data |= value;
    }

    pub fn peek(&self, i: u8) -> u8 {
        let tmp = self.data << (8 * i);
        (tmp >> 120) as u8
    }

    pub fn pop(&mut self) -> u8 {
        let top: u8 = (self.data >> 120) as u8;
        self.data = self.data << 8;
        self.tail -= 1;
        top
    }
}

trait FetcherHelpers {
    fn lcdc_bit(&self, mem: &Mem, b: u8) -> bool;

    fn ppu_enabled(&self, mem: &Mem) -> bool;
    fn get_win_map_addr(&self, mem: &Mem) -> u16;
    fn get_win_enabled(&self, mem: &Mem) -> bool;
    fn get_bg_win_data_addr(&self, mem: &Mem) -> u16;
    fn get_bg_map_addr(&self, mem: &Mem) -> u16;
    fn get_obj_size(&self, mem: &Mem) -> u8;
    fn get_obj_enabled(&self, mem: &Mem) -> bool;
    fn get_bg_win_enabled(&self, mem: &Mem) -> bool;

    fn get_ly_add_scy(&self, mem:&Mem) -> u8;
}

impl FetcherHelpers for Fetcher {

    fn lcdc_bit(&self, mem: &Mem, b: u8) -> bool {
        let lcdc = mem.get(0xFF40);
        (lcdc >> b) & 1 == 1
    }

    fn ppu_enabled(&self, mem: &Mem) -> bool {
        self.lcdc_bit(mem, 7)
    }

    fn get_win_map_addr(&self, mem: &Mem) -> u16 {
        if self.lcdc_bit(mem, 6) {
            return 0x9C00;
        }
        return 0x9800;
    }

    fn get_win_enabled(&self, mem: &Mem) -> bool {
        self.lcdc_bit(mem, 5)
    }

    fn get_bg_win_data_addr(&self, mem: &Mem) -> u16 {
        if self.lcdc_bit(mem, 4) {
            return 0x8000;
        }
        return 0x9000;
    }

    fn get_bg_map_addr(&self, mem: &Mem) -> u16 {
        if self.lcdc_bit(mem, 3) {
            return 0x9C00;
        }
        return 0x9800;
    }

    fn get_obj_size(&self, mem: &Mem) -> u8 {
        if self.lcdc_bit(mem, 2) {
            return 16;
        }
        return 8;
    }

    fn get_obj_enabled(&self, mem: &Mem) -> bool {
        self.lcdc_bit(mem, 1)
    }

    fn get_bg_win_enabled(&self, mem: &Mem) -> bool {
        self.lcdc_bit(mem, 0)
    }

    fn get_ly_add_scy(&self, mem:&Mem) -> u8 {
        let scy = mem.get(0xFF42);
        let ly = mem.get(0xFF44);
        ly.overflowing_add(scy).0
    }
}