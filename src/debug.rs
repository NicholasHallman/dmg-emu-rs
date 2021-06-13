
use crate::{Emu, cpu::{self, Cpu}, mem::Mem};
use std::{collections::HashMap, fs::{File, OpenOptions}, io::{stdout, Write}, time::Duration};
use KeyCode::Char;
use crossterm::{ExecutableCommand, Result, cursor::{MoveTo}, event::{Event, KeyCode, KeyEvent, poll, read}, style::{Color, Print, ResetColor, SetBackgroundColor}, terminal::{Clear, ClearType, self}};

pub struct DebugEmu {
    source: Vec<String>,
    source_map: HashMap<u16, usize>,
    line_breakpoints: Vec<usize>,
    pc_breakpoints: Vec<u16>,
    break_next: bool,
    current_line: u16,
    pub is_blocked: bool
}

impl DebugEmu {

    pub fn new() -> Self {
        Self {
            source: Vec::new(),
            source_map: HashMap::new(),
            line_breakpoints: Vec::new(),
            pc_breakpoints: Vec::new(),
            break_next: false,
            current_line: 0,
            is_blocked: false,
        }
    }

    pub fn add_breakpoint(&mut self, line_num: usize) {
        self.line_breakpoints.push(line_num);
    }

    pub fn add_pc_breakpoint(&mut self, addr: u16) {
        self.pc_breakpoints.push(addr);
    }

    pub fn print_registers(&self, emu: &Emu) -> Result<()> {

        stdout()
            .execute(MoveTo(0,0))?
            .execute(Print("Registers\n\n"))?
            .execute(Print(format!("AF : {:04X}\n", emu.cpu.AF)))?
            .execute(Print(format!("BC : {:04X}\n", emu.cpu.BC)))?
            .execute(Print(format!("DE : {:04X}\n", emu.cpu.DE)))?
            .execute(Print(format!("HL : {:04X}\n", emu.cpu.HL)))?
            .execute(Print(format!("PC : {:04X}\n", emu.cpu.PC)))?
            .execute(Print(format!("SP : {:04X}\n", emu.cpu.SP)))?
            .execute(Print("\nFlags\n"))?
            .execute(Print(format!("Z: {} N: {}\n", emu.cpu.AF >> 7 & 1, emu.cpu.AF >> 6 & 1)))?
            .execute(Print(format!("H: {} C: {} \n\n", emu.cpu.AF >> 5 & 1, emu.cpu.AF >> 4 & 1)))?
            .execute(Print("Next line: 'n'\n"))?
            .execute(Print("Remove Break: 'x'\n"))?
            .execute(Print("Jump in Mem: ':'\n"))?
            .execute(Print("Continue: any\n"))?;
        Ok(())
    }

    pub fn is_breakpoint(&mut self, pc: u16) -> bool {

        if self.break_next {
            self.break_next = false;
            return true;
        }

        match self.source_map.get(&pc) {
            Some(current_line_num) => {
                self.line_breakpoints.contains(&current_line_num) || self.pc_breakpoints.contains(&pc)
            },
            None => {
                self.pc_breakpoints.contains(&pc)
            }
        }
    }

    pub fn clear_term(&self) -> Result<()> {
        stdout()
            .execute(Clear(ClearType::All))?;
        
        Ok(())
    }

