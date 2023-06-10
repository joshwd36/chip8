pub struct Stack {
    buffer: Vec<u16>,
}

impl Stack {
    pub fn new() -> Self {
        let buffer = Vec::new();

        Self { buffer }
    }

    pub fn push(&mut self, value: u16) {
        self.buffer.push(value);
    }

    pub fn pop(&mut self) -> Option<u16> {
        self.buffer.pop()
    }
}
