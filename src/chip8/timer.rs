use std::time::Duration;

pub struct Timer {
    value: u8,
}

pub const TIMER_DECREMENT: Duration = Duration::from_nanos(16_666_667);

impl Timer {
    pub fn new() -> Self {
        Self { value: 0 }
    }

    pub fn decrement(&mut self) {
        self.value = self.value.saturating_sub(1);
    }

    pub fn set_value(&mut self, value: u8) {
        self.value = value;
    }

    pub fn get_value(&self) -> u8 {
        self.value
    }
}
