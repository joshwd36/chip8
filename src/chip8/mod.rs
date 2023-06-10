use std::fmt::Display;

use crossbeam_channel::Sender;

use self::{
    display::{Chip8Display, DisplayInstruction},
    memory::{Memory, PROGRAM_START},
    registers::Registers,
    stack::Stack,
};

pub mod display;
mod memory;
mod registers;
mod stack;

pub struct Chip8 {
    memory: Memory,
    display: Chip8Display,
    stack: Stack,
    registers: Registers,
    program_counter: u16,
    index_register: u16,
}

impl Chip8 {
    pub fn new(program: &[u8], sender: Sender<DisplayInstruction>) -> Self {
        let memory = Memory::new(program);
        let display = Chip8Display::new(sender);
        let stack = Stack::new();
        let registers = Registers::new();

        Self {
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