    pub fn block(&mut self, pc: u16, mem: &Mem, cpu: &Cpu) -> Result<()> {


        let event;
        if poll(Duration::from_millis(10))? {
            event = read()?;
        } else {
            self.is_blocked = true;
            return Ok(());
        }

        match event {
            Event::Key(KeyEvent { code, modifiers: _ }) => {
                match code {
                    Char('n') => self.break_next = true,
                    Char('x') => {
                        match self.source_map.get(&pc) {
                            Some(line) => {
                                self.line_breakpoints.retain(|x| x != line)
                            },
                            None => self.pc_breakpoints.retain(|x| x != &pc)
                        }
                    },
                    Char('c') => (),
                    Char(':') => {
                        // Jump to a position in memory
                        let mut values: [char; 3] = [' ',' ',' '];
                        let height = crossterm::terminal::size().unwrap_or( (32, 32) ).1;
                        stdout()
                            .execute(MoveTo(0, height))?
                            .execute(crossterm::cursor::EnableBlinking)?
                            .execute(Clear(ClearType::UntilNewLine))?
                            .execute(Print(":"))?;
                        for i in 0..3 {
                            if let Event::Key(KeyEvent {code, modifiers: _} ) = read()? {
                                values[i] = key_code_to_char(code);
                                stdout()
                                    .execute(MoveTo((i as u16) + 1, height))?
                                    .execute(Print(format!("{}",values[i])))?;
                            }
                        }
                        let string_val: String = values.iter().collect();
                        self.current_line = u16::from_str_radix(&string_val, 16)?;
                        self.current_line = if self.current_line > 0xFED {0xFED} else {self.current_line};
                        self.print_mem(mem)?;
                        return Ok(());
                    },
                    Char('+') => {
                        // add a breakpoint at a line
                        let mut values: [char; 4] = [' ',' ',' ',' '];
                        let height = crossterm::terminal::size().unwrap_or( (32, 32) ).1;
                        stdout()
                            .execute(MoveTo(0, height))?
                            .execute(crossterm::cursor::EnableBlinking)?
                            .execute(Clear(ClearType::UntilNewLine))?
                            .execute(Print("+"))?;
                        for i in 0..4 {
                            if let Event::Key(KeyEvent {code, modifiers: _} ) = read()? {
                                values[i] = key_code_to_char(code);
                                stdout()
                                    .execute(MoveTo((i as u16) + 1, height))?
                                    .execute(Print(format!("{}",values[i])))?;
                            }
                        }
                        let string_val: String = values.iter().collect();
                        self.add_breakpoint(usize::from_str_radix(&string_val, 16)? );
                        self.print_ins(cpu, mem)?;
                        self.print_mem(mem)?;
                        return Ok(());
                    },
                    KeyCode::Up => {
                        if self.current_line != 0 { self.current_line -= 1 };
                        self.print_mem(mem)?;
                        return Ok(());

                    },
                    KeyCode::Down => {
                        if self.current_line != 0xFED {self.current_line += 1 };
                        self.print_mem(mem)?;
                        return Ok(());
                    },
                    _ => return Ok(())
                }
            },
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        }
        self.is_blocked = false;
        Ok(())
    }

    pub fn load_assembly(&mut self, memory: &Mem, recompute: bool) {
        let source = group_byte(memory, recompute);
        self.source = source.0;
        self.source_map = source.1;
    }

    pub fn print_ins(&mut self, cpu: &Cpu, mem: &Mem) -> Result<()> {
        // get the source that the pc points too
        let pc = cpu.PC;
        let line_num: usize;

        match self.source_map.get(&pc) {
            Some(line) => {
                line_num = line.to_owned();
            },
            None => {
                // we may need to re-map
                self.load_assembly(mem, true);
                self.print_ins(cpu, mem).expect("Error printing ins");
                return Ok(());
            }

        }
        let line_offset= 10;
        let start = if (line_num as i32) - (line_offset as i32) >= 0 { line_num - line_offset } else { 0 };
        let end = if line_num + line_offset < self.source.len() { line_num + line_offset } else { self.source.len() };

        let x_offset = 19;
        stdout()
            .execute( MoveTo(x_offset, 0))?
            .execute(Print("Program"))?;

        for i in start..end {
            let pos = (i - start) + 2;
            let line = self.source.get(i).expect("???");
            if line_num == i {
                stdout()
                .execute(MoveTo(x_offset, pos as u16))?
                .execute(Clear(ClearType::UntilNewLine))?
                .execute(SetBackgroundColor(Color::DarkBlue))?
                .execute(Print(format!("{:04X} | {}", i, line)))?
                .execute(ResetColor)?;
            } else {
                stdout()
                .execute(MoveTo(x_offset, pos as u16))?
                .execute(Clear(ClearType::UntilNewLine))?
                .execute(Print(format!("{:04X} | {}", i, line)))?;
            }
        };

        Ok(())
    }

