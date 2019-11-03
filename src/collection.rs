/*
There are 2 forms of a collection: one with just the cpu-side data, and a
version with real sets uploaded to the GPU.
We need to make sure the type of each is very clear and will match the shaders.

User defines collection layout with type, say
(
    (
        ModelMatrix,
        Camera,
    ),
    (
        Material
    ),
)

This then gets converted to a real collection when drawing, which requires the
pipeline and queue.
 */
use vulkano::device::Queue;
use vulkano::pipeline::GraphicsPipelineAbstract;
use vulkano::descriptor::DescriptorSet;

use std::sync::Arc;

pub trait Collection {
    fn convert(
        &self,
        queue: Arc<Queue>,
        pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    ) -> Vec<Arc<dyn DescriptorSet + Send + Sync>>;
}

impl Collection for () {
    fn convert(
        &self,
        _queue: Arc<Queue>,
        _pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    ) -> Vec<Arc<dyn DescriptorSet + Send + Sync>> {
        vec![]
    }
}
