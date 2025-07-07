use winit::{
    event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent},
    window::Window,
};

#[derive(Debug)]
pub struct EventResponse {
    pub repaint: bool,
    pub click: bool, // an click release event, no drag when pressed
}

impl Default for EventResponse {
    fn default() -> Self {
        Self {
            repaint: false,
            click: false,
        }
    }
}

pub struct Input {
    pub is_mouse_left_down: bool,
    pub prev_mouse_pos: (f32, f32),
    pub current_mouse_pos: (f32, f32),
    pub mouse_wheel_delta: f32,
    pub moved_during_press: bool, // if not drag and drop, we select block
}

impl Input {
    pub fn new() -> Self {
        Self {
            is_mouse_left_down: false,
            prev_mouse_pos: (0.0, 0.0),
            current_mouse_pos: (0.0, 0.0),
            mouse_wheel_delta: 0.0,
            moved_during_press: false,
        }
    }

    pub fn handle_input(&mut self, _window: &Window, event: &WindowEvent) -> EventResponse {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.prev_mouse_pos = self.current_mouse_pos;
                self.current_mouse_pos = (position.x as f32, position.y as f32);
                if self.is_mouse_left_down {
                    self.moved_during_press = true;
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.mouse_wheel_delta = match delta {
                    MouseScrollDelta::LineDelta(_, y) => *y,
                    MouseScrollDelta::PixelDelta(physical_position) => {
                        physical_position.y as f32 / 9.0
                    }
                }
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                self.is_mouse_left_down = matches!(state, ElementState::Pressed);
                if !self.is_mouse_left_down {
                    if !self.moved_during_press {
                        return EventResponse {
                            click: true,
                            ..Default::default()
                        };
                    }
                    self.moved_during_press = false;
                }
            }
            _ => {
                self.prev_mouse_pos = self.current_mouse_pos;
                self.mouse_wheel_delta = 0.0;
            }
        };
        EventResponse::default()
    }
}