    pub fn print_mem(&self, mem: &Mem) -> Result<()> {

        let x_offset = 41;
        let size = crossterm::terminal::size().unwrap_or( (32, 32) );
        stdout()
            .execute(MoveTo(x_offset, 0))?
            .execute(Print("Memory"))?;

        for pos in 2..size.1 {

            let mem_loc = (self.current_line * 0x10).overflowing_add( (pos - 2) * 0x10 );
            if mem_loc.1 {
                stdout()
                .execute(MoveTo(x_offset, pos))?
                .execute(Clear(ClearType::UntilNewLine))?;
                continue;
            }
            let mem_loc = mem_loc.0;

            let mem_vals: [u8; 16] = [
                mem.get(mem_loc),
                mem.get(mem_loc + 1),
                mem.get(mem_loc + 2),
                mem.get(mem_loc + 3),
                mem.get(mem_loc + 4),
                mem.get(mem_loc + 5),
                mem.get(mem_loc + 6),
                mem.get(mem_loc + 7),
                mem.get(mem_loc + 8),
                mem.get(mem_loc + 9),
                mem.get(mem_loc + 10),
                mem.get(mem_loc + 11),
                mem.get(mem_loc + 12),
                mem.get(mem_loc + 13),
                mem.get(mem_loc + 14),
                mem.get(mem_loc + 15)
            ];

            let mut line: String = format!("{:04X} |", mem_loc);
            // build the string
            for i in 0..16 {
                line = format!("{} {:02X}", line, mem_vals[i as usize]);
            }

            stdout()
                .execute(MoveTo(x_offset, pos))?
                .execute(Print(line))?;
        }

        Ok(())
    }

    pub fn print_stat(&self, mem: &Mem) -> Result<()> {
        let x_offset = 95 + 5;
        stdout()
            .execute(MoveTo(x_offset, 0))?
            .execute(Print("LCD"))?

            .execute(MoveTo(x_offset, 2))?
            .execute(Print(format!("LY: {}", mem.get(0xFF44))))?
            .execute(MoveTo(x_offset, 3))?
            .execute(Print(format!("LYC: {}", mem.get(0xFF45))))?
            .execute(MoveTo(x_offset, 4))?
            .execute(Print(format!("SCY: {}", mem.get(0xFF42))))?
            .execute(MoveTo(x_offset, 5))?
            .execute(Print(format!("SCX: {}", mem.get(0xFF43))))?
            .execute(MoveTo(x_offset, 6))?
            .execute(Print(format!("WY: {}", mem.get(0xFF4A))))?
            .execute(MoveTo(x_offset, 7))?
            .execute(Print(format!("WX: {}", mem.get(0xFF4B))))?

            .execute(MoveTo(x_offset, 9))?
            .execute(Print("LCD Control"))?

            .execute(MoveTo(x_offset, 11))?
            .execute(Print(format!("On: {}", mem.get(0xFF40) >> 7 & 1 == 1)))?
            .execute(MoveTo(x_offset, 12))?
            .execute(Print(format!("Window Map: {:04X}", if mem.get(0xFF40) >> 6 & 1 == 1 {0x9C00} else {0x9800})))?
            .execute(MoveTo(x_offset, 13))?
            .execute(Print(format!("Window On: {}", mem.get(0xFF40) >> 5 & 1 == 1)))?
            .execute(MoveTo(x_offset, 14))?
            .execute(Print(format!("BG Window Tile Data: {:04X}", if mem.get(0xFF40) >> 4 & 1 == 1 {0x8800} else {0x8000})))?
            .execute(MoveTo(x_offset, 15))?
            .execute(Print(format!("BG Map: {:04X}", if mem.get(0xFF40) >> 3 & 1 == 1 {0x9C00} else {0x9800})))?
            .execute(MoveTo(x_offset, 16))?
            .execute(Print(format!("OBJ Size: {}", if mem.get(0xFF40) >> 2 & 1 == 1 {16} else {8})))?
            .execute(MoveTo(x_offset, 17))?
            .execute(Print(format!("OBJ Enabled: {}", mem.get(0xFF40) >> 1 & 1 == 1)))?
            .execute(MoveTo(x_offset, 18))?
            .execute(Print(format!("BG Window On: {}", mem.get(0xFF40) & 1 == 1)))?;

        Ok(())
    }

