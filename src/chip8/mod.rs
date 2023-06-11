use std::fmt::Display;

use crossbeam_channel::{Receiver, Sender};
use rand::{rngs::ThreadRng, Rng};

use self::{
    display::{Chip8Display, DisplayInstruction},
    keypad::{Event, Keypad},
    memory::{Memory, PROGRAM_START},
    registers::Registers,
    settings::Settings,
    stack::Stack,
};

pub mod display;
pub mod keypad;
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
    rng: ThreadRng,
    keypad: Keypad,
}

impl Chip8 {
    pub fn new(
        settings: Settings,
        program: &[u8],
        sender: Sender<DisplayInstruction>,
        receiver: Receiver<Event>,
    ) -> Self {
        let memory = Memory::new(program);
        let display = Chip8Display::new(sender);
        let stack = Stack::new();
        let registers = Registers::new();
        let rng = rand::thread_rng();
        let keypad = Keypad::new(receiver);

        Self {
            settings,
            memory,
            display,
            stack,
            registers,
            program_counter: PROGRAM_START,
            index_register: 0,
            rng,
            keypad,
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
            0x0 if instruction.nnn() == 0x0E0 => self.clear_display(),
            0x6 => self.set_value(instruction.x(), instruction.nn()),
            0xA => self.set_index(instruction.nnn()),
            0xD => self.display(instruction.x(), instruction.y(), instruction.n()),
            0x1 => self.jump(instruction.nnn()),
            0x7 => self.add_value(instruction.x(), instruction.nn()),
            0x3 => self.skip_if_equals_value(instruction.x(), instruction.nn()),
            0x4 => self.skip_if_not_equals_value(instruction.x(), instruction.nn()),
            0x5 => self.skip_if_equals_register(instruction.x(), instruction.y()),
            0x9 => self.skip_if_not_equals_register(instruction.x(), instruction.y()),
            0x2 => self.call_subroutine(instruction.nnn()),
            0x0 if instruction.nnn() == 0x0EE => self.return_subroutine(),
            0x8 if instruction.n() == 0x0 => self.set_register(instruction.x(), instruction.y()),
            0x8 if instruction.n() == 0x1 => self.or_register(instruction.x(), instruction.y()),
            0x8 if instruction.n() == 0x2 => self.and_register(instruction.x(), instruction.y()),
            0x8 if instruction.n() == 0x3 => self.xor_register(instruction.x(), instruction.y()),
            0x8 if instruction.n() == 0x4 => self.add_register(instruction.x(), instruction.y()),
            0x8 if instruction.n() == 0x5 => self.sub_register_xy(instruction.x(), instruction.y()),
            0x8 if instruction.n() == 0x7 => self.sub_register_yx(instruction.x(), instruction.y()),
            0x8 if instruction.n() == 0x6 => self.shift_right(instruction.x(), instruction.y()),
            0x8 if instruction.n() == 0xE => self.shift_left(instruction.x(), instruction.y()),
            0xF if instruction.nn() == 0x55 => self.store_registers(instruction.x()),
            0xF if instruction.nn() == 0x65 => self.load_registers(instruction.x()),
            0xF if instruction.nn() == 0x33 => self.binary_coded_decimal(instruction.x()),
            0xF if instruction.nn() == 0x1E => self.add_to_index(instruction.x()),
            0xC => self.random(instruction.x(), instruction.nn()),
            _ => panic!("Unknown instruction {}", instruction),
        }
    }

    fn clear_display(&mut self) {
        self.display.clear();
    }

    fn set_value(&mut self, register_number: u8, value: u8) {
        self.registers.set_value(register_number, value);
    }

    fn set_index(&mut self, value: u16) {
        self.index_register = value;
    }

    fn display(&mut self, x_register: u8, y_register: u8, sprite_height: u8) {
        let x_start = self.registers.get_value(x_register) % 64;
        let y_start = self.registers.get_value(y_register) % 32;

        let mut flags_value = false;

        for row in 0..sprite_height {
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
                flags_value |= self.display.set(x as usize, y as usize);
            }
        }

