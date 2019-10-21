use vulkano::descriptor::DescriptorSet;
use vulkano::device::{Device, Queue};
use vulkano::pipeline::input_assembly::PrimitiveTopology;
use vulkano::buffer::{ImmutableBuffer, BufferAccess};
use vulkano::framebuffer::{RenderPassAbstract, Subpass};
use vulkano::pipeline::{GraphicsPipelineAbstract, GraphicsPipeline};

use crate::pipeline_cache::PipelineSpec;
use crate::system::RenderableObject;
use crate::utils::bufferize_slice;
use crate::shaders::ShaderSystem;

use std::path::PathBuf;
use std::sync::Arc;
use std::marker::PhantomData;
use std::any::Any;

#[derive(Default, Debug, Clone, Copy)]
pub struct Dummy {}
vulkano::impl_vertex!(Dummy);

pub struct ObjectSpec<V: Vertex> {
    pub vs_path: PathBuf,
    pub fs_path: PathBuf,
    pub mesh: Mesh<V>,
    pub custom_sets: Vec<Arc<dyn DescriptorSet + Send + Sync>>,
    pub depth_buffer: bool,
    pub fill_type: PrimitiveTopology,
}

impl<V: Vertex> ObjectSpec<V> {
    pub fn build(self, queue: Arc<Queue>) -> RenderableObject {
        let pipeline_spec = PipelineSpec {
            vs_path: self.vs_path,
            fs_path: self.fs_path,
            fill_type: self.fill_type,
            depth: self.depth_buffer,
            vtype: self.mesh.get_vtype(),
        };

        let vbuf = self.mesh.get_vbuf(queue.clone());
        let ibuf = self.mesh.get_ibuf(queue.clone());

        RenderableObject {
            pipeline_spec,
            vbuf,
            ibuf,
            custom_sets: self.custom_sets,
        }
    }
}

impl<V: Vertex> Default for ObjectSpec<V> {
    fn default() -> Self {
        let vertices: Vec<V> = vec![];

        Self {
            vs_path: relative_path("shaders/forward/default_vert.glsl"),
            fs_path: relative_path("shaders/forward/default_frag.glsl"),
            mesh: Mesh {
                vertices,
                indices: vec![],
            },
            custom_sets: vec![],
            depth_buffer: false,
            fill_type: PrimitiveTopology::TriangleList,
        }
    }
}

// TODO: instead of having arc<dyn vertexlist>, give mesh a type parameter and
// create a MeshAbstract type.
#[derive(Clone)]
pub struct Mesh<V: Vertex> {
    pub vertices: Vec<V>,
    pub indices: Vec<u32>,
}

pub trait Vertex: vulkano::pipeline::vertex::Vertex + Clone {}

impl<V: vulkano::pipeline::vertex::Vertex + Clone> Vertex for V {}

pub trait MeshAbstract {
    fn get_vbuf(&self, queue: Arc<Queue>) -> Arc<dyn BufferAccess + Send + Sync>;
    fn get_ibuf(&self, queue: Arc<Queue>) -> Arc<ImmutableBuffer<[u32]>>;
    fn get_vtype(&self) -> Arc<dyn VertexTypeAbstract>;
}

impl<V: Vertex> MeshAbstract for Mesh<V> {
    fn get_vbuf(&self, queue: Arc<Queue>) -> Arc<dyn BufferAccess + Send + Sync> {
        bufferize_slice(queue, &self.vertices)
    }

    fn get_ibuf(&self, queue: Arc<Queue>) -> Arc<ImmutableBuffer<[u32]>> {
        bufferize_slice(queue, &self.indices)
    }

    fn get_vtype(&self) -> Arc<dyn VertexTypeAbstract> {
        Arc::new(VertexType {
            phantom: PhantomData::<V>,
        })
    }
}

#[derive(Clone)]
pub struct VertexType<V: Vertex + Send + Sync + Clone> {
    pub phantom: PhantomData<V>,
}

// TODO: properly implement clone and partialeq
pub trait VertexTypeAbstract: Any {
    fn create_pipeline(
        &self,
        device: Arc<Device>,
        shaders: ShaderSystem,
        fill_type: PrimitiveTopology,
        render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
        depth: bool,
    ) -> Arc<dyn GraphicsPipelineAbstract + Send + Sync>;

    fn clone(&self) -> Arc<dyn VertexTypeAbstract>;
}

impl<V: Vertex + Send + Sync + Clone + 'static> VertexTypeAbstract for VertexType<V> {
    fn create_pipeline(
        &self,
        device: Arc<Device>,
        shaders: ShaderSystem,
        fill_type: PrimitiveTopology,
        render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
        depth: bool,
    ) -> Arc<dyn GraphicsPipelineAbstract + Send + Sync> {
        let (vs_main, fs_main) = shaders.get_entry_points();

        if depth {
            Arc::new(
                GraphicsPipeline::start()
                    .vertex_input_single_buffer::<V>()
                    .vertex_shader(vs_main, ())
                    .primitive_topology(fill_type)
                    .viewports_dynamic_scissors_irrelevant(1)
                    .fragment_shader(fs_main, ())
                    .render_pass(Subpass::from(render_pass, 0).unwrap())
                    .depth_stencil_simple_depth()
                    .cull_mode_back()
                    .build(device)
                    .unwrap()
            )
        } else {
            Arc::new(
                GraphicsPipeline::start()
                    .vertex_input_single_buffer::<V>()
                    .vertex_shader(vs_main, ())
                    .primitive_topology(fill_type)
                    .viewports_dynamic_scissors_irrelevant(1)
                    .fragment_shader(fs_main, ())
                    .render_pass(Subpass::from(render_pass, 0).unwrap())
                    .cull_mode_back()
                    .build(device)
                    .unwrap()
            )
        }
    }

    fn clone(&self) -> Arc<dyn VertexTypeAbstract> {
        Arc::new(
            Self {
                phantom: PhantomData::<V>
            }
        )
    }
}

fn relative_path(local_path: &str) -> PathBuf {
    [env!("CARGO_MANIFEST_DIR"), local_path].iter().collect()
}
