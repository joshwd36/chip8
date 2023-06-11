use crossbeam_channel::Receiver;

pub struct Keypad {
    receiver: Receiver<Event>,
    has_stopped: bool,
    key_states: [bool; 16],
    last_pressed: Option<u8>,
}

impl Keypad {
    pub fn new(receiver: Receiver<Event>) -> Self {
        let key_states = [false; 16];
        Self {
            receiver,
            has_stopped: false,
            key_states,
            last_pressed: None,
        }
    }

    pub fn process(&mut self) {
        while let Ok(event) = self.receiver.try_recv() {
            match event {
                Event::KeyDown(key) => self.key_states[key as usize] = true,
                Event::KeyUp(key) => {
                    self.key_states[key as usize] = false;
                    self.last_pressed = Some(key as u8)
                }
                Event::Stop => self.has_stopped = true,
            }
        }
    }

    pub fn is_key_pressed(&self, key_number: u8) -> bool {
        self.key_states[key_number as usize]
    }

    pub fn last_pressed(&mut self) -> Option<u8> {
        self.last_pressed.take()
    }
}

pub enum Event {
    KeyDown(Key),
    KeyUp(Key),
    Stop,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Key {
    Key1,
    Key2,
    Key3,
    Key4,
    KeyQ,
    KeyW,
    KeyE,
    KeyR,
    KeyA,
    KeyS,
    KeyD,
    KeyF,
    KeyZ,
    KeyX,
    KeyC,
    KeyV,
}
