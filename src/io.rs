
pub const SB_ADDR: u16 = 0xFF01; 
pub const SC_ADDR: u16 = 0xFF02; 
pub const P1_ADDR: u16 = 0xFF00;
pub struct Serial {
    SB: u8,
    SC: u8
}

impl Serial {

    pub fn new() -> Self {
        Self {
            SB: 0,
            SC: 0
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            SB_ADDR => {
                self.SB = value;
            },
            SC_ADDR => {
                self.SC = value;
                if value == 0x81 {
                    println!("Serial: {}", self.SB as char);
                }
            },
            _ => panic!("Serial does not exist at this address")
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            SB_ADDR => self.SB,
            SC_ADDR => self.SC,
            _ => panic!("Serial does not exist at this address")
        }
    }
}

const DIV_ADDR: u16 = 0xFF04;
const TIMA_ADDR: u16 = 0xFF05;
const TMA_ADDR: u16 = 0xFF06;
const TAC_ADDR: u16 = 0xFF07;

struct Timer {
    DIV: u8,
    TIMA: u8,
    TMA: u8,
    TAC: u8
}

impl Timer {
    pub fn new() -> Self {
        Self {
            DIV: 0,
            TIMA: 0,
            TMA: 0,
            TAC: 0,
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            DIV_ADDR => self.DIV = value,
            TIMA_ADDR => self.TIMA = value,
            TMA_ADDR => self.TMA = value,
            TAC_ADDR => self.TAC = value,
            _ => panic!("Timer does not exist at this address")
        }
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        match addr {
            DIV_ADDR => self.DIV,
            TIMA_ADDR => self.TIMA,
            TMA_ADDR => self.TMA,
            TAC_ADDR => self.TAC,
            _ => panic!("Timer does not exist at this address")
        }
    }
}

pub enum Button {
    A,
    B,
    Up,
    Down,
    Left,
    Right,
    Start,
    Select
}

impl From<char> for Button {
    fn from(c: char) -> Self {
        match c {
            'a' => Button::A,
            'b' => Button::B,
            'u' => Button::Up,
            'd' => Button::Down,
            'l' => Button::Left,
            'r' => Button::Right,
            't' => Button::Start,
            'e' => Button::Select,
            _ => panic!("Character cannot be converted to button")
        }
    }
}

impl From<String> for Button {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "a" => Button::A,
            "b" => Button::B,
            "up" => Button::Up,
            "down" => Button::Down,
            "left" => Button::Left,
            "right" => Button::Right,
            "start" => Button::Start,
            "select" => Button::Select,
            _ => panic!("String cannot be converted to button")
        }
    }
}

pub struct Joypad {
    a: bool,
    b: bool,
    start: bool,
    select: bool,
    up: bool,
    down: bool,
    left: bool,
    right: bool,
    arrow_select: bool,
    action_select: bool
}

impl Joypad {
    pub fn new() -> Self {
        Self {
            a: false,
            b: false,
            start: false,
            select: false,
            up: false,
            down: false,
            left: false,
            right: false,
            arrow_select: false,
            action_select: false
        }
    }
    pub fn read(&self) -> u8 {
        if !self.arrow_select {
            return ((!self.down) as u8) << 3 | ((!self.up) as u8) << 2 | ((!self.left) as u8) << 1 | (!self.right) as u8;
        } else {
            return ((!self.start) as u8) << 3 | ((!self.select) as u8) << 2 | ((!self.b) as u8) << 1 | ((!self.a) as u8);
        }
    }

    pub fn write(&mut self, mut value: u8) {
        self.action_select = (value >> 5) & 1 != 1;
        self.arrow_select = (value >> 4) & 1 != 1;
    }

    pub fn set(&mut self, b: Button, state: bool) {
        match b.into() {
            Button::A => self.a = state,
            Button::B => self.b = state,
            Button::Up => self.up = state,
            Button::Down => self.down = state,
            Button::Left => self.left = state,
            Button::Right => self.right = state,
            Button::Start => self.start = state,
            Button::Select => self.select = state,
        }
    }
}