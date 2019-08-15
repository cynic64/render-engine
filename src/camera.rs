extern crate nalgebra_glm as glm;

use self::glm::*;

use crate::exposed_tools::*;
use crate::input::*;

pub struct OrbitCamera {
    pub center_position: Vec3,
    pub front: Vec3,
    up: Vec3,
    right: Vec3,
    world_up: Vec3,
    // pitch and yaw are in radians
    pub pitch: f32,
    pub yaw: f32,
    pub orbit_distance: f32,
    mouse_sens: f32,
}

impl OrbitCamera {
    pub fn default() -> Self {
        let center_position = vec3(0.0, 0.0, 0.0);
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
        let mouse_sens = 0.0007;
        let orbit_distance = 4.0;

        Self {
            center_position,
            front,
            up,
            right,
            world_up,
            pitch,
            yaw,
            orbit_distance,
            mouse_sens,
        }
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

impl Camera for OrbitCamera {
    fn get_view_matrix(&self) -> [[f32; 4]; 4] {
        // orbits at 4 units away
        let farther_front = self.front * self.orbit_distance;
        look_at(
            &(self.center_position + farther_front),
            &self.center_position,
            &self.up,
        )
        .into()
    }

    fn handle_input(&mut self, frame_info: FrameInfo) {
        let x = frame_info.mouse_movement[0];
        let y = frame_info.mouse_movement[1];

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
}

pub struct FlyCamera {
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

impl FlyCamera {
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

        Self {
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

    pub fn move_forward(&mut self, delta: f32) {
        self.position += self.front * self.movement_speed * delta;
    }

    pub fn move_backward(&mut self, delta: f32) {
        self.position -= self.front * self.movement_speed * delta;
    }

    pub fn move_left(&mut self, delta: f32) {
        self.position -= self.right * self.movement_speed * delta;
    }

    pub fn move_right(&mut self, delta: f32) {
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

impl Camera for FlyCamera {
    fn get_view_matrix(&self) -> [[f32; 4]; 4] {
        // for normal fly camera
        look_at(&self.position, &(self.position + self.front), &self.up).into()
    }

    fn handle_input(&mut self, frame_info: FrameInfo) {
        let x = frame_info.mouse_movement[0];
        let y = frame_info.mouse_movement[1];

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

        // move if keys are down
        if frame_info.keys_down.w {
            self.move_forward(frame_info.delta);
        }
        if frame_info.keys_down.a {
            self.move_left(frame_info.delta);
        }
        if frame_info.keys_down.s {
            self.move_backward(frame_info.delta);
        }
        if frame_info.keys_down.d {
            self.move_right(frame_info.delta);
        }
    }
}

pub struct OrthoCamera {}

impl Camera for OrthoCamera {
    fn get_view_matrix(&self) -> [[f32; 4]; 4] {
        Mat4::identity().into()
    }

    fn get_projection_matrix(&self) -> [[f32; 4]; 4] {
        Mat4::identity().into()
    }

    fn handle_input(&mut self, _frame_info: FrameInfo) {}
}
