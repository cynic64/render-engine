pub use vulkano::pipeline::input_assembly::PrimitiveTopology;
pub use vulkano::impl_vertex;

use vulkano::device::{Device, Queue};
use vulkano::buffer::{ImmutableBuffer, BufferAccess};
use vulkano::framebuffer::{RenderPassAbstract, Subpass};
use vulkano::pipeline::{GraphicsPipelineAbstract, GraphicsPipeline};
use vulkano::pipeline::depth_stencil::{DepthStencil, Compare};

use crate::utils::immutable_slice;
use crate::shaders::ShaderSystem;

use std::sync::Arc;
use std::marker::PhantomData;
use std::any::Any;

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
        immutable_slice(queue, &self.vertices)
    }

    fn get_ibuf(&self, queue: Arc<Queue>) -> Arc<ImmutableBuffer<[u32]>> {
        immutable_slice(queue, &self.indices)
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

impl<V: Vertex + Send + Sync + Clone> VertexType<V> {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            phantom: PhantomData::<V>,
        })
    }
}

// TODO: properly implement clone and partialeq
pub trait VertexTypeAbstract: Any {
    fn create_pipeline(
        &self,
        device: Arc<Device>,
        shaders: ShaderSystem,
        fill_type: PrimitiveTopology,
        render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
        read_depth: bool,
        write_depth: bool,
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
        read_depth: bool,
        write_depth: bool,
    ) -> Arc<dyn GraphicsPipelineAbstract + Send + Sync> {
        let (vs_main, fs_main) = shaders.get_entry_points();

        if !read_depth && !write_depth {
            // no depth buffer at all
            Arc::new(
                GraphicsPipeline::start()
                    .vertex_input_single_buffer::<V>()
                    .vertex_shader(vs_main, ())
                    .primitive_topology(fill_type)
                    .viewports_dynamic_scissors_irrelevant(1)
                    .fragment_shader(fs_main, ())
                    .render_pass(Subpass::from(render_pass, 0).unwrap())
                    .build(device)
                    .unwrap()
            )
        } else {
            let mut stencil = DepthStencil::disabled();
            stencil.depth_compare = if read_depth {
                Compare::LessOrEqual
            } else {
                Compare::Always
            };
            stencil.depth_write = write_depth;

            Arc::new(
                GraphicsPipeline::start()
                    .vertex_input_single_buffer::<V>()
                    .vertex_shader(vs_main, ())
                    .primitive_topology(fill_type)
                    .viewports_dynamic_scissors_irrelevant(1)
                    .fragment_shader(fs_main, ())
                    .depth_stencil(stencil)
                    .render_pass(Subpass::from(render_pass, 0).unwrap())
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
