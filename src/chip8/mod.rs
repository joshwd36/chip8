use std::fmt::Display;

use crossbeam_channel::Sender;

use self::{
    display::{Chip8Display, DisplayInstruction},
    memory::{Memory, PROGRAM_START},
    registers::Registers,
    settings::Settings,
    stack::Stack,
};

pub mod display;
mod memory;
mod registers;
pub mod settings;
mod stack;

pub struct Chip8 {
    settings: Settings,
    memory: Memory,
    display: Chip8Display,
    stack: Stack,
    registers: Registers,
    program_counter: u16,
    index_register: u16,
}

impl Chip8 {
    pub fn new(settings: Settings, program: &[u8], sender: Sender<DisplayInstruction>) -> Self {
        let memory = Memory::new(program);
        let display = Chip8Display::new(sender);
        let stack = Stack::new();
        let registers = Registers::new();

        Self {
            settings,
            memory,
            display,
            stack,
            registers,
            program_counter: PROGRAM_START,
            index_register: 0,
        }
    }

    pub fn run(&mut self) {
        loop {
            let instruction = self.fetch();
            self.execute(instruction);
        }
    }

    fn fetch(&mut self) -> Instruction {
        let instruction = self.memory.get_u16(self.program_counter);
        self.program_counter += 2;
        Instruction::new(instruction)
    }

    fn execute(&mut self, instruction: Instruction) {
        let first = instruction.first();
        match first {
            0x0 if instruction.nnn() == 0x0E0 => {
                self.display.clear();
            }
            0x6 => {
                let x = instruction.x();
                let nn = instruction.nn();
                self.registers.set_value(x, nn);
            }
            0xA => {
                let nnn = instruction.nnn();
                self.index_register = nnn;
            }
            0xD => {
                let x = instruction.x();
                let y = instruction.y();
                let n = instruction.n();

                let x_start = self.registers.get_value(x) % 64;
                let y_start = self.registers.get_value(y) % 32;

                let mut vf = false;

                for row in 0..n {
                    let y = y_start + row;
                    if y >= 32 {
                        break;
                    }
                    let sprite_data = self.memory.get_u8(self.index_register + (row as u16));
                    for x_offset in (BitIterator { num: sprite_data }) {
                        let x = x_start + x_offset;
                        if x >= 64 {
                            break;
                        }
                        vf |= self.display.set(x as usize, y as usize);
                    }
                }

                self.registers.set_value(0xF, vf as u8);
            }
            0x1 => {
                let nnn = instruction.nnn();
                self.program_counter = nnn;
            }
            0x7 => {
                let x = instruction.x();
                let nn = instruction.nn();
                let existing = self.registers.get_value(x);
                let new = existing.wrapping_add(nn);
                self.registers.set_value(x, new);
            }
            0x3 => {
                let x = instruction.x();
                let nn = instruction.nn();
                let vx = self.registers.get_value(x);
                if vx == nn {
                    self.program_counter += 2;
                }
            }
            0x4 => {
                let x = instruction.x();
                let nn = instruction.nn();
                let vx = self.registers.get_value(x);
                if vx != nn {
                    self.program_counter += 2;
                }
            }
            0x5 => {
                let x = instruction.x();
                let y = instruction.y();
                let vx = self.registers.get_value(x);
                let vy = self.registers.get_value(y);
                if vx == vy {
                    self.program_counter += 2;
                }
            }
            0x9 => {
                let x = instruction.x();
                let y = instruction.y();
                let vx = self.registers.get_value(x);
                let vy = self.registers.get_value(y);
                if vx != vy {
                    self.program_counter += 2;
                }
            }
            0x2 => {
                let nnn = instruction.nnn();
                self.stack.push(self.program_counter);
                self.program_counter = nnn;
            }
            0x0 if instruction.nnn() == 0x0EE => {
                let address = self.stack.pop().unwrap();
                self.program_counter = address;
            }
            0x8 if instruction.n() == 0x0 => {
                let x = instruction.x();
                let y = instruction.y();
                let vy = self.registers.get_value(y);
                self.registers.set_value(x, vy);
            }
            0x8 if instruction.n() == 0x1 => {
                let x = instruction.x();
                let y = instruction.y();
                let vx = self.registers.get_value(x);
                let vy = self.registers.get_value(y);
                let value = vx | vy;
                self.registers.set_value(x, value);
            }
            0x8 if instruction.n() == 0x2 => {
                let x = instruction.x();
                let y = instruction.y();
                let vx = self.registers.get_value(x);
                let vy = self.registers.get_value(y);
                let value = vx & vy;
                self.registers.set_value(x, value);
            }
            0x8 if instruction.n() == 0x3 => {
                let x = instruction.x();
                let y = instruction.y();
                let vx = self.registers.get_value(x);
                let vy = self.registers.get_value(y);
                let value = vx ^ vy;
                self.registers.set_value(x, value);
            }
            0x8 if instruction.n() == 0x4 => {
                let x = instruction.x();
                let y = instruction.y();
                let vx = self.registers.get_value(x);
                let vy = self.registers.get_value(y);
                let (value, overflowed) = vx.overflowing_add(vy);
                self.registers.set_value(x, value);
                self.registers.set_value(0xF, overflowed as u8);
            }
            0x8 if instruction.n() == 0x5 => {
                let x = instruction.x();
                let y = instruction.y();
                let vx = self.registers.get_value(x);
                let vy = self.registers.get_value(y);
                let (value, overflowed) = vx.overflowing_sub(vy);
                self.registers.set_value(x, value);
                self.registers.set_value(0xf, !overflowed as u8);
            }
            0x8 if instruction.n() == 0x7 => {
                let x = instruction.x();
                let y = instruction.y();
                let vx = self.registers.get_value(x);
                let vy = self.registers.get_value(y);
                let (value, overflowed) = vy.overflowing_sub(vx);
                self.registers.set_value(x, value);
                self.registers.set_value(0xf, !overflowed as u8);
            }
            0x8 if instruction.n() == 0x6 => {
                let x = instruction.x();
                let y = instruction.y();
                let vy = self.registers.get_value(y);
                if self.settings.assign_shift {
                    self.registers.set_value(x, vy);
                }
                let vx = self.registers.get_value(x);
                let vf = vx & 0x1 != 0;
                let shifted = vx >> 1;
                self.registers.set_value(x, shifted);
                self.registers.set_value(0xf, vf as u8);
            }
            0x8 if instruction.n() == 0xE => {
                let x = instruction.x();
                let y = instruction.y();
                let vy = self.registers.get_value(y);
                if self.settings.assign_shift {
                    self.registers.set_value(x, vy);
                }
                let vx = self.registers.get_value(x);
                let vf = vx & 0x80 != 0;
                let shifted = vx << 1;
                self.registers.set_value(x, shifted);
                self.registers.set_value(0xf, vf as u8);
            }
            0xF if instruction.nn() == 0x55 => {
                let x = instruction.x();
                for register in 0..=x {
                    let address = self.index_register + register as u16;
                    let vx = self.registers.get_value(register);
                    self.memory.set_u8(address, vx);
                }
                if self.settings.load_store_increment {
                    self.index_register += x as u16 + 1;
                }
            }
            0xF if instruction.nn() == 0x65 => {
                let x = instruction.x();
                for register in 0..=x {
                    let address = self.index_register + register as u16;
                    let memory_value = self.memory.get_u8(address);
                    self.registers.set_value(register, memory_value);
                }
                if self.settings.load_store_increment {
                    self.index_register += x as u16 + 1;
                }
            }
            0xF if instruction.nn() == 0x33 => {
                let x = instruction.x();
                let vx = self.registers.get_value(x);
                let first = vx / 100;
                let second = (vx / 10) % 10;
                let third = vx % 10;
                self.memory.set_u8(self.index_register, first);
                self.memory.set_u8(self.index_register + 1, second);
                self.memory.set_u8(self.index_register + 2, third);
            }
            0xF if instruction.nn() == 0x1E => {
                let x = instruction.x();
                let vx = self.registers.get_value(x);
                self.index_register += vx as u16;
                if self.settings.add_to_index_overflow {
                    let overflowed = self.index_register > 0x0FFF;
                    self.registers.set_value(0xF, overflowed as u8);
                }
            }
            _ => panic!("Unknown instruction {}", instruction),
        }
    }
}

