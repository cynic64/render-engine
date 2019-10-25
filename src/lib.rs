// TODO: make tests for this whole crate

pub mod system;

// pub mod camera;
// pub use camera::{FlyCamera, OrbitCamera, OrthoCamera};

pub mod collection_cache;
pub mod pipeline_cache;

pub mod input;

pub mod mesh;

pub mod utils;

pub mod window;

pub mod shaders;

pub mod render_passes;

// re-exports of vulkano's stuff
use std::sync::Arc;
pub type RenderPass = Arc<dyn vulkano::framebuffer::RenderPassAbstract + Send + Sync>;
pub type Pipeline = Arc<dyn vulkano::pipeline::GraphicsPipelineAbstract + Send + Sync>;
pub type Device = Arc<vulkano::device::Device>;
pub type Queue = Arc<vulkano::device::Queue>;
pub type Buffer = Arc<dyn vulkano::buffer::BufferAccess + Send + Sync>;
pub type Image = Arc<dyn vulkano::image::ImageViewAccess + Send + Sync>;
pub type Set = Arc<dyn vulkano::descriptor::descriptor_set::DescriptorSet + Send + Sync>;
pub use vulkano::format::Format;
