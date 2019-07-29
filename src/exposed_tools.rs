pub use winit;
pub use winit::Event;
pub use winit::KeyboardInput;
pub use winit::VirtualKeyCode;

use crate::internal_tools::*;

pub const CURSOR_RESET_POS_X: u32 = 50;
pub const CURSOR_RESET_POS_Y: u32 = 50;

pub type VertexBuffer = CpuAccessibleBuffer<[Vertex]>;

extern crate nalgebra_glm as glm;
use glm::*;

pub type CameraMatrix = [[f32; 4]; 4];

#[derive(Clone)]
pub struct KeysDown {
    pub a: bool,
    pub b: bool,
    pub c: bool,
    pub d: bool,
    pub e: bool,
    pub f: bool,
    pub g: bool,
    pub h: bool,
    pub i: bool,
    pub j: bool,
    pub k: bool,
    pub l: bool,
    pub m: bool,
    pub n: bool,
    pub o: bool,
    pub p: bool,
    pub q: bool,
    pub r: bool,
    pub s: bool,
    pub t: bool,
    pub u: bool,
    pub v: bool,
    pub w: bool,
    pub x: bool,
    pub y: bool,
    pub z: bool,
}

pub struct KeyboardEvent {}

pub enum KeyboardEventType {
    Keyup,
    Keydown,
}

#[derive(Default, Debug, Clone)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub normal: [f32; 3],
}

pub fn get_elapsed(start: std::time::Instant) -> f32 {
    start.elapsed().as_secs() as f32 + start.elapsed().subsec_nanos() as f32 / 1_000_000_000.0
}

impl KeysDown {
    pub fn all_false() -> Self {
        KeysDown {
            a: false,
            b: false,
            c: false,
            d: false,
            e: false,
            f: false,
            g: false,
            h: false,
            i: false,
            j: false,
            k: false,
            l: false,
            m: false,
            n: false,
            o: false,
            p: false,
            q: false,
            r: false,
            s: false,
            t: false,
            u: false,
            v: false,
            w: false,
            x: false,
            y: false,
            z: false,
        }
    }
}

pub trait Camera {
    fn get_view_matrix(&self) -> CameraMatrix {
        Mat4::identity().into()
    }

    fn get_projection_matrix(&self) -> CameraMatrix {
        glm::perspective(
            // aspect ratio
            16. / 9.,
            // fov
            1.0,
            // near
            0.1,
            // far
            100_000_000.,
        )
        .into()
    }

    #[allow(unused_variables)]
    fn handle_input(&mut self, events: &[Event], keys_down: &KeysDown, delta: f32) {}
}

pub fn winit_event_to_keycode(event: &Event) -> Option<winit::KeyboardInput> {
    // only matches key press/release events
    if let Event::WindowEvent {
        event: WindowEvent::KeyboardInput { input, .. },
        ..
    } = event
    {
        if input.virtual_keycode.is_some() {
            Some(*input)
        } else {
            None
        }
    } else {
        None
    }
}

pub fn winit_event_to_mouse_movement(event: &Event) -> Option<(f32, f32)> {
    if let Event::WindowEvent {
        event: WindowEvent::CursorMoved { position: p, .. },
        ..
    } = event
    {
        let (x_diff, y_diff) = (
            p.x - (CURSOR_RESET_POS_X as f64),
            p.y - (CURSOR_RESET_POS_Y as f64),
        );
        let x_movement = x_diff as f32;
        let y_movement = y_diff as f32;

        Some((x_movement, y_movement))
    } else {
        None
    }
}

#[derive(Clone, Debug)]
pub struct MVP {
    pub model: CameraMatrix,
    pub view: CameraMatrix,
    pub proj: CameraMatrix,
}
