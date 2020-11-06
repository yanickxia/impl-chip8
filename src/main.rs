mod cpu;
mod display;
mod keypad;
mod chip8app_sdl2;
mod chip8app;

#[macro_use]
extern crate lazy_static;

use std::thread::sleep;
use std::time::Duration;
use device_query::{DeviceQuery, DeviceState, MouseState, Keycode};
use std::fs;

fn main() {
    let contents = fs::read("flightrunner.ch8")
        .expect("Something went wrong reading the file");
    cpu::reset();
    cpu::load_rom(contents.as_slice());

    loop {
        cpu::execute_cycle();
        sleep(Duration::from_millis(100));
        cpu::draw();
    }
}
