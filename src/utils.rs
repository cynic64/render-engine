use vulkano::buffer::{BufferUsage, ImmutableBuffer};
use vulkano::descriptor::DescriptorSet;
use vulkano::device::Queue;
use vulkano::format::Format;
use vulkano::image::{Dimensions, ImageViewAccess, ImmutableImage};
use vulkano::memory::Content;
use vulkano::pipeline::input_assembly::PrimitiveTopology;
use vulkano::sync::GpuFuture;

use image::ImageFormat;

use crate::pipeline_cache::PipelineSpec;
use crate::system::{RenderableObject, Vertex};

use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub fn bufferize_slice<T: Content + 'static + Send + Sync + Clone>(
    queue: Arc<Queue>,
    slice: &[T],
) -> Arc<ImmutableBuffer<[T]>> {
    ImmutableBuffer::from_iter(slice.iter().cloned(), BufferUsage::all(), queue)
        .unwrap()
        .0
}

pub fn bufferize_data<T: Content + 'static + Send + Sync>(
    queue: Arc<Queue>,
    data: T,
) -> Arc<ImmutableBuffer<T>> {
    ImmutableBuffer::from_data(data, BufferUsage::all(), queue)
        .unwrap()
        .0
}

pub fn load_texture(queue: Arc<Queue>, path: &Path) -> Arc<dyn ImageViewAccess + Send + Sync> {
    let (texture, tex_future) = {
        let image = image::load_from_memory_with_format(
            &File::open(path)
                .unwrap()
                .bytes()
                .map(|x| x.unwrap())
                .collect::<Vec<u8>>(),
            ImageFormat::PNG,
        )
        .unwrap()
        .to_rgba();
        let image_data = image.into_raw().clone();

        ImmutableImage::from_iter(
            image_data.iter().cloned(),
            Dimensions::Dim2d {
                width: 1024,
                height: 1024,
            },
            Format::R8G8B8A8Srgb,
            queue.clone(),
        )
        .unwrap()
    };

    tex_future
        .then_signal_fence_and_flush()
        .unwrap()
        .wait(None)
        .unwrap();

    texture
}

// TODO: move this to its own file, bufferize too
pub struct ObjectSpec {
    pub vs_path: PathBuf,
    pub fs_path: PathBuf,
    pub mesh: Mesh,
    pub custom_set: Option<Arc<dyn DescriptorSet + Send + Sync>>,
    pub depth_buffer: bool,
    pub fill_type: PrimitiveTopology,
}

impl ObjectSpec {
    pub fn build(self, queue: Arc<Queue>) -> RenderableObject {
        let pipeline_spec = PipelineSpec {
            vs_path: self.vs_path,
            fs_path: self.fs_path,
            fill_type: self.fill_type,
            depth: self.depth_buffer,
        };

        let vbuf = bufferize_slice(queue.clone(), &self.mesh.vertices);
        let ibuf = bufferize_slice(queue.clone(), &self.mesh.indices);

        RenderableObject {
            pipeline_spec,
            vbuf,
            ibuf,
            custom_set: self.custom_set,
        }
    }
}

impl Default for ObjectSpec {
    fn default() -> Self {
        Self {
            vs_path: relative_path("shaders/forward/default_vert.glsl"),
            fs_path: relative_path("shaders/forward/default_frag.glsl"),
            mesh: Mesh {
                vertices: vec![],
                indices: vec![],
            },
            custom_set: None,
            depth_buffer: false,
            fill_type: PrimitiveTopology::TriangleList,
        }
    }
}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

fn relative_path(local_path: &str) -> PathBuf {
    [env!("CARGO_MANIFEST_DIR"), local_path].iter().collect()
}
