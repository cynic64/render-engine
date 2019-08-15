pub use winit;
pub use winit::Event;
pub use winit::KeyboardInput;
pub use winit::VirtualKeyCode;

use crate::internal_tools::*;

use crate::input::FrameInfo;

pub const CURSOR_RESET_POS_X: u32 = 50;
pub const CURSOR_RESET_POS_Y: u32 = 50;

extern crate nalgebra_glm as glm;
use glm::*;

pub type CameraMatrix = [[f32; 4]; 4];

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

pub type AbstractVbuf = Arc<dyn BufferAccess + Send + Sync>;

pub fn get_elapsed(start: std::time::Instant) -> f32 {
    start.elapsed().as_secs() as f32 + start.elapsed().subsec_nanos() as f32 / 1_000_000_000.0
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
    fn handle_input(&mut self, frame_info: FrameInfo) {}
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

pub fn winit_event_to_cursor_position(event: &Event) -> Option<[f32; 2]> {
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

        Some([x_movement, y_movement])
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
