use std::fmt::Display;

use crossbeam_channel::Sender;

pub struct Chip8Display {
    buffer: Box<[bool]>,
    sender: Sender<DisplayInstruction>,
}

impl Display for Chip8Display {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for y in 0..32 {
            for x in 0..64 {
                let value = self.buffer[x + y * 64];
                let icon = if value { "◽" } else { "◾" };
                write!(f, "{}", icon)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

pub enum DisplayInstruction {
    Set { value: bool, index: usize },
    Clear,
}

impl Chip8Display {
    pub fn new(sender: Sender<DisplayInstruction>) -> Self {
        let buffer = vec![false; 2048].into_boxed_slice();
        Self { buffer, sender }
    }

    pub fn set(&mut self, x: usize, y: usize) -> bool {
        let index = x + y * 64;
        let existing = self.buffer[index];
        self.buffer[index] = !existing;
        self.sender
            .send(DisplayInstruction::Set {
                value: !existing,
                index,
            })
            .unwrap();
        existing
    }

    pub fn clear(&mut self) {
        self.buffer.fill(false);
        self.sender.send(DisplayInstruction::Clear).unwrap();
    }
}
