use crate::display::{Display, FONT_SET};
use std::sync::Mutex;
use rand::Rng;
use crate::keypad::{Keypad, Keystate};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::error::Error;

pub struct Vm {
    // register index
    pub i: u16,

    // program counter
    pub pc: u16,

    // memory
    pub memory: [u8; 4096],

    //registers, v0 - vf
    pub v: [u8; 16],

    //display
    pub display: Display,

    //key
    pub keypad: Keypad,

    //stack
    pub stack: [u16; 16],

    //stack point
    pub sp: u16,

    // delay timer
    pub delay_timer: u8,

    // sound timer
    pub sound_timer: u8,

    pub wait_for_key: (bool, u8),

    run_counter: u64,
}

fn read_word(memory: [u8; 4096], index: u16) -> u16 {
    (memory[index as usize] as u16) << 8
        | (memory[(index + 1) as usize] as u16)
}


macro_rules! arg_x {
    ($opcode:expr) => {
        (($opcode & 0x0F00) >> 8) as usize
    };
}

macro_rules! arg_y {
    ($opcode:expr) => {
        (($opcode & 0x00F0) >> 4) as usize
    };
}

macro_rules! arg_nnn {
    ($opcode:expr) => {
        $opcode & 0x0FFF
    };
}

macro_rules! arg_nn {
    ($opcode:expr) => {
        ($opcode & 0x00FF) as u8
    };
}

macro_rules! arg_n {
    ($opcode:expr) => {
        ($opcode & 0x000F) as u8
    };
}

impl Vm {
    pub fn new() -> Vm {
        let mut vm = Vm {
            i: 0,
            pc: 0,
            memory: [0; 4096],
            v: [0; 16],
            display: Display::new(),
            keypad: Keypad::new(),
            stack: [0; 16],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            wait_for_key: (false, 0),
            run_counter: 0,
        };

        for i in 0..80 {
            vm.memory[i] = FONT_SET[i];
        }
        // the program space starts at 0x200
        vm.pc = 0x200;

        vm
    }

    pub fn is_waiting_for_key(&self) -> bool {
        self.wait_for_key.0
    }

    pub fn debug_info(&self, opt: u16) {
        println!("run counter: {}, opt: {:X?}", self.run_counter, opt);
        println!("register: {:?}", self.v);
        println!("pc: {:?}", self.pc);
        println!("stack: {:?}\n\n\n", self.stack);
    }

    pub fn end_wait_for_key(&mut self, key_index: usize) {
        if !self.is_waiting_for_key() {
            warn!(concat!(
            "Chip8::end_wait_for_key_press called but the VM ",
            "wasn't waiting for a key press - ignoring"
            ));
            return;
        }
        self.v[self.wait_for_key.1 as usize] = key_index as u8;
        self.wait_for_key.0 = false;
        self.pc += 2;
    }

    pub fn load(&mut self, filepath: &Path) -> Option<String> {
        let file = match File::open(filepath) {
            Ok(f) => f,
            Err(ref why) => {
                return Some(format!(
                    "couldn't open rom file \"{}\" : {}",
                    filepath.display(),
                    Error::description(why)
                ));
            }
        };
        for (i, b) in file.bytes().enumerate() {
            //if b.is_none() /* EOF */ { break; }
            match b {
                Ok(byte) => self.memory[self.pc as usize + i] = byte,
                Err(e) => {
                    return Some(format!("error while reading ROM : {}", e.to_string()));
                }
            }
        }
        None
    }

    pub fn reset(&mut self) {
        self.i = 0;
        self.pc = 0x200;
        self.memory = [0; 4096];
        self.v = [0; 16];
        self.stack = [0; 16];
        self.sp = 0;
        self.delay_timer = 0;
        self.display.clear();
        for i in 0..80 {
            self.memory[i] = FONT_SET[i];
        }
    }

    pub fn emulate_cycle(&mut self) -> bool {
        // Is the program finished ?
        if self.pc >= 4094 {
            return true;
        }
        // Fetch and execute the opcode to execute ;
        // an opcode being 2 bytes long, we need to read 2 bytes from memory
        let op = read_word(self.memory, self.pc);

        self.run_counter += 1;
        self.process_opcode(op);
        self.debug_info(op);

        false
    }

    pub fn run(&mut self) {
        let opcode = read_word(self.memory, self.pc);
        self.process_opcode(opcode);
    }