    pub fn enable_trace(&self) {
        if let Ok(file) = OpenOptions::new()
        .write(true)
        .open("trace.txt") {
            file.set_len(0).expect("Couldn't clear file"); 
        } else {
            File::create("trace.txt").expect("Couldn't create file");
        }
    }

    pub fn trace_to_file(&mut self, mem: &Mem, cpu: &Cpu) {

        if cpu.current_cycle != 1 {return}

        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open("trace.txt")
            .unwrap();

        let pc = cpu.get_word_reg(&cpu::Reg::PC);
        let line;
        
        match self.source_map.get(&pc) {
            Some(l) => {
                line = Some(l);
            },
            None => {
                // we may need to re-map
                self.load_assembly(mem, true);
                line = self.source_map.get(&pc);
            }
        }
        
        let ins = self.source.get(line.unwrap().to_owned()).expect("???");

        let _e = match writeln!(file, "{:04X} | {}", pc, ins) {
            Err(_) => eprintln!("Error couldn't append to trace"),
            Ok(_) => {}
        };
    }

}

fn group_byte(memory: &Mem, recompute: bool) -> (Vec<String>, HashMap<u16, usize>) {
    // create a byte group vector for all the data in memory
    let mut grouped_ops: Vec<String> = Vec::new();
    // points an op code to its source line
    let mut source_map = HashMap::new();
    let mut start = 0x0000;
    let mut end = 0x7FFF;

    if recompute { 
        // we only recompute to work memory because the rom doesn't change
        start = 0xC000;
        end = 0xDFFF;
    }

    let mut i = start; 

    while i < end {

        if i > 0x7FFF && i < 0xC000 {
            grouped_ops.push("???".to_owned());
            source_map.insert(i, 1);
            i += 1;
            continue;
        } 

        let op = memory.get(i);
        source_map.insert(i, grouped_ops.len());
        let length = op_byte_len(&op);
        if length == 1 {
            grouped_ops.push(op_to_string(op, 0, 0));
        } else if length == 2 {
            i += 1;
            source_map.insert(i, grouped_ops.len());
            let second_op = memory.get(i);
            grouped_ops.push(op_to_string(op, second_op, 0));
        } else {
            i += 1;
            source_map.insert(i, grouped_ops.len());
            let second_op = memory.get(i);
            i += 1;
            source_map.insert(i, grouped_ops.len());
            let third_op = memory.get(i);
            grouped_ops.push(op_to_string(op, second_op, third_op));
        }
        i += 1;
    }
    (grouped_ops, source_map)
}

fn op_byte_len(op: &u8) -> u8 {
    match op {
        // 2 byte op codes
        0x06 | 0x0E | 0x10 | 0x02 | 0x16 | 0x18 | 0x1E |
        0x20 | 0x26 | 0x28 | 0x2E | 0x30 | 0x38 | 0x3E |
        0xC6 | 0xCE | 0xD6 | 0xDE | 0xE0 | 0xE6 | 0xE8 | 
        0xEE | 0xF0 | 0xF6 | 0xF8 | 0xFE | 0xCB => 2,
        // 3 byte op codes
        0x01 | 0x08 | 0x11 | 0x21 | 0x31 | 0xC2 | 0xC3 |
        0xC4 | 0xCA | 0xCC | 0xCD | 0xD2 | 0xD4 | 0xDA | 
        0xDC | 0xEA | 0xF1 => 3, 
        // remainder are 1 byte
        _ => 1
    }
}

