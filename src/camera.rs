use cgmath::prelude::*;

#[derive(Debug)]
enum MouseState {
    Unpressed,
    PressedDown { position: cgmath::Point2<f32> },
    Held,
}

#[derive(Debug)]
pub struct Camera {
    displacement: cgmath::Vector2<f32>,
    scale: f32,
    mouse_state: MouseState,
}

impl Camera {
    pub fn create_matrix(&self) -> cgmath::Matrix4<f32> {
        cgmath::Matrix4::from_scale(self.scale)
            * cgmath::Matrix4::from_translation(self.displacement.extend(0.0))
    }

    pub fn new() -> Self {
        Self {
            displacement: Zero::zero(),
            scale: 0.1,
            mouse_state: MouseState::Unpressed,
        }
    }

    pub fn click_mouse(&mut self, p: cgmath::Point2<f32>) {
        if let MouseState::Unpressed = self.mouse_state {
            self.mouse_state = MouseState::PressedDown { position: p }
        }
    }

    pub fn drag_mouse(&mut self, abs_pos: cgmath::Point2<f32>, mut rel_pos: cgmath::Vector2<f32>) {
        if let MouseState::PressedDown { position } = self.mouse_state {
            if position.distance(abs_pos) > 0.025 {
                self.mouse_state = MouseState::Held;
                rel_pos += position - abs_pos;
            }
        }
        if let MouseState::Held = self.mouse_state {
            self.displacement += rel_pos / self.scale;
        }
    }

    pub fn let_go_of_mouse(&mut self) {
        self.mouse_state = MouseState::Unpressed;
    }

    pub fn scroll(&mut self, amount: f32) {
        self.scale *= (-amount).exp();
    }
}
