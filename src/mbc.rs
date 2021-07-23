
enum Hardware {
    RealTimeClock,
    Rumble,
    Accelerometer,
    BatteryRam,
}

pub struct MBCProperties {
    rom_banks: u16,
    ram_total: u16,
    ram_banks: u16,
    features: Vec<Hardware>
}
pub struct MBCBuilder {}

impl MBCBuilder {
    pub fn get_mbc_from_header(type_header: u8, rom_header: u8, ram_header: u8) -> Box<dyn MBC> {
        let rom_banks = MBCBuilder::get_rom_banks_from_header(rom_header);
        let (ram_total, ram_banks) = MBCBuilder::get_ram_size_from_header(ram_header);
        let features = MBCBuilder::get_features_from_header(type_header);
        let properties = MBCProperties {
            rom_banks,
            ram_total,
            ram_banks,
            features
        };
        MBCBuilder::get_mbc_type_from_header(type_header, properties)
    }
    
    fn get_rom_banks_from_header(header: u8) -> u16 {
        match header {
            0x00 => 1,
            0x01 => 4,
            0x02 => 8,
            0x03 => 16,
            0x04 => 32,
            0x05 => 64,
            0x06 => 128,
            0x07 => 256,
            0x08 => 512,
            0x52 => 72,
            0x53 => 80,
            0x54 => 96,
            _ => 1
        }
    }
    //Returns: total, banks
    fn get_ram_size_from_header(header: u8) -> (u16, u16) {
        match header {
            0x00 => (0, 0),
            0x01 => (2, 1),
            0x02 => (8, 1),
            0x03 => (32, 4),
            0x04 => (128, 16),
            0x05 => (64, 8),
            _ => panic!("Impossible to have {} for RAM", header)
        }
    }

    fn get_features_from_header(header: u8) -> Vec<Hardware> {
        match header {
            0x03 =>	vec![Hardware::BatteryRam],
            0x06 =>	vec![Hardware::BatteryRam],
            0x09 =>	vec![Hardware::BatteryRam],
            0x0D =>	vec![Hardware::BatteryRam],
            0x0F =>	vec![Hardware::RealTimeClock],
            0x10 =>	vec![Hardware::RealTimeClock, Hardware::BatteryRam],
            0x13 =>	vec![Hardware::BatteryRam],
            0x1B =>	vec![Hardware::BatteryRam],
            0x1C =>	vec![Hardware::Rumble],
            0x1D =>	vec![Hardware::Rumble],
            0x1E =>	vec![Hardware::Rumble, Hardware::BatteryRam],
            0x22 =>	vec![Hardware::Accelerometer, Hardware::Rumble, Hardware::BatteryRam],
            0xFF =>	vec![Hardware::BatteryRam],
            _ => vec![]
        }
    }
    fn get_mbc_type_from_header(header: u8, properties: MBCProperties) -> Box<dyn MBC> {
        match header {
            0x00 => Box::new(MBCNone::new(properties)),
            0x01 => Box::new(MBC1::new(properties)),
            0x02 =>	Box::new(MBC1::new(properties)),
            0x03 =>	Box::new(MBC1::new(properties)),
            0x06 =>	Box::new(MBC2::new(properties)),
            0x05 =>	Box::new(MBC2::new(properties)),
            0x08 =>	Box::new(MBCNone::new(properties)),
            0x09 =>	Box::new(MBCNone::new(properties)),
            0x0B =>	Box::new(MMM01::new(properties)), 
            0x0C =>	Box::new(MMM01::new(properties)), 
            0x0D =>	Box::new(MMM01::new(properties)), 
            0x0F =>	Box::new(MBC3::new(properties)),
            0x10 =>	Box::new(MBC3::new(properties)),
            0x11 =>	Box::new(MBC3::new(properties)),
            0x12 =>	Box::new(MBC3::new(properties)),
            0x13 =>	Box::new(MBC3::new(properties)),
            0x19 =>	Box::new(MBC5::new(properties)),
            0x1A =>	Box::new(MBC5::new(properties)),
            0x1B =>	Box::new(MBC5::new(properties)),
            0x1C =>	Box::new(MBC5::new(properties)),
            0x1D =>	Box::new(MBC5::new(properties)),
            0x1E =>	Box::new(MBC5::new(properties)),
            0x20 =>	Box::new(MBC6::new(properties)),
            0x22 =>	Box::new(MBC7::new(properties)),
            _ => panic!("Impossible to have {} for Type", header)
        }
    }

    pub fn undefined() -> MBCUndefined {
        MBCUndefined::new()
    }
}

pub trait MBC {
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, v: u8);
    fn load_cart(&mut self, rom: Vec<u8>);
}

pub struct MBCNone {
    properties: MBCProperties,
    rom_bank: [u8; 1024 * 32],
    ram_bank: [u8; 1024 * 16],
}

impl MBCNone {
    pub fn new(properties: MBCProperties) -> Self {
        Self {
            properties,
            rom_bank: [0x0; 1024 * 32],
            ram_bank: [0x0; 1024 * 16],
        }
    }
}

impl MBC for MBCNone {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x7FFF => self.rom_bank[addr as usize],
            0xA000..=0xBFFF => {
                let ram_size = self.properties.ram_total * 1024;
                let ram_pos = 0xA000 - addr;
                if ram_pos > ram_size {
                    return 0xFF;
                }
                self.ram_bank[ram_pos as usize]
            },
            _ => 0xFF
        }
    }

    fn write(&mut self, _addr: u16, _value: u8) {
        return;
    }

    fn load_cart(&mut self, rom: Vec<u8>) {
        for (i, data) in rom.iter().enumerate() {
            self.rom_bank[i] = *data;
        }
    }
}

