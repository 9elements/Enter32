extern crate gilrs;
extern crate melon;
#[cfg(all(target_os = "linux", any(target_arch = "arm", target_arch = "aarch64")))]
extern crate rpi_led_matrix;
#[cfg(not(any(target_os = "linux", target_arch = "arm", target_arch = "aarch64")))]
extern crate sdl2;

mod controller_state;
mod def;
mod enter_system;

#[cfg(all(target_os = "linux", any(target_arch = "arm", target_arch = "aarch64")))]
mod led_matrix_frontend;
#[cfg(all(target_os = "linux", any(target_arch = "arm", target_arch = "aarch64")))]
use led_matrix_frontend as frontend;

#[cfg(not(any(target_os = "linux", target_arch = "arm", target_arch = "aarch64")))]
mod sdl2_frontend;
#[cfg(not(any(target_os = "linux", target_arch = "arm", target_arch = "aarch64")))]
use sdl2_frontend as frontend;

use controller_state::ControllerState;
use enter_system::EnterSystem;
use melon::{Debugger, Program, VM};
use std::{env, sync::mpsc, thread};

const DEBUGGING_VAR: &str = "DEBUG";

fn main() {
    let color_map: Vec<_> = (0..=255)
        .map(|color_byte| {
            let red = (((color_byte >> 5) & 0b111) * 36) as u8;
            let green = (((color_byte >> 2) & 0b111) * 36) as u8;
            let blue = ((color_byte & 0b11) * 85) as u8;

            (red, green, blue)
        })
        .collect();

    let (frame_sender, frame_reveicer) = mpsc::channel();
    let (input_sender, input_reveicer) = mpsc::channel();

    thread::spawn(move || {
        let mut sys = EnterSystem::new(input_reveicer, frame_sender);

        let program = Program::from_file("enter.rom").unwrap();

        println!("Running \"enter.rom\"");

        match env::var(DEBUGGING_VAR) {
            Ok(..) => Debugger::default()
                .exec(&program, &mut sys)
                .unwrap_or_else(|e| panic!("{}", e)),
            _ => VM::default()
                .exec(&program, &mut sys)
                .unwrap_or_else(|e| panic!("{}", e)),
        }
    });

    frontend::start(input_sender, frame_reveicer, &color_map);
}
