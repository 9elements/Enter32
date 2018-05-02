use controller_state::ControllerState;
use def::DIMENSION;
use gilrs::Gilrs;
use rpi_led_matrix::{LedColor, LedMatrix, LedMatrixOptions};
use std::sync::mpsc::{Receiver, Sender, TryRecvError};

pub fn start(
    input_sender: Sender<ControllerState>,
    frame_reveicer: Receiver<Vec<u8>>,
    color_map: &Vec<(u8, u8, u8)>,
) {
    println!("Initializing input driver");
    let mut gilrs = Gilrs::new().unwrap();

    let mut ctrl_state = ControllerState::default();

    println!("Initializing matrix driver");
    let mut config = LedMatrixOptions::new();
    config.set_hardware_mapping("adafruit-hat");
    config.set_brightness(40).unwrap();

    let matrix = LedMatrix::new(Some(config)).unwrap();

    'main: loop {
        if gilrs.counter().positive() {
            if let Some(new_state) = ctrl_state.from_gamepad_state(&mut gilrs) {
                if let Err(..) = input_sender.send(new_state.clone()) {
                    break 'main;
                }

                ctrl_state = new_state
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
                            let (red, green, blue) = color_map.get(*byte as usize).unwrap();

                            canvas.set(
                                x_coord as i32,
                                y_coord as i32,
                                &LedColor {
                                    red: *red,
                                    green: *green,
                                    blue: *blue,
                                },
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
}