    pub fn decrement_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
    }

    fn process_opcode(&mut self, opcode: u16) {
        self.pc += 2;

        match opcode {
            0x00E0 => {
                self.display.clear()
            }
            0x00EE => {
                self.sp = self.sp - 1;
                self.pc = self.stack[(self.sp) as usize] + 2
            }
            0x1000..=0x1FFF => {
                self.pc = arg_nnn!(opcode)
            }
            0x2000..=0x2FFF => {
                self.stack[self.sp as usize] = self.pc - 2;
                self.sp = self.sp + 1;
                self.pc = arg_nnn!(opcode);
            }
            0x3000..=0x3FFF => {
                // let nn = arg_nn!(opcode);
                // let vx = self.v[arg_x!(opcode)];
                if self.v[arg_x!(opcode)] == arg_nn!(opcode) {
                    self.pc += 2
                }
            }
            0x4000..=0x4FFF => {
                if self.v[arg_x!(opcode)] != arg_nn!(opcode) {
                    self.pc += 2
                }
            }
            0x5000..=0x5FFF => {
                if self.v[arg_x!(opcode)] == self.v[arg_y!(opcode)] {
                    self.pc += 2
                }
            }
            0x6000..=0x6FFF => {
                self.v[arg_x!(opcode)] = arg_nn!(opcode);
            }
            0x7000..=0x7FFF => {
                self.v[arg_x!(opcode)] += arg_nn!(opcode);
            }

            0x8000..=0x8FFF => {
                match opcode & 0x000F {
                    0 => {
                        self.v[arg_x!(opcode)] = self.v[arg_y!(opcode)]
                    }
                    1 => {
                        self.v[arg_x!(opcode)] |= self.v[arg_y!(opcode)]
                    }
                    2 => {
                        self.v[arg_x!(opcode)] &= self.v[arg_y!(opcode)]
                    }
                    3 => {
                        self.v[arg_x!(opcode)] ^= self.v[arg_y!(opcode)]
                    }
                    4 => {
                        let (res, overflow) = self.v[arg_x!(opcode)].overflowing_add(self.v[arg_y!(opcode)]);
                        match overflow {
                            true => self.v[0xF] = 1,
                            false => self.v[0xF] = 0,
                        }
                        self.v[arg_x!(opcode)] = res;
                    }
                    5 => {
                        let (res, overflow) = self.v[arg_x!(opcode)].overflowing_sub(self.v[arg_y!(opcode)]);
                        match overflow {
                            true => self.v[0xF] = 0,
                            false => self.v[0xF] = 1,
                        }
                        self.v[arg_x!(opcode)] = res;
                    }
                    6 => {
                        self.v[0xF] = self.v[arg_x!(opcode)] & 0x1;
                        self.v[arg_x!(opcode)] >>= 1;
                    }
                    7 => {
                        let (res, overflow) = self.v[arg_y!(opcode)].overflowing_sub(self.v[arg_x!(opcode)]);
                        match overflow {
                            true => self.v[0xF] = 0,
                            false => self.v[0xF] = 1,
                        }
                        self.v[arg_x!(opcode)] = res;
                    }
                    0xE => {
                        self.v[0xF] = self.v[arg_x!(opcode)] & 0x80;
                        self.v[arg_x!(opcode)] <<= 1;
                    }
                    _ => {}
                }
            }
            0x9000..=0x9FF0 => {
                if self.v[arg_x!(opcode)] != self.v[arg_y!(opcode)] {
                    self.sp += 2
                }
            }
            0xA000..=0xAFFF => {
                self.i = arg_nnn!(opcode)
            }
            0xB000..=0xBFFF => {
                self.pc = self.v[0] as u16 + arg_nnn!(opcode)
            }
            0xC000..=0xCFFF => {
                let rand: u8 = rand::random::<u8>();
                self.v[arg_x!(opcode)] = rand & arg_nn!(opcode)
            }
            0xD000..=0xDFFF => {
                let collision = self.display.draw(self.v[arg_x!(opcode)] as usize, self.v[arg_y!(opcode)] as usize,
                                                  &self.memory[self.i as usize..(self.i + arg_n!(opcode) as u16) as usize]);
                self.v[0xF] = if collision { 1 } else { 0 };
            }
            0xE000..=0xEFFF => {
                if arg_nn!(opcode) == 0x9E {
                    self.pc += match self.keypad.get_key_state(self.v[arg_x!(opcode)] as usize) {
                        Keystate::Pressed => 2,
                        Keystate::Released => 0,
                    };
                } else if arg_nn!(opcode) == 0xA1 {
                    self.pc += match self.keypad.get_key_state(self.v[arg_x!(opcode)] as usize) {
                        Keystate::Pressed => 0,
                        Keystate::Released => 2,
                    };
                }
            }

            0xF000..=0xFFFF => {
                match opcode & 0x00FF {
                    0x07..=0x07 => {
                        self.v[arg_x!(opcode)] = self.delay_timer
                    }
                    0x0A => {
                        self.wait_for_key = (true, arg_x!(opcode) as u8);
                        self.pc -= 2;
                    }
                    0x15 => {
                        self.delay_timer = self.v[arg_x!(opcode)]
                    }
                    0x18 => {
                        self.sound_timer = self.v[arg_x!(opcode)]
                    }
                    0x1E => {
                        self.i += self.v[arg_x!(opcode)] as u16
                    }
                    0x29 => {
                        self.i = self.v[arg_x!(opcode)] as u16 * 5
                    }
                    0x33 => {
                        self.memory[self.i as usize] = self.v[arg_x!(opcode)] / 100;
                        self.memory[self.i as usize + 1] = (self.v[arg_x!(opcode)] / 10) % 10;
                        self.memory[self.i as usize + 2] = (self.v[arg_x!(opcode)] % 100) % 10;
                    }
                    0x55 => {
                        self.memory[(self.i as usize)..(self.i + arg_x!(opcode) as u16 + 1) as usize]
                            .copy_from_slice(&self.v[0..(arg_x!(opcode) as usize + 1)])
                    }
                    0x65 => {
                        self.v[0..(arg_x!(opcode) as usize + 1)]
                            .copy_from_slice(&self.memory[(self.i as usize)..(self.i + arg_x!(opcode) as u16 + 1) as usize])
                    }
                    _ => println!("got unknown opcode: {}", opcode)
                }
            }
            _ => {
                println!("got unknown opcode: {}", opcode)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arg_x() {
        assert_eq!(arg_x!(0x0100), 0x1);
    }
}