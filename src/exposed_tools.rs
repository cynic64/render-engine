use super::*;
pub use winit::VirtualKeyCode;

pub type VertexBuffer = CpuAccessibleBuffer<[Vertex]>;

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