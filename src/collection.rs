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
use vulkano::descriptor::descriptor_set::{DescriptorSet, PersistentDescriptorSet};
use vulkano::image::ImageViewAccess;

use crate::utils::bufferize_data;

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

impl<T: Set> Collection for (T,) {
    fn convert(
        &self,
        queue: Arc<Queue>,
        pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    ) -> Vec<Arc<dyn DescriptorSet + Send + Sync>> {
        vec![
            self.0.upload(queue, pipeline)
        ]
    }
}

impl<T1: Set, T2: Set> Collection for (T1, T2) {
    fn convert(
        &self,
        queue: Arc<Queue>,
        pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    ) -> Vec<Arc<dyn DescriptorSet + Send + Sync>> {
        vec![
            self.0.upload(queue.clone(), pipeline.clone()),
            self.1.upload(queue.clone(), pipeline.clone()),
        ]
    }
}

/*
Set
 */

pub trait Set {
    fn upload(
        &self,
        queue: Arc<Queue>,
        pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    ) -> Arc<dyn DescriptorSet + Send + Sync>;
}

impl<T: Data> Set for (T,) {
    fn upload(
        &self,
        queue: Arc<Queue>,
        pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    ) -> Arc<dyn DescriptorSet + Send + Sync> {
        let buffer = bufferize_data(queue.clone(), self.0.clone());

        Arc::new(
            PersistentDescriptorSet::start(pipeline, 0)
                .add_buffer(buffer)
                .unwrap()
                .build()
                .unwrap()
        )
    }
}

/*
impl Set for (Image,) {
    fn upload(
        &self,
        queue: Arc<Queue>,
        pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    ) -> Arc<dyn DescriptorSet + Send + Sync> {
        Arc::new(
            PersistentDescriptorSet::start(pipeline, 0)
                .add_buffer(buffer)
                .unwrap()
                .build()
                .unwrap()
        )
    }
}
*/

impl<T1: Data, T2: Data> Set for (T1, T2) {
    fn upload(
        &self,
        queue: Arc<Queue>,
        pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    ) -> Arc<dyn DescriptorSet + Send + Sync> {
        let buffer1 = bufferize_data(queue.clone(), self.0.clone());
        let buffer2 = bufferize_data(queue.clone(), self.1.clone());

        Arc::new(
            PersistentDescriptorSet::start(pipeline, 0)
                .add_buffer(buffer1)
                .unwrap()
                .add_buffer(buffer2)
                .unwrap()
                .build()
                .unwrap()
        )
    }
}

/*
pub type Image = Arc<dyn ImageViewAccess + Send + Sync>;
*/

pub trait Data: Send + Sync + Clone + 'static {}
