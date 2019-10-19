use vulkano::descriptor::DescriptorSet;
use vulkano::device::Queue;
use vulkano::pipeline::input_assembly::PrimitiveTopology;

use crate::pipeline_cache::PipelineSpec;
use crate::system::RenderableObject;
use crate::utils::bufferize_slice;

use std::path::PathBuf;
use std::sync::Arc;

#[derive(Default, Debug, Clone)]
pub struct SimpleVertex {
    pub position: [f32; 2],
}
vulkano::impl_vertex!(SimpleVertex, position);

// TODO: maybe rename to vertex3D?
#[derive(Default, Debug, Clone)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coord: [f32; 2],
}
vulkano::impl_vertex!(Vertex, position, normal, tex_coord);

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
