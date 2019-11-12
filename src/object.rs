use vulkano::buffer::{BufferAccess, ImmutableBuffer};
use vulkano::command_buffer::DynamicState;
use vulkano::descriptor::DescriptorSet;
use vulkano::device::Queue;
use vulkano::pipeline::input_assembly::PrimitiveTopology;
use vulkano::pipeline::GraphicsPipelineAbstract;

use crate::collection::Collection;
use crate::mesh::{Mesh, MeshAbstract, Vertex, VertexType};
use crate::pipeline_cache::PipelineSpec;

use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone)]
pub struct Object<C: Collection> {
    pub pipeline_spec: PipelineSpec,
    pub vbuf: Arc<dyn BufferAccess + Send + Sync>,
    pub ibuf: Arc<ImmutableBuffer<[u32]>>,
    pub collection: C,
    pub custom_dynamic_state: Option<DynamicState>,
}

pub trait Drawcall {
    fn pipe_spec(&self) -> &PipelineSpec;
    fn vbuf(&self) -> Arc<dyn BufferAccess + Send + Sync>;
    fn ibuf(&self) -> Arc<ImmutableBuffer<[u32]>>;
    fn collection(
        &self,
        queue: Arc<Queue>,
        pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
        set_idx_offset: usize,
    ) -> Vec<Arc<dyn DescriptorSet + Send + Sync>>;
    fn custom_dynstate(&self) -> Option<DynamicState>;
}

impl<C: Collection> Drawcall for Object<C> {
    fn pipe_spec(&self) -> &PipelineSpec {
        &self.pipeline_spec
    }

    fn vbuf(&self) -> Arc<dyn BufferAccess + Send + Sync> {
        self.vbuf.clone()
    }

    fn ibuf(&self) -> Arc<ImmutableBuffer<[u32]>> {
        self.ibuf.clone()
    }

    fn collection(
        &self,
        queue: Arc<Queue>,
        pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
        set_idx_offset: usize,
    ) -> Vec<Arc<dyn DescriptorSet + Send + Sync>> {
        self.collection.convert(queue, pipeline, set_idx_offset)
    }

    fn custom_dynstate(&self) -> Option<DynamicState> {
        self.custom_dynamic_state.clone()
    }
}

#[derive(Clone)]
pub struct ObjectPrototype<V: Vertex, C: Collection> {
    pub vs_path: PathBuf,
    pub fs_path: PathBuf,
    pub fill_type: PrimitiveTopology,
    pub read_depth: bool,
    pub write_depth: bool,
    pub mesh: Mesh<V>,
    pub collection: C,
    pub custom_dynamic_state: Option<DynamicState>,
}

impl<V: Vertex, C: Collection + 'static> ObjectPrototype<V, C> {
    pub fn build(self, queue: Arc<Queue>) -> Object<C> {
        let vbuf = self.mesh.get_vbuf(queue.clone());
        let ibuf = self.mesh.get_ibuf(queue.clone());

        Object {
            pipeline_spec: PipelineSpec {
                vs_path: self.vs_path,
                fs_path: self.fs_path,
                fill_type: self.fill_type,
                read_depth: self.read_depth,
                write_depth: self.write_depth,
                vtype: VertexType::<V>::new(),
            },
            vbuf,
            ibuf,
            collection: self.collection,
            custom_dynamic_state: self.custom_dynamic_state,
        }
    }
}
