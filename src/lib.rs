// TODO: make tests for this whole crate

pub mod system;

pub mod app;
pub use app::App;
pub mod camera;
pub use camera::{FlyCamera, OrbitCamera, OrthoCamera};

pub mod template_systems;

// TODO: organize this whole file better
pub mod producer;

pub mod input;

pub mod shaders;

pub mod render_passes;

pub mod world;
pub use world::ObjectSpec;
pub use world::ObjectSpecBuilder;
pub use world::World;
pub use world::WorldCommunicator;

pub use vulkano::pipeline::input_assembly::PrimitiveTopology;

pub mod mesh_gen;
