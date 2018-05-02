use gilrs::{ev::EventType, Button, Event, Gilrs};

const BUTTON_DOWN_VALUE: f32 = 1.0;
const BUTTON_UP_VALUE: f32 = 0.0;

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

impl ControllerState {
    #[cfg(not(target_os = "macos"))]
    pub fn from_gamepad_state(&mut self, gilrs: &mut Gilrs) -> Option<ControllerState> {
        // Examine new events
        if let Some(Event { event, .. }) = gilrs.next_event() {
            match event {
                EventType::ButtonChanged(button, value, ..) => match button {
                    Button::South => self.b = value == BUTTON_DOWN_VALUE,
                    Button::East => self.a = value == BUTTON_DOWN_VALUE,
                    Button::North => self.x = value == BUTTON_DOWN_VALUE,
                    Button::West => self.y = value == BUTTON_DOWN_VALUE,
                    Button::LeftTrigger => self.left_trigger = value == BUTTON_DOWN_VALUE,
                    Button::RightTrigger => self.right_trigger = value == BUTTON_DOWN_VALUE,
                    Button::Select => self.select = value == BUTTON_DOWN_VALUE,
                    Button::Start => self.start = value == BUTTON_DOWN_VALUE,
                    Button::DPadUp => {
                        if value == BUTTON_UP_VALUE {
                            self.d_pad_up = true;
                            self.d_pad_down = !self.d_pad_up;
                        } else if value == BUTTON_DOWN_VALUE {
                            self.d_pad_down = true;
                            self.d_pad_up = !self.d_pad_down;
                        } else {
                            self.d_pad_up = false;
                            self.d_pad_down = false;
                        }
                    }
                    Button::DPadRight => {
                        if value == BUTTON_DOWN_VALUE {
                            self.d_pad_right = true;
                            self.d_pad_left = !self.d_pad_right;
                        } else if value == BUTTON_UP_VALUE {
                            self.d_pad_left = true;
                            self.d_pad_right = !self.d_pad_left;
                        } else {
                            self.d_pad_right = false;
                            self.d_pad_left = false;
                        }
                    }
                    _ => {}
                },
                _ => {}
            }

            Some(self.clone())
        } else {
            None
        }
    }

    #[cfg(target_os = "macos")]
    pub fn from_gamepad_state(&mut self, _: &mut ()) -> Option<ControllerState> {
        eprintln!("Gamepad support is disabled on macOS");

        None
    }
}