        self.registers.set_value(0xF, flags_value as u8);
    }

    fn jump(&mut self, address: u16) {
        self.program_counter = address;
    }

    fn add_value(&mut self, register_number: u8, value: u8) {
        let existing = self.registers.get_value(register_number);
        let new = existing.wrapping_add(value);
        self.registers.set_value(register_number, new);
    }

    fn skip_if_equals_value(&mut self, register_number: u8, value: u8) {
        let x_value = self.registers.get_value(register_number);
        if x_value == value {
            self.program_counter += 2;
        }
    }

    fn skip_if_not_equals_value(&mut self, register_number: u8, value: u8) {
        let x_value = self.registers.get_value(register_number);
        if x_value != value {
            self.program_counter += 2;
        }
    }

    fn skip_if_equals_register(&mut self, register_number_x: u8, register_number_y: u8) {
        let x_value = self.registers.get_value(register_number_x);
        let y_value = self.registers.get_value(register_number_y);
        if x_value == y_value {
            self.program_counter += 2;
        }
    }

    fn skip_if_not_equals_register(&mut self, register_number_x: u8, register_number_y: u8) {
        let x_value = self.registers.get_value(register_number_x);
        let y_value = self.registers.get_value(register_number_y);
        if x_value != y_value {
            self.program_counter += 2;
        }
    }

    fn call_subroutine(&mut self, address: u16) {
        self.stack.push(self.program_counter);
        self.program_counter = address;
    }

    fn return_subroutine(&mut self) {
        let address = self.stack.pop().unwrap();
        self.program_counter = address;
    }

    fn set_register(&mut self, register_number_x: u8, register_number_y: u8) {
        let y_value = self.registers.get_value(register_number_y);
        self.registers.set_value(register_number_x, y_value);
    }

    fn or_register(&mut self, register_number_x: u8, register_number_y: u8) {
        let x_value = self.registers.get_value(register_number_x);
        let y_value = self.registers.get_value(register_number_y);
        let value = x_value | y_value;
        self.registers.set_value(register_number_x, value);
    }

    fn and_register(&mut self, register_number_x: u8, register_number_y: u8) {
        let x_value = self.registers.get_value(register_number_x);
        let y_value = self.registers.get_value(register_number_y);
        let value = x_value & y_value;
        self.registers.set_value(register_number_x, value);
    }

    fn xor_register(&mut self, register_number_x: u8, register_number_y: u8) {
        let x_value = self.registers.get_value(register_number_x);
        let y_value = self.registers.get_value(register_number_y);
        let value = x_value ^ y_value;
        self.registers.set_value(register_number_x, value);
    }

    fn add_register(&mut self, register_number_x: u8, register_number_y: u8) {
        let x_value = self.registers.get_value(register_number_x);
        let y_value = self.registers.get_value(register_number_y);
        let (value, overflowed) = x_value.overflowing_add(y_value);
        self.registers.set_value(register_number_x, value);
        self.registers.set_value(0xF, overflowed as u8);
    }

    fn sub_register_xy(&mut self, register_number_x: u8, register_number_y: u8) {
        let x_value = self.registers.get_value(register_number_x);
        let y_value = self.registers.get_value(register_number_y);
        let (value, overflowed) = x_value.overflowing_sub(y_value);
        self.registers.set_value(register_number_x, value);
        self.registers.set_value(0xf, !overflowed as u8);
    }

    fn sub_register_yx(&mut self, register_number_x: u8, register_number_y: u8) {
        let x_value = self.registers.get_value(register_number_x);
        let y_value = self.registers.get_value(register_number_y);
        let (value, overflowed) = y_value.overflowing_sub(x_value);
        self.registers.set_value(register_number_x, value);
        self.registers.set_value(0xf, !overflowed as u8);
    }

    fn shift_right(&mut self, register_number_x: u8, register_number_y: u8) {
        if self.settings.assign_shift {
            let y_value = self.registers.get_value(register_number_y);
            self.registers.set_value(register_number_x, y_value);
        }
        let x_value = self.registers.get_value(register_number_x);
        let flags_value = x_value & 0x1 != 0;
        let shifted = x_value >> 1;
        self.registers.set_value(register_number_x, shifted);
        self.registers.set_value(0xf, flags_value as u8);
    }

    fn shift_left(&mut self, register_number_x: u8, register_number_y: u8) {
        if self.settings.assign_shift {
            let y_value = self.registers.get_value(register_number_y);
            self.registers.set_value(register_number_x, y_value);
        }
        let x_value = self.registers.get_value(register_number_x);
        let flags_value = x_value & 0x80 != 0;
        let shifted = x_value << 1;
        self.registers.set_value(register_number_x, shifted);
        self.registers.set_value(0xf, flags_value as u8);
    }

    fn store_registers(&mut self, register_number: u8) {
        for register in 0..=register_number {
            let address = self.index_register + register as u16;
            let x_value = self.registers.get_value(register);
            self.memory.set_u8(address, x_value);
        }
        if self.settings.load_store_increment {
            self.index_register += register_number as u16 + 1;
        }
    }

    fn load_registers(&mut self, register_number: u8) {
        for register in 0..=register_number {
            let address = self.index_register + register as u16;
            let memory_value = self.memory.get_u8(address);
            self.registers.set_value(register, memory_value);
        }
        if self.settings.load_store_increment {
            self.index_register += register_number as u16 + 1;
        }
    }

    fn binary_coded_decimal(&mut self, register_number: u8) {
        let x_value = self.registers.get_value(register_number);
        let first = x_value / 100;
        let second = (x_value / 10) % 10;
        let third = x_value % 10;
        self.memory.set_u8(self.index_register, first);
        self.memory.set_u8(self.index_register + 1, second);
        self.memory.set_u8(self.index_register + 2, third);
    }

    fn add_to_index(&mut self, register_number: u8) {
        let x_value = self.registers.get_value(register_number);
        self.index_register += x_value as u16;
        if self.settings.add_to_index_overflow {
            let overflowed = self.index_register > 0x0FFF;
            self.registers.set_value(0xF, overflowed as u8);
        }
    }

    fn random(&mut self, register_number: u8, mask: u8) {
        let random_number: u8 = self.rng.gen();
        let result = random_number & mask;
        self.registers.set_value(register_number, result);
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