fn op_to_string(op: u8, op_2: u8, op_3: u8) -> String {
    match op {
        0x00 => "NOP".to_string(),
        0x01 => format!("LD BC, {:02X}{:02X}", op_3, op_2),
        0x02 => "LD (BC), A".to_string(),
        0x03 => "INC BC".to_string(),
        0x04 => "INC B".to_string(),
        0x05 => "DEC B".to_string(),
        0x06 => format!("LD B, {}", op_2),
        0x07 => "RLCA".to_string(),
        0x08 => format!("LD ({:02X}{:02X}), SP", op_3, op_2),
        0x09 => "ADD HL, BC".to_string(),
        0x0A => "LD A, (BC)".to_string(),
        0x0B => "DEC BC".to_string(),
        0x0C => "INC C".to_string(),
        0x0D => "DEC C".to_string(),
        0x0E => format!("LD C, {:02X}", op_2),
        0x0F => "RRCA".to_string(),

        0x10 => "STOP".to_string(),
        0x11 => format!("LD DE, {:02X}{:02X}", op_3, op_2),
        0x12 => "LD (DE), A".to_string(),
        0x13 => "INC DE".to_string(),
        0x14 => "INC D".to_string(),
        0x15 => "DEC D".to_string(),
        0x16 => format!("LD D, {}", op_2),
        0x17 => "RLA".to_string(),
        0x18 => format!("JR {}", op_2 as i8),
        0x19 => "ADD HL, DE".to_string(),
        0x1A => "LD A, (DE)".to_string(),
        0x1B => "DEC DE".to_string(),
        0x1C => "INC E".to_string(),
        0x1D => "DEC E".to_string(),
        0x1E => format!("LD E, {}", op_2),
        0x1F => "RRA".to_string(),

        0x20 => format!("JR NZ, {}", op_2 as i8),
        0x21 => format!("LD HL, {:02X}{:02X}", op_3, op_2),
        0x22 => "LD (HL+), A".to_string(),
        0x23 => "INC HL".to_string(),
        0x24 => "INC H".to_string(),
        0x25 => "DEC H".to_string(),
        0x26 => format!("LD H, {}", op_2),
        0x27 => "DAA".to_string(),
        0x28 => format!("JR Z {}", op_2 as i8),
        0x29 => "ADD HL, HL".to_string(),
        0x2A => "LD A, (HL+)".to_string(),
        0x2B => "DEC HL".to_string(),
        0x2C => "INC L".to_string(),
        0x2D => "DEC L".to_string(),
        0x2E => format!("LD L, {}", op_2),
        0x2F => "CPL".to_string(),

        0x30 => format!("JR NC, {}", op_2 as i8),
        0x31 => format!("LD SP, {:02X}{:02X}", op_3, op_2),
        0x32 => "LD (HL-), A".to_string(),
        0x33 => "INC SP".to_string(),
        0x34 => "INC (HL)".to_string(),
        0x35 => "DEC (HL)".to_string(),
        0x36 => format!("LD (HL), {}", op_2),
        0x37 => "SCF".to_string(),
        0x38 => format!("JR C {}", op_2 as i8),
        0x39 => "ADD HL, SP".to_string(),
        0x3A => "LD A, (HL-)".to_string(),
        0x3B => "DEC SP".to_string(),
        0x3C => "INC A".to_string(),
        0x3D => "DEC A".to_string(),
        0x3E => format!("LD A, {}", op_2),
        0x3F => "CCF".to_string(),

        0x40 => "LD B, B".to_string(),
        0x41 => "LD B, C".to_string(),
        0x42 => "LD B, D".to_string(),
        0x43 => "LD B, E".to_string(),
        0x44 => "LD B, H".to_string(),
        0x45 => "LD B, L".to_string(),
        0x46 => "LD B, (HL)".to_string(),
        0x47 => "LD B, A".to_string(),
        0x48 => "LD C, B".to_string(),
        0x49 => "LD C, C".to_string(),
        0x4A => "LD C, D".to_string(),
        0x4B => "LD C, E".to_string(),
        0x4C => "LD C, H".to_string(),
        0x4D => "LD C, L".to_string(),
        0x4E => "LD C, (HL)".to_string(),
        0x4F => "LD C, A".to_string(),

        0x50 => "LD D, B".to_string(),
        0x51 => "LD D, C".to_string(),
        0x52 => "LD D, D".to_string(),
        0x53 => "LD D, E".to_string(),
        0x54 => "LD D, H".to_string(),
        0x55 => "LD D, L".to_string(),
        0x56 => "LD D, (HL)".to_string(),
        0x57 => "LD D, A".to_string(),
        0x58 => "LD E, B".to_string(),
        0x59 => "LD E, C".to_string(),
        0x5A => "LD E, D".to_string(),
        0x5B => "LD E, E".to_string(),
        0x5C => "LD E, H".to_string(),
        0x5D => "LD E, L".to_string(),
        0x5E => "LD E, (HL)".to_string(),
        0x5F => "LD E, A".to_string(),

        0x60 => "LD H, B".to_string(),
        0x61 => "LD H, C".to_string(),
        0x62 => "LD H, D".to_string(),
        0x63 => "LD H, E".to_string(),
        0x64 => "LD H, H".to_string(),
        0x65 => "LD H, L".to_string(),
        0x66 => "LD H, (HL)".to_string(),
        0x67 => "LD H, A".to_string(),
        0x68 => "LD L, B".to_string(),
        0x69 => "LD L, C".to_string(),
        0x6A => "LD L, D".to_string(),
        0x6B => "LD L, E".to_string(),
        0x6C => "LD L, H".to_string(),
        0x6D => "LD L, L".to_string(),
        0x6E => "LD L, (HL)".to_string(),
        0x6F => "LD L, A".to_string(),

        0x70 => "LD (HL), B".to_string(),
        0x71 => "LD (HL), C".to_string(),
        0x72 => "LD (HL), D".to_string(),
        0x73 => "LD (HL), E".to_string(),
        0x74 => "LD (HL), H".to_string(),
        0x75 => "LD (HL), L".to_string(),
        0x76 => "LD (HL), (HL)".to_string(),
        0x77 => "LD (HL), A".to_string(),
        0x78 => "LD A, B".to_string(),
        0x79 => "LD A, C".to_string(),
        0x7A => "LD A, D".to_string(),
        0x7B => "LD A, E".to_string(),
        0x7C => "LD A, H".to_string(),
        0x7D => "LD A, L".to_string(),
        0x7E => "LD A, (HL)".to_string(),
        0x7F => "LD A, A".to_string(),

        0x80 => "ADD A, B".to_string(),
        0x81 => "ADD A, C".to_string(),
        0x82 => "ADD A, D".to_string(),
        0x83 => "ADD A, E".to_string(),
        0x84 => "ADD A, H".to_string(),
        0x85 => "ADD A, L".to_string(),
        0x86 => "ADD A, HL".to_string(),
        0x87 => "ADD A, A".to_string(),
        0x88 => "ADC A, B".to_string(),
        0x89 => "ADC A, C".to_string(),
        0x8A => "ADC A, D".to_string(),
        0x8B => "ADC A, E".to_string(),
        0x8C => "ADC A, H".to_string(),
        0x8D => "ADC A, L".to_string(),
        0x8E => "ADC A, HL".to_string(),
        0x8F => "ADC A, A".to_string(),

        0x90 => "SUB A, B".to_string(),
        0x91 => "SUB A, C".to_string(),
        0x92 => "SUB A, D".to_string(),
        0x93 => "SUB A, E".to_string(),
        0x94 => "SUB A, H".to_string(),
        0x95 => "SUB A, L".to_string(),
        0x96 => "SUB A, HL".to_string(),
        0x97 => "SUB A, A".to_string(),
        0x98 => "SBC A, B".to_string(),
        0x99 => "SBC A, C".to_string(),
        0x9A => "SBC A, D".to_string(),
        0x9B => "SBC A, E".to_string(),
        0x9C => "SBC A, H".to_string(),
        0x9D => "SBC A, L".to_string(),
        0x9E => "SBC A, HL".to_string(),
        0x9F => "SBC A, A".to_string(),

        0xA0 => "AND A, B".to_string(),
        0xA1 => "AND A, C".to_string(),
        0xA2 => "AND A, D".to_string(),
        0xA3 => "AND A, E".to_string(),
        0xA4 => "AND A, H".to_string(),
        0xA5 => "AND A, L".to_string(),
        0xA6 => "AND A, HL".to_string(),
        0xA7 => "AND A, A".to_string(),
        0xA8 => "XOR A, B".to_string(),
        0xA9 => "XOR A, C".to_string(),
        0xAA => "XOR A, D".to_string(),
        0xAB => "XOR A, E".to_string(),
        0xAC => "XOR A, H".to_string(),
        0xAD => "XOR A, L".to_string(),
        0xAE => "XOR A, HL".to_string(),
        0xAF => "XOR A, A".to_string(),

        0xB0 => "OR A, B".to_string(),
        0xB1 => "OR A, C".to_string(),
        0xB2 => "OR A, D".to_string(),
        0xB3 => "OR A, E".to_string(),
        0xB4 => "OR A, H".to_string(),
        0xB5 => "OR A, L".to_string(),
        0xB6 => "OR A, HL".to_string(),
        0xB7 => "OR A, A".to_string(),
        0xB8 => "CP A, B".to_string(),
        0xB9 => "CP A, C".to_string(),
        0xBA => "CP A, D".to_string(),
        0xBB => "CP A, E".to_string(),
        0xBC => "CP A, H".to_string(),
        0xBD => "CP A, L".to_string(),
        0xBE => "CP A, HL".to_string(),
        0xBF => "CP A, A".to_string(),

        0xC0 => "RET NZ".to_string(),
        0xC1 => "POP BC".to_string(),
        0xC2 => format!("JP NZ, {:02X}{:02X}", op_3, op_2),
        0xC3 => format!("JP {:02X}{:02X}", op_3, op_2),
        0xC4 => format!("CALL NZ {:02X}{:02X}", op_3, op_2),
        0xC5 => "PUSH BC".to_string(),
        0xC6 => format!("ADD A, {}", op_2),
        0xC7 => "RST 0".to_string(),
        0xC8 => "RET Z".to_string(),
        0xC9 => "RET".to_string(),
        0xCA => format!("JP Z, {:02X}{:02X}", op_3, op_2),
        0xCB => match op_2 {
            0x00 => "RLC B".to_string(),
            0x01 => "RLC C".to_string(),
            0x02 => "RLC D".to_string(),
            0x03 => "RLC E".to_string(),
            0x04 => "RLC H".to_string(),
            0x05 => "RLC L".to_string(),
            0x06 => "RLC (HL)".to_string(),
            0x07 => "RLC A".to_string(),
            0x08 => "RRC B".to_string(),
            0x09 => "RRC C".to_string(),
            0x0A => "RRC D".to_string(),
            0x0B => "RRC E".to_string(),
            0x0C => "RRC H".to_string(),
            0x0D => "RRC L".to_string(),
            0x0E => "RRC (HL)".to_string(),
            0x0F => "RRC A".to_string(),

            0x10 => "RL B".to_string(),
            0x11 => "RL C".to_string(),
            0x12 => "RL D".to_string(),
            0x13 => "RL E".to_string(),
            0x14 => "RL H".to_string(),
            0x15 => "RL L".to_string(),
            0x16 => "RL (HL)".to_string(),
            0x17 => "RL A".to_string(),
            0x18 => "RR B".to_string(),
            0x19 => "RR C".to_string(),
            0x1A => "RR D".to_string(),
            0x1B => "RR E".to_string(),
            0x1C => "RR H".to_string(),
            0x1D => "RR L".to_string(),
            0x1E => "RR (HL)".to_string(),
            0x1F => "RR A".to_string(),

            0x20 => "SLA B".to_string(),
            0x21 => "SLA C".to_string(),
            0x22 => "SLA D".to_string(),
            0x23 => "SLA E".to_string(),
            0x24 => "SLA H".to_string(),
            0x25 => "SLA L".to_string(),
            0x26 => "SLA (HL)".to_string(),
            0x27 => "SLA A".to_string(),
            0x28 => "SRA B".to_string(),
            0x29 => "SRA C".to_string(),
            0x2A => "SRA D".to_string(),
            0x2B => "SRA E".to_string(),
            0x2C => "SRA H".to_string(),
            0x2D => "SRA L".to_string(),
            0x2E => "SRA (HL)".to_string(),
            0x2F => "SRA A".to_string(),

            0x30 => "SWAP B".to_string(),
            0x31 => "SWAP C".to_string(),
            0x32 => "SWAP D".to_string(),
            0x33 => "SWAP E".to_string(),
            0x34 => "SWAP H".to_string(),
            0x35 => "SWAP L".to_string(),
            0x36 => "SWAP (HL)".to_string(),
            0x37 => "SWAP A".to_string(),
            0x38 => "SRL B".to_string(),
            0x39 => "SRL C".to_string(),
            0x3A => "SRL D".to_string(),
            0x3B => "SRL E".to_string(),
            0x3C => "SRL H".to_string(),
            0x3D => "SRL L".to_string(),
            0x3E => "SRL (HL)".to_string(),
            0x3F => "SRL A".to_string(),

            0x40..= 0xFF => {
                let low = op_2 & 0xF;
                let high = op_2 >> 4 & 0xF;

                let reg = match low {
                    0 | 8=> "B",
                    1 | 9 => "C",
                    2 | 0xA => "D",
                    3 | 0xB => "E",
                    4 | 0xC => "H",
                    5 | 0xD => "L",
                    6 | 0xE => "(HL)",
                    7 | 0xF => "A",
                    _ => "??"
                };

                let bit =  match high {
                    4 | 8   | 0xC => if low < 8 {0} else {1},
                    5 | 9   | 0xD => if low < 8 {2} else {3},
                    6 | 0xA | 0xE => if low < 8 {4} else {5},
                    7 | 0xB | 0xF => if low < 8 {6} else {7},
                    _ => 9
                };

                let ins = match high {
                    4 ..= 7 => "BIT",
                    8 ..= 0xB => "RES",
                    0xC ..= 0xF => "SET",
                    _ => "???"
                };

                format!("{} {}, {}", ins, bit, reg)
            },
        },
        0xCC => format!("CALL Z, {:02X}{:02X}", op_3, op_2),
        0xCD => format!("CALL {:02X}{:02X}", op_3, op_2),
        0xCE => format!("ADC A {:02X}", op_2),
        0xCF => "RST 1".to_string(),

        0xD0 => "RET NC".to_string(),
        0xD1 => "POP DE".to_string(),
        0xD2 => format!("JP NC, {:02X}{:02X}", op_3, op_2),
        0xD4 => format!("CALL NC {:02X}{:02X}", op_3, op_2),
        0xD5 => "PUSH DE".to_string(),
        0xD6 => format!("SUB A, {}", op_2),
        0xD7 => "RST 2".to_string(),
        0xD8 => "RET C".to_string(),
        0xD9 => "RETI".to_string(),
        0xDA => format!("JP C, {:02X}{:02X}", op_3, op_2),
        0xDC => format!("CALL C, {:02X}{:02X}", op_3, op_2),
        0xDE => format!("SBC A {:02X}", op_2),
        0xDF => "RST 3".to_string(),

        0xE0 => format!("LD (FF{:02X}), A", op_2),
        0xE1 => "POP HL".to_string(),
        0xE2 => "LD (C), A".to_string(),
        0xE5 => "PUSH HL".to_string(),
        0xE6 => format!("AND A, {}", op_2),
        0xE7 => "RST 4".to_string(),
        0xE8 => format!("ADD SP, {}", op_2),
        0xE9 => "JP HL".to_string(),
        0xEA => format!("LD ({:02X}{:02X}), A", op_3, op_2),
        0xEE => format!("XOR A {:02X}", op_2),
        0xEF => "RST 5".to_string(),

        0xF0 => format!("LD A, (FF{:02X})", op_2),
        0xF1 => "POP AF".to_string(),
        0xF2 => "LD A, (C)".to_string(),
        0xF3 => "DI".to_string(),
        0xF5 => "PUSH AF".to_string(),
        0xF6 => format!("OR A, {}", op_2),
        0xF7 => "RST 6".to_string(),
        0xF8 => format!("LD HL SP+{}", op_2 as i8),
        0xF9 => "LD SP HL".to_string(),
        0xFA => format!("LD A, ({:02X}{:02X})", op_3, op_2),
        0xFB => "EI".to_string(),
        0xFE => format!("CP A {:02X}", op_2),
        0xFF => "RST 7".to_string(),
        _ => "UNKOWN".to_string()
    }
}

fn key_code_to_char(key: KeyCode) -> char {
    match key {
        Char('0') => '0',
        Char('1') => '1',
        Char('2') => '2',
        Char('3') => '3',
        Char('4') => '4',
        Char('5') => '5',
        Char('6') => '6',
        Char('7') => '7',
        Char('8') => '8',
        Char('9') => '9',
        Char('A') | Char('a') => 'A',
        Char('B') | Char('b') => 'B',
        Char('C') | Char('c') => 'C',
        Char('D') | Char('d') => 'D',
        Char('E') | Char('e') => 'E',
        Char('F') | Char('f') => 'F',
        _ => '0'
    }
}