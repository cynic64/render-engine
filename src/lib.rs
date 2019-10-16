// TODO: make tests for this whole crate

pub mod system;

pub mod camera;
pub use camera::{FlyCamera, OrbitCamera, OrthoCamera};

// TODO: organize this whole file better
pub mod producer;

pub mod pipeline_cache;
pub mod collection_cache;

pub mod input;

pub mod utils;

pub mod window;

pub mod shaders;

pub mod render_passes;

pub use vulkano::pipeline::input_assembly::PrimitiveTopology;

pub use vulkano::impl_vertex;
