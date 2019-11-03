use vulkano::device::Queue;
use vulkano::pipeline::GraphicsPipelineAbstract;
use vulkano::descriptor::descriptor_set::{DescriptorSet, PersistentDescriptorSet};

use crate::utils::bufferize_data;

use std::sync::Arc;

pub trait DataAbstract {
    fn create_sets(&self, queue: Arc<Queue>, pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>) -> Vec<Arc<dyn DescriptorSet + Send + Sync>>;
}

pub struct Data<T: Send + Sync + Clone + 'static> {
    pub data: T,
}

impl<T: Send + Sync + Clone + 'static> DataAbstract for Data<T> {
    fn create_sets(&self, queue: Arc<Queue>, pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>) -> Vec<Arc<dyn DescriptorSet + Send + Sync>> {
        let buffer = bufferize_data(queue, self.data.clone());
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

pub struct NoData {}

impl DataAbstract for NoData {
    fn create_sets(&self, _queue: Arc<Queue>, _pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>) -> Vec<Arc<dyn DescriptorSet + Send + Sync>> {
        vec![]
    }
}
