extern crate gilrs;
extern crate melon;
extern crate rand;
extern crate rpi_led_matrix;

mod enter_system;

use enter_system::EnterSystem;
use gilrs::{ev::EventType, Button, Event, Gilrs};
use melon::{Program, VM};
use rpi_led_matrix::{LedColor, LedMatrix, LedMatrixOptions};
use std::{sync::mpsc::{self, TryRecvError},
          thread};

const DIMENSION: usize = 32;

#[derive(Debug, Default, Clone)]
pub struct ControllerState {
    pub a: bool,
    pub b: bool,
    pub x: bool,
    pub y: bool,
    pub start: bool,
    pub select: bool,
    pub right_trigger: bool,
    pub left_trigger: bool,
    pub d_pad_up: bool,
    pub d_pad_down: bool,
    pub d_pad_left: bool,
    pub d_pad_right: bool,
}

const BUTTON_DOWN_VALUE: f32 = 1.0;
const BUTTON_UP_VALUE: f32 = 0.0;

fn main() {
    println!("Initializing matrix driver");
    let mut config = LedMatrixOptions::new();
    config.set_hardware_mapping("adafruit-hat");
    config.set_brightness(20).unwrap();

    let matrix = LedMatrix::new(Some(config)).unwrap();

    println!("Initializing input driver");
    let mut gilrs = Gilrs::new().unwrap();

    let mut ctrl_state = ControllerState::default();

    let color_map: Vec<_> = (0..256)
        .map(|color_byte: u16| {
            let red = (((color_byte >> 5) & 0b111) * 36) as u8;
            let green = (((color_byte >> 2) & 0b111) * 36) as u8;
            let blue = ((color_byte & 0b11) * 85) as u8;

            LedColor { red, green, blue }
        })
        .collect();

    let (frame_sender, frame_reveicer) = mpsc::channel();
    let (input_sender, input_reveicer) = mpsc::channel();

    let handle = thread::spawn(move || {
        let mut sys = EnterSystem::new(input_reveicer, frame_sender);

        let program = Program::from_file("enter.rom").unwrap();

        println!("Running \"enter.rom\"");
        VM::default()
            .exec(&program, &mut sys)
            .unwrap_or_else(|e| panic!("{}", e));
    });

    'main: loop {
        // Examine new events
        if let Some(Event { event, .. }) = gilrs.next_event() {
            match event {
                EventType::ButtonChanged(button, value, ..) => match button {
                    Button::South => ctrl_state.b = value == BUTTON_DOWN_VALUE,
                    Button::East => ctrl_state.a = value == BUTTON_DOWN_VALUE,
                    Button::North => ctrl_state.x = value == BUTTON_DOWN_VALUE,
                    Button::West => ctrl_state.y = value == BUTTON_DOWN_VALUE,
                    Button::LeftTrigger => ctrl_state.left_trigger = value == BUTTON_DOWN_VALUE,
                    Button::RightTrigger => ctrl_state.right_trigger = value == BUTTON_DOWN_VALUE,
                    Button::Select => ctrl_state.select = value == BUTTON_DOWN_VALUE,
                    Button::Start => ctrl_state.start = value == BUTTON_DOWN_VALUE,
                    Button::DPadUp => {
                        if value == BUTTON_UP_VALUE {
                            ctrl_state.d_pad_up = true;
                            ctrl_state.d_pad_down = !ctrl_state.d_pad_up;
                        } else if value == BUTTON_DOWN_VALUE {
                            ctrl_state.d_pad_down = true;
                            ctrl_state.d_pad_up = !ctrl_state.d_pad_down;
                        } else {
                            ctrl_state.d_pad_up = false;
                            ctrl_state.d_pad_down = false;
                        }
                    }
                    Button::DPadRight => {
                        if value == BUTTON_DOWN_VALUE {
                            ctrl_state.d_pad_right = true;
                            ctrl_state.d_pad_left = !ctrl_state.d_pad_right;
                        } else if value == BUTTON_UP_VALUE {
                            ctrl_state.d_pad_left = true;
                            ctrl_state.d_pad_right = !ctrl_state.d_pad_left;
                        } else {
                            ctrl_state.d_pad_right = false;
                            ctrl_state.d_pad_left = false;
                        }
                    }
                    _ => {}
                },
                _ => {}
            }

            if let Err(..) = input_sender.send(ctrl_state.clone()) {
                break 'main;
            }
        }

        match frame_reveicer.try_recv() {
            Ok(frame) => {
                let frame: Vec<u8> = frame;

                assert!(frame.len() <= DIMENSION * DIMENSION);

                let mut canvas = matrix.offscreen_canvas();

                let mut index = 0;
                for y_coord in 0..DIMENSION {
                    for x_coord in 0..DIMENSION {
                        if let Some(byte) = frame.get(index) {
                            canvas.set(
                                x_coord as i32,
                                y_coord as i32,
                                color_map.get(*byte as usize).unwrap(),
                            );

                            index += 1;
                        } else {
                            break 'main;
                        }
                    }
                }

                matrix.swap(canvas);
            }
            Err(TryRecvError::Disconnected) => break 'main,
            _ => {}
        }
    }

    handle.join().unwrap();
}
