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

use crate::utils::{bufferize_data, default_sampler};

use std::sync::Arc;

// TODO: tests for all this crap

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

impl<T1: Set, T2: Set, T3: Set> Collection for (T1, T2, T3) {
    fn convert(
        &self,
        queue: Arc<Queue>,
        pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    ) -> Vec<Arc<dyn DescriptorSet + Send + Sync>> {
        vec![
            self.0.upload(queue.clone(), pipeline.clone()),
            self.1.upload(queue.clone(), pipeline.clone()),
            self.2.upload(queue.clone(), pipeline.clone()),
        ]
    }
}

impl<T1: Set, T2: Set, T3: Set, T4: Set> Collection for (T1, T2, T3, T4) {
    fn convert(
        &self,
        queue: Arc<Queue>,
        pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    ) -> Vec<Arc<dyn DescriptorSet + Send + Sync>> {
        vec![
            self.0.upload(queue.clone(), pipeline.clone()),
            self.1.upload(queue.clone(), pipeline.clone()),
            self.2.upload(queue.clone(), pipeline.clone()),
            self.3.upload(queue.clone(), pipeline.clone()),
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

// length 1
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

impl Set for (Image,) {
    fn upload(
        &self,
        queue: Arc<Queue>,
        pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    ) -> Arc<dyn DescriptorSet + Send + Sync> {
        let sampler = default_sampler(queue.device().clone());
        Arc::new(
            PersistentDescriptorSet::start(pipeline, 0)
                .add_sampled_image(self.0.clone(), sampler)
                .unwrap()
                .build()
                .unwrap()
        )
    }
}

// length 2
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

impl<T: Data> Set for (Image, T) {
    fn upload(
        &self,
        queue: Arc<Queue>,
        pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    ) -> Arc<dyn DescriptorSet + Send + Sync> {
        let sampler = default_sampler(queue.device().clone());
        let buffer2 = bufferize_data(queue.clone(), self.1.clone());

        Arc::new(
            PersistentDescriptorSet::start(pipeline, 0)
                .add_sampled_image(self.0.clone(), sampler)
                .unwrap()
                .add_buffer(buffer2)
                .unwrap()
                .build()
                .unwrap()
        )
    }
}

impl<T: Data> Set for (T, Image) {
    fn upload(
        &self,
        queue: Arc<Queue>,
        pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    ) -> Arc<dyn DescriptorSet + Send + Sync> {
        let sampler = default_sampler(queue.device().clone());
        let buffer1 = bufferize_data(queue.clone(), self.0.clone());

        Arc::new(
            PersistentDescriptorSet::start(pipeline, 0)
                .add_sampled_image(self.1.clone(), sampler)
                .unwrap()
                .add_buffer(buffer1)
                .unwrap()
                .build()
                .unwrap()
        )
    }
}

impl Set for (Image, Image) {
    fn upload(
        &self,
        queue: Arc<Queue>,
        pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    ) -> Arc<dyn DescriptorSet + Send + Sync> {
        let sampler = default_sampler(queue.device().clone());

        Arc::new(
            PersistentDescriptorSet::start(pipeline, 0)
                .add_sampled_image(self.0.clone(), sampler.clone())
                .unwrap()
                .add_sampled_image(self.1.clone(), sampler)
                .unwrap()
                .build()
                .unwrap()
        )
    }
}

// length 3
impl<T1: Data, T2: Data, T3: Data> Set for (T1, T2, T3) {
    fn upload(
        &self,
        queue: Arc<Queue>,
        pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    ) -> Arc<dyn DescriptorSet + Send + Sync> {
        let buffer1 = bufferize_data(queue.clone(), self.0.clone());
        let buffer2 = bufferize_data(queue.clone(), self.1.clone());
        let buffer3 = bufferize_data(queue.clone(), self.2.clone());

        Arc::new(
            PersistentDescriptorSet::start(pipeline, 0)
                .add_buffer(buffer1)
                .unwrap()
                .add_buffer(buffer2)
                .unwrap()
                .add_buffer(buffer3)
                .unwrap()
                .build()
                .unwrap()
        )
    }
}

impl<T1: Data, T2: Data> Set for (Image, T1, T2) {
    fn upload(
        &self,
        queue: Arc<Queue>,
        pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    ) -> Arc<dyn DescriptorSet + Send + Sync> {
        let sampler = default_sampler(queue.device().clone());
        let buffer2 = bufferize_data(queue.clone(), self.1.clone());
        let buffer3 = bufferize_data(queue.clone(), self.2.clone());

        Arc::new(
            PersistentDescriptorSet::start(pipeline, 0)
                .add_sampled_image(self.0.clone(), sampler)
                .unwrap()
                .add_buffer(buffer2)
                .unwrap()
                .add_buffer(buffer3)
                .unwrap()
                .build()
                .unwrap()
        )
    }
}

impl<T1: Data, T2: Data> Set for (T1, Image, T2) {
    fn upload(
        &self,
        queue: Arc<Queue>,
        pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    ) -> Arc<dyn DescriptorSet + Send + Sync> {
        let sampler = default_sampler(queue.device().clone());
        let buffer1 = bufferize_data(queue.clone(), self.0.clone());
        let buffer3 = bufferize_data(queue.clone(), self.2.clone());

        Arc::new(
            PersistentDescriptorSet::start(pipeline, 0)
                .add_buffer(buffer1)
                .unwrap()
                .add_sampled_image(self.1.clone(), sampler)
                .unwrap()
                .add_buffer(buffer3)
                .unwrap()
                .build()
                .unwrap()
        )
    }
}

impl<T1: Data, T2: Data> Set for (T1, T2, Image) {
    fn upload(
        &self,
        queue: Arc<Queue>,
        pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    ) -> Arc<dyn DescriptorSet + Send + Sync> {
        let sampler = default_sampler(queue.device().clone());
        let buffer1 = bufferize_data(queue.clone(), self.0.clone());
        let buffer2 = bufferize_data(queue.clone(), self.1.clone());

        Arc::new(
            PersistentDescriptorSet::start(pipeline, 0)
                .add_buffer(buffer1)
                .unwrap()
                .add_buffer(buffer2)
                .unwrap()
                .add_sampled_image(self.2.clone(), sampler)
                .unwrap()
                .build()
                .unwrap()
        )
    }
}

impl<T: Data> Set for (T, Image, Image) {
    fn upload(
        &self,
        queue: Arc<Queue>,
        pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    ) -> Arc<dyn DescriptorSet + Send + Sync> {
        let sampler = default_sampler(queue.device().clone());
        let buffer1 = bufferize_data(queue.clone(), self.0.clone());

        Arc::new(
            PersistentDescriptorSet::start(pipeline, 0)
                .add_buffer(buffer1)
                .unwrap()
                .add_sampled_image(self.1.clone(), sampler.clone())
                .unwrap()
                .add_sampled_image(self.2.clone(), sampler)
                .unwrap()
                .build()
                .unwrap()
        )
    }
}

impl<T: Data> Set for (Image, T, Image) {
    fn upload(
        &self,
        queue: Arc<Queue>,
        pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    ) -> Arc<dyn DescriptorSet + Send + Sync> {
        let sampler = default_sampler(queue.device().clone());
        let buffer2 = bufferize_data(queue.clone(), self.1.clone());

        Arc::new(
            PersistentDescriptorSet::start(pipeline, 0)
                .add_sampled_image(self.0.clone(), sampler.clone())
                .unwrap()
                .add_buffer(buffer2)
                .unwrap()
                .add_sampled_image(self.2.clone(), sampler)
                .unwrap()
                .build()
                .unwrap()
        )
    }
}

impl<T: Data> Set for (Image, Image, T) {
    fn upload(
        &self,
        queue: Arc<Queue>,
        pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    ) -> Arc<dyn DescriptorSet + Send + Sync> {
        let sampler = default_sampler(queue.device().clone());
        let buffer3 = bufferize_data(queue.clone(), self.2.clone());

        Arc::new(
            PersistentDescriptorSet::start(pipeline, 0)
                .add_sampled_image(self.0.clone(), sampler.clone())
                .unwrap()
                .add_sampled_image(self.1.clone(), sampler)
                .unwrap()
                .add_buffer(buffer3)
                .unwrap()
                .build()
                .unwrap()
        )
    }
}

impl Set for (Image, Image, Image) {
    fn upload(
        &self,
        queue: Arc<Queue>,
        pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    ) -> Arc<dyn DescriptorSet + Send + Sync> {
        let sampler = default_sampler(queue.device().clone());

        Arc::new(
            PersistentDescriptorSet::start(pipeline, 0)
                .add_sampled_image(self.0.clone(), sampler.clone())
                .unwrap()
                .add_sampled_image(self.1.clone(), sampler.clone())
                .unwrap()
                .add_sampled_image(self.2.clone(), sampler)
                .unwrap()
                .build()
                .unwrap()
        )
    }
}

// length 4 will be FUN!

pub type Image = Arc<dyn ImageViewAccess + Send + Sync>;

pub trait Data: Send + Sync + Clone + 'static {}
