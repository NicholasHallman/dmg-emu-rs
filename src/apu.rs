use js_sys::SharedArrayBuffer;

const SAMPLE_SIZE: usize = 44100 / 60;
const SAMPLE_RATE: usize = 44100;

pub struct APU {
    channel_1_reg: [u8; 5],
    channel_2_reg: [u8; 4],
    channel_3_reg: [u8; 5],
    channel_4_reg: [u8; 4],
    wave_ram: [u8; 16],
    NR50: u8,
    NR51: u8,
    NR52: u8,
    channel_1_buffer: [u8; SAMPLE_SIZE],
    channel_2_buffer: [u8; SAMPLE_SIZE],
    channel_3_buffer: [u8; SAMPLE_SIZE],
    channel_4_buffer: [u8; SAMPLE_SIZE],
} 

impl APU {
    pub fn new() -> Self {
        Self {
            channel_1_reg: [0; 5],
            channel_2_reg: [0; 4],
            channel_3_reg: [0; 5],
            channel_4_reg: [0; 4],
            wave_ram: [0; 16],
            NR50: 0,
            NR51: 0,
            NR52: 0,
            channel_1_buffer: [0; SAMPLE_SIZE],
            channel_2_buffer: [0; SAMPLE_SIZE],
            channel_3_buffer: [0; SAMPLE_SIZE],
            channel_4_buffer: [0; SAMPLE_SIZE],
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF10..=0xFF14 => self.channel_1_reg[(addr as usize) - 0xFF10] = value,
            0xFF16..=0xFF19 => self.channel_2_reg[(addr as usize) - 0xFF16] = value,
            0xFF1A..=0xFF1E => self.channel_3_reg[(addr as usize) - 0xFF1A] = value,
            0xFF30..=0xFF3F => self.wave_ram [(addr as usize) - 0xFF30] = value,
            0xFF20..=0xFF23 => self.channel_4_reg[(addr as usize) - 0xFF20] = value,
            0xFF24 => self.NR50 = value,
            0xFF25 => self.NR51 = value,
            0xFF26 => self.NR52 = value,
            _ => panic!("APU does not cover address range {}", addr)
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xFF10..=0xFF14 => self.channel_1_reg[(addr as usize) - 0xFF10],
            0xFF16..=0xFF19 => self.channel_2_reg[(addr as usize) - 0xFF16],
            0xFF1A..=0xFF1E => self.channel_3_reg[(addr as usize) - 0xFF1A],
            0xFF30..=0xFF3F => self.wave_ram [(addr as usize) - 0xFF30],
            0xFF20..=0xFF23 => self.channel_4_reg[(addr as usize) - 0xFF20],
            0xFF24 => self.NR50,
            0xFF25 => self.NR51,
            0xFF26 => self.NR52,
            _ => 0xFF
        }
    }

    pub fn get_shared_buffer(&self) -> [&[u8;SAMPLE_SIZE]; 4] {
        [
            &self.channel_1_buffer,
            &self.channel_2_buffer,
            &self.channel_3_buffer,
            &self.channel_4_buffer,
        ]
    }

    fn buffer_channel_1(&mut self) {
        let x: u32 = (self.channel_1_reg[3] as u32 & 3) << 8 | self.channel_1_reg[2] as u32;
        let frequency = 131072 / ( 2048 - x );
        for i in 0..self.channel_1_buffer.len() {
            let value: u8 = if (i / (frequency as usize / 2)) % 2 == 0 {0} else {1};
            self.channel_1_buffer[i] = value;
        }
    }

    pub fn tick(&mut self) {
        // populate channels;
        // self.buffer_channel_1();
    }
}