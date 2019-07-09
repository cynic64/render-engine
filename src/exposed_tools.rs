use super::*;
pub use winit::Event;
pub use winit::KeyboardInput;
pub use winit::VirtualKeyCode;

pub const CURSOR_RESET_POS_X: u32 = 50;
pub const CURSOR_RESET_POS_Y: u32 = 50;

pub type VertexBuffer = CpuAccessibleBuffer<[Vertex]>;

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

#[derive(Debug, Clone)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
}

pub struct PixelCoord {
    pub x: u32,
    pub y: u32,
}

pub fn get_elapsed(start: std::time::Instant) -> f32 {
    start.elapsed().as_secs() as f32 + start.elapsed().subsec_nanos() as f32 / 1_000_000_000.0
}

pub struct Square {
    pub corner: PixelCoord,
    pub size: u32,
    pub color: [f32; 4],
}

impl Square {
    pub fn create_vertices(&self, app: &app::App) -> Vec<Vertex> {
        let points = [
            PixelCoord {
                x: self.corner.x,
                y: self.corner.y,
            },
            PixelCoord {
                x: self.corner.x + self.size,
                y: self.corner.y,
            },
            PixelCoord {
                x: self.corner.x,
                y: self.corner.y + self.size,
            },
            PixelCoord {
                x: self.corner.x + self.size,
                y: self.corner.y + self.size,
            },
        ];

        let indices = [0, 1, 2, 1, 2, 3];

        indices
            .iter()
            .map(|&idx| app.vert_from_pixel_coords(&points[idx], self.color))
            .collect()
    }
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

// traits
pub trait BasicCamera {
    fn get_view_matrix(&self) -> [[f32; 4]; 4];
}

pub trait InputHandlingCamera {
    fn get_view_matrix(&self) -> [[f32; 4]; 4];
    fn handle_input(&mut self, events: &[Event], keys_down: &KeysDown, delta: f32);
}
