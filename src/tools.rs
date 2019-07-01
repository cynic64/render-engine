#[derive(Debug, Clone)]
pub struct Vertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

pub struct PixelCoord {
    pub x: u32,
    pub y: u32,
}

pub enum Keydown {
    W,
    A,
    S,
    D,
}

use winit::{Event, WindowEvent};
pub use winit::VirtualKeyCode;

pub fn winit_event_to_keydown(event: Event) -> Option<VirtualKeyCode> {
    // only matches keydown events
    if let Event::WindowEvent {
        event: WindowEvent::KeyboardInput {
            input: winit::KeyboardInput {
                virtual_keycode: Some(key),
                state: winit::ElementState::Pressed,
                ..
            },
            ..
        },
        ..
    } = event {
        Some(key)
    } else {
        None
    }
}
