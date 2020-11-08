#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Keystate {
    Pressed,
    Released,
}

pub struct Keypad {
    /// 16 个键，1 -> F , 如果被按下的话，就是 true
    keys: [Keystate; 16],
}

impl Keypad {
    pub fn new() -> Keypad {
        Keypad {
            keys: [Keystate::Released; 16],
        }
    }

    /// Return the state of the key at the given index.
    pub fn get_key_state(&self, index: usize) -> Keystate {
        debug_assert!(index < 16);
        self.keys[index]
    }

    /// Set the current key state for the key at the given index.
    pub fn set_key_state(&mut self, index: usize, state: Keystate) {
        debug_assert!(index < 16);
        self.keys[index] = state;
    }
}
