use crossbeam_channel::Receiver;

pub struct Keypad {
    receiver: Receiver<Event>,
    has_stopped: bool,
    key_states: [bool; 16],
    last_pressed: LastKeyState,
}

impl Keypad {
    pub fn new(receiver: Receiver<Event>) -> Self {
        let key_states = [false; 16];
        Self {
            receiver,
            has_stopped: false,
            key_states,
            last_pressed: LastKeyState::NotWaiting,
        }
    }

    pub fn process(&mut self) {
        while let Ok(event) = self.receiver.try_recv() {
            match event {
                Event::KeyDown(key) => self.key_states[key as usize] = true,
                Event::KeyUp(key) => {
                    self.key_states[key as usize] = false;
                    self.last_pressed = match self.last_pressed {
                        LastKeyState::NotWaiting => LastKeyState::NotWaiting,
                        LastKeyState::Waiting => LastKeyState::Pressed(key as u8),
                        LastKeyState::Pressed(_) => LastKeyState::Pressed(key as u8),
                    };
                }
                Event::Stop => self.has_stopped = true,
            }
        }
    }

    pub fn is_key_pressed(&self, key_number: u8) -> bool {
        self.key_states[key_number as usize]
    }

    pub fn last_pressed(&mut self) -> Option<u8> {
        let (new_state, result) = match self.last_pressed {
            LastKeyState::NotWaiting | LastKeyState::Waiting => (LastKeyState::Waiting, None),
            LastKeyState::Pressed(key) => (LastKeyState::NotWaiting, Some(key)),
        };
        self.last_pressed = new_state;
        result
    }
}

enum LastKeyState {
    NotWaiting,
    Waiting,
    Pressed(u8),
}

pub enum Event {
    KeyDown(Key),
    KeyUp(Key),
    Stop,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Key {
    Key1 = 0x1,
    Key2 = 0x2,
    Key3 = 0x3,
    Key4 = 0xC,
    KeyQ = 0x4,
    KeyW = 0x5,
    KeyE = 0x6,
    KeyR = 0xD,
    KeyA = 0x7,
    KeyS = 0x8,
    KeyD = 0x9,
    KeyF = 0xE,
    KeyZ = 0xA,
    KeyX = 0x0,
    KeyC = 0xB,
    KeyV = 0xF,
}