struct Instruction {
    value: u16,
}

impl Instruction {
    pub fn new(value: u16) -> Self {
        Self { value }
    }

    pub fn first(&self) -> u8 {
        ((self.value >> 12) & 0b0000_0000_0000_1111) as u8
    }

    pub fn x(&self) -> u8 {
        ((self.value >> 8) & 0b0000_0000_0000_1111) as u8
    }

    pub fn y(&self) -> u8 {
        ((self.value >> 4) & 0b0000_0000_0000_1111) as u8
    }

    pub fn n(&self) -> u8 {
        (self.value & 0b0000_0000_0000_1111) as u8
    }

    pub fn nn(&self) -> u8 {
        (self.value & 0b0000_0000_1111_1111) as u8
    }

    pub fn nnn(&self) -> u16 {
        self.value & 0b0000_1111_1111_1111
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#06x}", self.value)
    }
}

struct BitIter {
    num: u8,
}

impl IntoIterator for BitIter {
    type Item = u8;

    type IntoIter = BitIterator;

    fn into_iter(self) -> Self::IntoIter {
        BitIterator { num: self.num }
    }
}

struct BitIterator {
    num: u8,
}

impl Iterator for BitIterator {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        let value = self.num.leading_zeros();
        if value < 8 {
            let mask = 0x80 >> value;
            self.num = self.num ^ (mask as u8);
            Some(value as u8)
        } else {
            None
        }
    }
}
