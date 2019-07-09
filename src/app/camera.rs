extern crate nalgebra_glm as glm;

use self::glm::*;

use super::*;

pub struct OrbitCamera {
    pub position: Vec3,
    pub front: Vec3,
    up: Vec3,
    right: Vec3,
    world_up: Vec3,
    // pitch and yaw are in radians
    pub pitch: f32,
    pub yaw: f32,
    movement_speed: f32,
    mouse_sens: f32,
}

impl OrbitCamera {
    pub fn default() -> Self {
        let position = vec3(0.0, 0.0, 0.0);
        let pitch: f32 = 0.0;
        let yaw: f32 = std::f32::consts::PI / 2.0;
        let front = normalize(&vec3(
            pitch.cos() * yaw.cos(),
            pitch.sin(),
            pitch.cos() * yaw.sin(),
        ));
        let right = vec3(0.0, 0.0, 0.0);
        let up = vec3(0.0, 1.0, 0.0);
        let world_up = vec3(0.0, 1.0, 0.0);
        let movement_speed = 20.0;
        let mouse_sens = 0.0007;

        OrbitCamera {
            position,
            front,
            up,
            right,
            world_up,
            pitch,
            yaw,
            movement_speed,
            mouse_sens,
        }
    }

    pub fn _move_forward(&mut self, delta: f32) {
        self.position += self.front * self.movement_speed * delta;
    }

    pub fn _move_backward(&mut self, delta: f32) {
        self.position -= self.front * self.movement_speed * delta;
    }

    pub fn _move_left(&mut self, delta: f32) {
        self.position -= self.right * self.movement_speed * delta;
    }

    pub fn _move_right(&mut self, delta: f32) {
        self.position += self.right * self.movement_speed * delta;
    }

    fn update(&mut self) {
        self.front = normalize(&vec3(
            self.pitch.cos() * self.yaw.cos(),
            self.pitch.sin(),
            self.pitch.cos() * self.yaw.sin(),
        ));

        self.right = normalize(&glm::Vec3::cross(&self.front, &self.world_up));
    }
}

impl InputHandlingCamera for OrbitCamera {
    fn get_view_matrix(&self) -> [[f32; 4]; 4] {
        // for normal fly camera
        // look_at(&self.position, &(self.position + self.front), &self.up)

        // for orbit camera (orbits at 4 units away)
        let farther_front = self.front * 4.0;
        look_at(&(self.position + farther_front), &self.position, &self.up).into()
    }

    fn handle_input(&mut self, events: &[Event]) {
        events.iter().for_each(|ev| {
            if let Some(mouse_movement) = winit_event_to_mouse_movement(ev) {
                let (x, y) = mouse_movement;

                self.pitch += y * self.mouse_sens;
                self.yaw += x * self.mouse_sens;
                let halfpi = std::f32::consts::PI / 2.0;
                let margin = 0.01;
                let max_pitch = halfpi - margin;

                if self.pitch > max_pitch {
                    self.pitch = max_pitch;
                } else if self.pitch < -max_pitch {
                    self.pitch = -max_pitch;
                }

                self.update();
            }
        });
    }
}
