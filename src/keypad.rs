#[derive(Copy, Clone, Debug)]
pub enum KeyState {
    Pressed,
    Released,
}

pub struct Keypad {
    keys: [KeyState; 16],
}

impl Keypad {
    pub fn new() -> Keypad {
        Keypad {
            keys: [KeyState::Released; 16],
        }
    }

    pub fn get_key_state(&self, index: usize) -> KeyState {
        self.keys[index]
    }

    pub fn set_key_state(&mut self, index: usize, state: KeyState) {
        self.keys[index] = state;
    }
}