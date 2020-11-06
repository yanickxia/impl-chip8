use crate::display::{Display, FONT_SET};
use std::sync::Mutex;
use rand::Rng;
use crate::keypad::Keypad;
use std::io;


lazy_static! {
    pub static ref CPU: Mutex<Cpu> = Mutex::new(Cpu {
        i: 0,
        pc: 0,
        memory: [0; 4096],
        v: [0; 16],
        display: Display::new(),
        stack: [0; 16],
        sp: 0,
        delay_timer: 0,
        sound_timer: 0,
        keypad: Keypad::new()
    });
}

pub struct Cpu {
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
    pub sp: u8,

    // delay timer
    pub delay_timer: u8,

    // sound timer
    pub sound_timer: u8,
}

pub fn execute_cycle() {
    CPU.lock().unwrap().run()
}

pub fn reset() {
    CPU.lock().unwrap().reset()
}


pub fn draw() {
    let cpu = CPU.lock().unwrap();
    cpu.display.render();
}

pub fn load_rom(rom: &[u8]) {
    let cpu: &mut Cpu = &mut *(CPU.lock().unwrap());
    for i in 0..rom.len() {
        cpu.memory[0x200 + i] = rom[i]
    }
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

impl Cpu {
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
                self.pc = self.stack[self.sp as usize]
            }
            0x1000...0x1FFF => {
                self.pc = arg_nnn!(opcode)
            }
            0x2000...0x2FFF => {
                self.stack[self.sp as usize] = self.pc;
                self.sp = self.sp + 1;
                self.pc = arg_nnn!(opcode);
            }
            0x3000...0x3FFF => {
                if self.v[arg_x!(opcode)] == arg_nn!(opcode) {
                    self.sp += 2
                }
            }
            0x4000...0x4FFF => {
                if self.v[arg_x!(opcode)] != arg_nn!(opcode) {
                    self.sp += 2
                }
            }
            0x5000...0x5FFF => {
                if self.v[arg_x!(opcode)] == self.v[arg_y!(opcode)] {
                    self.sp += 2
                }
            }
            0x600...0x6FFF => {
                self.v[arg_x!(opcode)] = arg_nn!(opcode);
            }
            0x700...0x7FFF => {
                self.v[arg_x!(opcode)] = self.v[arg_x!(opcode)].overflowing_add(self.v[arg_y!(opcode)]).0
            }
            0x8000...0x8FF0 => {
                self.v[arg_x!(opcode)] = self.v[arg_y!(opcode)]
            }
            0x8001...0x8FF1 => {
                self.v[arg_x!(opcode)] |= self.v[arg_y!(opcode)]
            }
            0x8002...0x8FF2 => {
                self.v[arg_x!(opcode)] &= self.v[arg_y!(opcode)]
            }
            0x8003...0x8FF3 => {
                self.v[arg_x!(opcode)] ^= self.v[arg_y!(opcode)]
            }
            0x8004...0x8FF4 => {
                let (res, overflow) = self.v[arg_x!(opcode)].overflowing_add(self.v[arg_y!(opcode)]);
                match overflow {
                    true => self.v[0xF] = 1,
                    false => self.v[0xF] = 0,
                }
                self.v[arg_x!(opcode)] = res;
            }
            0x8005...0x8FF5 => {
                let (res, overflow) = self.v[arg_x!(opcode)].overflowing_sub(self.v[arg_y!(opcode)]);
                match overflow {
                    true => self.v[0xF] = 0,
                    false => self.v[0xF] = 1,
                }
                self.v[arg_x!(opcode)] = res;
            }
            0x8006...0x8FF6 => {
                let y = self.v[arg_y!(opcode)];
                self.v[0xF] = (0x0001 & opcode) as u8;
                self.v[arg_x!(opcode)] = y >> 1;
            }
            0x8007...0x8FF7 => {
                self.v[arg_x!(opcode)] = self.v[arg_y!(opcode)] - self.v[arg_x!(opcode)]
            }
            0x800E...0x8FFE => {
                let y = self.v[arg_y!(opcode)];
                self.v[0xF] = ((0x8000 & opcode) >> 15) as u8;
                self.v[arg_x!(opcode)] = y >> 1;
            }
            0x9000...0x9FF0 => {
                if self.v[arg_x!(opcode)] != self.v[arg_y!(opcode)] {
                    self.sp += 2
                }
            }
            0xA000...0xAFFF => {
                self.i = arg_nnn!(opcode)
            }
            0xB000...0xBFFF => {
                self.pc = self.v[0] as u16 + arg_nnn!(opcode)
            }
            0xC000...0xCFFF => {
                let rand: u8 = rand::random::<u8>();
                self.v[arg_x!(opcode)] = rand & arg_nn!(opcode)
            }
            0xD000...0xDFFF => {
                let collision = self.display.draw(self.v[arg_x!(opcode)] as usize, self.v[arg_y!(opcode)] as usize,
                                                  &self.memory[self.i as usize..(self.i + arg_n!(opcode) as u16) as usize]);
                self.v[0xF] = if collision { 1 } else { 0 };
            }
            0xE09E...0xEF9E => {
                if self.keypad.is_press(arg_x!(opcode) as usize) {
                    self.pc += 2
                }
            }
            0xE0A1...0xEFA1 => {
                if !self.keypad.is_press(arg_x!(opcode) as usize) {
                    self.pc += 2
                }
            }
            0xF007...0xFF07 => {
                self.v[arg_x!(opcode)] = self.delay_timer
            }
            0xF00A...0xFF0A => {
                self.v[arg_x!(opcode)] = self.keypad.key_blocking();
            }
            0xF015...0xFF15 => {
                self.delay_timer = self.v[arg_x!(opcode)]
            }
            0xF018...0xFF18 => {
                self.sound_timer = self.v[arg_x!(opcode)]
            }
            0xF01E...0xFF1E => {
                self.i += self.v[arg_x!(opcode)] as u16
            }
            0xF029...0xFF29 => {
                self.i = self.v[arg_x!(opcode)] as u16 * 5
            }
            0xF033...0xFF33 => {
                self.memory[self.i as usize] = self.v[arg_x!(opcode)] / 100;
                self.memory[self.i as usize + 1] = (self.v[arg_x!(opcode)] / 10) % 10;
                self.memory[self.i as usize + 2] = (self.v[arg_x!(opcode)] % 100) % 10;
            }
            0xF055...0xFF55 => {
                self.memory[(self.i as usize)..(self.i + arg_x!(opcode) as u16 + 1) as usize]
                    .copy_from_slice(&self.v[0..(arg_x!(opcode) as usize + 1)])
            }
            0xF065...0xFF65 => {
                self.v[0..(arg_x!(opcode) as usize + 1)]
                    .copy_from_slice(&self.memory[(self.i as usize)..(self.i + arg_x!(opcode) as u16 + 1) as usize])
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