pub struct MBC1 {
    properties: MBCProperties,
    active_rom: u8,
    active_ram: u8,
    ram_enabled: bool,
    rom_banks: Vec<[u8; 16384]>,
    ram_banks: Vec<[u8; 16384]>,
}

impl MBC1 {
    pub fn new(properties: MBCProperties) -> Self {
        let ram_banks = properties.ram_banks;
        let rom_banks = properties.rom_banks;
        Self {
            properties,
            active_rom: 1,
            active_ram: 1,
            ram_enabled: false,
            rom_banks: vec![[0x0; 16384]; rom_banks as usize],
            ram_banks: vec![[0x0; 16384]; ram_banks as usize],
        }
    }
}

impl MBC for MBC1 {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x3FFF => self.rom_banks[0][addr as usize],
            0x4000..=0x7FFF => self.rom_banks[self.active_rom as usize][(addr - 0x4000) as usize],
            0xA000..=0xBFFF => {
                if self.ram_enabled {
                    let mut ram_addr = (addr - 0xA000) as usize;
                    if self.properties.ram_total == 2 || self.properties.ram_total == 8 {
                        ram_addr = ram_addr % self.properties.ram_total as usize;
                    }
                    self.ram_banks[self.active_ram as usize][ram_addr]
                } else {
                    0xFF
                }
            },
            _ => 0xFF
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x1FFF => if value & 0x0F == 0x0A {self.ram_enabled = true} else {self.ram_enabled = false}
            0x2000..=0x3FFF => {
                let mask = self.properties.ram_banks - 1;
            }
            0xA000..=0xBFFF => {
                if self.ram_enabled {
                    let mut ram_addr = (addr - 0xA000) as usize;
                    if self.properties.ram_total == 2 || self.properties.ram_total == 8 {
                        ram_addr = ram_addr % self.properties.ram_total as usize;
                    }
                    self.ram_banks[self.active_ram as usize][ram_addr] = value;
                }
            },
            _ => ()
        }
    }

    fn load_cart(&mut self, rom: Vec<u8>) {
        todo!()
    }
}

pub struct MBC2 {
    properties: MBCProperties,
}

impl MBC2 {
    pub fn new(properties: MBCProperties) -> Self {
        Self {
            properties
        }
    }
}

impl MBC for MBC2 {
    fn read(&self, addr: u16) -> u8 {
        todo!()
    }

    fn write(&mut self, addr: u16, v: u8) {
        todo!()
    }

    fn load_cart(&mut self, rom: Vec<u8>) {
        todo!()
    }
}

pub struct MBC3 {
    properties: MBCProperties
}

impl MBC3 {
    pub fn new(properties: MBCProperties) -> Self {
        Self {
            properties
        }
    }
}

impl MBC for MBC3 {
    fn read(&self, addr: u16) -> u8 {
        todo!()
    }

    fn write(&mut self, addr: u16, v: u8) {
        todo!()
    }

    fn load_cart(&mut self, rom: Vec<u8>) {
        todo!()
    }
}

pub struct MBC4 {
    properties: MBCProperties
}

impl MBC4 {
    pub fn new(properties: MBCProperties) -> Self {
        Self {
            properties
        }
    }
}

impl MBC for MBC4 {
    fn read(&self, addr: u16) -> u8 {
        todo!()
    }

    fn write(&mut self, addr: u16, v: u8) {
        todo!()
    }

    fn load_cart(&mut self, rom: Vec<u8>) {
        todo!()
    }
}

pub struct MBC5 {
    properties: MBCProperties
}

impl MBC5 {
    pub fn new(properties: MBCProperties) -> Self {
        Self {
            properties
        }
    }
}

impl MBC for MBC5 {
    fn read(&self, addr: u16) -> u8 {
        todo!()
    }

    fn write(&mut self, addr: u16, v: u8) {
        todo!()
    }

    fn load_cart(&mut self, rom: Vec<u8>) {
        todo!()
    }
}

pub struct MBC6 {
    properties: MBCProperties
}

impl MBC6 {
    pub fn new(properties: MBCProperties) -> Self {
        Self {
            properties
        }
    }
}

impl MBC for MBC6 {
    fn read(&self, addr: u16) -> u8 {
        todo!()
    }

    fn write(&mut self, addr: u16, v: u8) {
        todo!()
    }

    fn load_cart(&mut self, rom: Vec<u8>) {
        todo!()
    }
}

pub struct MBC7 {
    properties: MBCProperties
}

impl MBC7 {
    pub fn new(properties: MBCProperties) -> Self {
        Self {
            properties
        }
    }
}

impl MBC for MBC7 {
    fn read(&self, addr: u16) -> u8 {
        todo!()
    }

    fn write(&mut self, addr: u16, v: u8) {
        todo!()
    }

    fn load_cart(&mut self, rom: Vec<u8>) {
        todo!()
    }
}

pub struct MMM01 {
    properties: MBCProperties
}

impl MMM01 {
    pub fn new(properties: MBCProperties) -> Self {
        Self {
            properties
        }
    }
}

impl MBC for MMM01 {
    fn read(&self, addr: u16) -> u8 {
        todo!()
    }

    fn write(&mut self, addr: u16, v: u8) {
        todo!()
    }

    fn load_cart(&mut self, rom: Vec<u8>) {
        todo!()
    }
}

pub struct MBCUndefined {}

impl MBCUndefined {
    pub fn new() -> Self {
        Self {}
    }
}

impl MBC for MBCUndefined {
    fn read(&self, addr: u16) -> u8 {
        todo!()
    }

    fn write(&mut self, addr: u16, v: u8) {
        todo!()
    }

    fn load_cart(&mut self, rom: Vec<u8>) {
        panic!("Never initialized the MBC");
    }
}
