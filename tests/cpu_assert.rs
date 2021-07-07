
use dmg_emu::{Emu, cpu::{Flag, HalfReg, Reg}}; 

pub trait EmuTester {
    fn assert(self) -> Assert;
}

impl EmuTester for Emu {

    fn assert(self) -> Assert {
        Assert::new(self)
    }
}

pub trait EmuTestHelpers {
    fn tick_cpu(&mut self, times: u16);
    fn tick_till_done(&mut self);
}

impl EmuTestHelpers for Emu {
    fn tick_cpu(&mut self, times: u16) {
        for _ in 0..times {
            self.tick();
        }
    }
    fn tick_till_done(&mut self) {
        self.tick();
        while self.cpu().current_cycle != 1 {
            self.tick();
        }
    }
}


pub enum Register {
    Half(HalfReg),
    Full(Reg)
}

impl From<HalfReg> for Register {
    fn from(half: HalfReg) -> Self {
        Register::Half(half)
    }
}

impl From<Reg> for Register {
    fn from(full: Reg) -> Self {
        Register::Full(full)
    }
}

pub enum RegisterValue {
    Half(u8),
    Full(u16)
}

impl From<u8> for RegisterValue {
    fn from(half: u8) -> Self {
        RegisterValue::Half(half)
    }
}

impl From<u16> for RegisterValue {
    fn from(full: u16) -> Self {
        RegisterValue::Full(full)
    }
}

enum LastCompared {
    Mem,
    Flag,
    Reg,
    None
}

pub struct Assert {
    emu: Emu,
    last_value: CompValue,
    last_compared: LastCompared
}

pub enum CompValue {
    U16(u16),
    Bool(bool),
    U8(u8),
    None
}

impl From<bool> for CompValue {
    fn from(b: bool) -> CompValue {
        CompValue::Bool(b)
    }
}

impl From<u8> for CompValue {
    fn from(v: u8) -> CompValue {
        CompValue::U8(v)
    }
}

impl From<u16> for CompValue {
    fn from(v: u16) -> CompValue {
        CompValue::U16(v)
    }
}

impl Assert {
    pub fn new(emu: Emu) -> Self {
        Self {
            emu,
            last_value: CompValue::None,
            last_compared: LastCompared::None
        }
    }

    pub fn mem(&mut self, addr: u16) {
        let value = self.emu.mem().get(addr);
        self.last_value = CompValue::U8(value);
        self.last_compared = LastCompared::Mem;
    }

    pub fn flag(&mut self, f: Flag) {
        let value = self.emu.cpu().get_flag(f);
        self.last_value = CompValue::Bool(value);
        self.last_compared = LastCompared::Flag;
    }

    pub fn reg<R>(&mut self, r: R) where R: Into<Register>{
        match r.into() {
            Register::Half(h) => 
                self.last_value = CompValue::U8(self.emu.cpu().get_byte_reg(&h)),
            Register::Full(f) => 
                self.last_value = CompValue::U16(self.emu.cpu().get_word_reg(&f)),
        };
        self.last_compared = LastCompared::Reg;
    }

    pub fn equals<T>(&self, given: T) where T: Into<CompValue> {
        match self.last_compared {
            LastCompared::Mem => {
                if let CompValue::U8(stored_v) = self.last_value {
                    if let CompValue::U8(given_v) = given.into() {
                        assert_eq!(stored_v, given_v);
                    } else {
                        panic!("Expect Memory to be a u8");
                    }
                }
            }
            LastCompared::Flag => {
                if let CompValue::Bool(stored_v) = self.last_value {
                    if let CompValue::Bool(given_v) = given.into() {
                        assert_eq!(stored_v, given_v);
                    } else {
                        panic!("Expect Flag to be a bool");
                    }
                }
            }
            LastCompared::Reg => {
                match self.last_value {
                    CompValue::U16(stored_v) => {
                        if let CompValue::U16(given_v) = given.into() {
                            assert_eq!(stored_v, given_v);
                        } else {
                            panic!("Full registers expect u16")
                        }
                    },
                    CompValue::U8(stored_v) => {
                        if let CompValue::U8(given_v) = given.into() {
                            assert_eq!(stored_v, given_v);
                        } else {
                            panic!("Half registers expect u8")
                        }
                    },
                    _ => ()
                }
            }
            LastCompared::None => panic!("Nothing to compare against")
        }
    }
}

/* 
assert.mem().equals()
    .flag().equals()
    .reg().equals()
*/
