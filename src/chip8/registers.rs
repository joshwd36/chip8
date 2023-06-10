pub struct Registers {
    registers: [u8; 16],
}

impl Registers {
    pub fn new() -> Self {
        let registers = [0; 16];
        Self {
            registers: registers,
        }
    }

    pub fn set_value(&mut self, register: u8, value: u8) {
        self.registers[register as usize] = value;
    }

    pub fn get_value(&self, register: u8) -> u8 {
        self.registers[register as usize]
    }
}
