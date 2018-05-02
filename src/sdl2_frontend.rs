use controller_state::ControllerState;
use def::DIMENSION;
use gilrs::Gilrs;
use sdl2::{self, event::Event as SDL2Event, pixels::Color};
use std::{sync::mpsc::{Receiver, Sender, TryRecvError},
          thread,
          time::Duration};

pub fn start(
    input_sender: Sender<ControllerState>,
    frame_reveicer: Receiver<Vec<u8>>,
    color_map: &Vec<(u8, u8, u8)>,
) {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("DEBUG MATRIX", 32 * 8, 32 * 8)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window
        .into_canvas()
        .target_texture()
        .present_vsync()
        .build()
        .unwrap();

    canvas.set_scale(8.0, 8.0).unwrap();

    let mut gilrs = if cfg!(not(target_os = "macos")) {
        Gilrs::new().unwrap();
    };

    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut ctrl_state = ControllerState::default();

    'main: loop {
        if let Some(SDL2Event::Quit { .. }) = event_pump.poll_event() {
            break 'main;
        }

        if cfg!(not(target_os = "macos")) {
            if let Some(new_state) = ctrl_state.from_gamepad_state(&mut gilrs) {
                if let Err(..) = input_sender.send(new_state.clone()) {
                    break 'main;
                }

                ctrl_state = new_state
            }
        };

        match frame_reveicer.try_recv() {
            Ok(frame) => {
                let frame: Vec<u8> = frame;

                assert!(frame.len() <= DIMENSION * DIMENSION);

                let mut index = 0;
                for y_coord in 0..DIMENSION {
                    for x_coord in 0..DIMENSION {
                        if let Some(byte) = frame.get(index) {
                            let &(red, green, blue) = color_map.get(*byte as usize).unwrap();

                            canvas.set_draw_color(Color::RGB(red, green, blue));
                            canvas.draw_point((x_coord as i32, y_coord as i32)).unwrap();

                            index += 1;
                        } else {
                            break 'main;
                        }
                    }
                }

                canvas.present();
            }
            Err(TryRecvError::Disconnected) => break 'main,
            _ => {
                thread::sleep(Duration::from_millis(10));
            }
        }
    }
}
