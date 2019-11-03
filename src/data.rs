use vulkano::device::Queue;
use vulkano::pipeline::GraphicsPipelineAbstract;
use vulkano::descriptor::descriptor_set::{DescriptorSet, PersistentDescriptorSet};

use crate::utils::bufferize_data;

use std::sync::Arc;

pub trait DataAbstract {
    fn create_sets(&self, queue: Arc<Queue>, pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>) -> Vec<Arc<dyn DescriptorSet + Send + Sync>>;
}

impl<T: Usable> DataAbstract for T {
    fn create_sets(&self, queue: Arc<Queue>, pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>) -> Vec<Arc<dyn DescriptorSet + Send + Sync>> {
        let buffer = bufferize_data(queue, self.clone());
        let set = Arc::new(
            PersistentDescriptorSet::start(pipeline, 0)
                .add_buffer(buffer)
                .unwrap()
                .build()
                .unwrap()
        );
        vec![set]
    }
}

// TODO: more specific name
pub trait Usable: Send + Sync + Clone + 'static {}

impl<T: Send + Sync + Clone + 'static> Usable for T {}

pub struct NoData {}

impl DataAbstract for NoData {
    fn create_sets(&self, _queue: Arc<Queue>, _pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>) -> Vec<Arc<dyn DescriptorSet + Send + Sync>> {
        vec![]
    }
}
