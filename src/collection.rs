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
// TODO: convert immediately when the user creates the collection, because this
// means that if it panics it will panic there and not in render_frame, where it
// is harder to track down.

#[derive(Clone)]
pub struct Collection<T: CollectionUpload> {
    pub data: T,
    gpu_data: Option<Vec<Arc<dyn DescriptorSet + Send + Sync>>>,
    pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    set_idx_offset: usize,
}

impl<T: CollectionUpload> Collection<T> {
    pub fn new(data: T, pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>, set_idx_offset: usize) -> Self {
        Self {
            data,
            gpu_data: None,
            pipeline,
            set_idx_offset,
        }
    }

    pub fn upload(&mut self, queue: Arc<Queue>) {
        self.gpu_data = Some(self.data.convert(queue, self.pipeline.clone(), self.set_idx_offset));
    }

    pub fn get(&self) -> Vec<Arc<dyn DescriptorSet + Send + Sync>> {
        self.gpu_data.as_ref().expect("Collection not uploaded").clone()
    }
}

pub trait CollectionUpload {
    fn convert(
        &self,
        queue: Arc<Queue>,
        pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
        set_idx_offset: usize,
    ) -> Vec<Arc<dyn DescriptorSet + Send + Sync>>;
}

impl CollectionUpload for () {
    fn convert(
        &self,
        _queue: Arc<Queue>,
        _pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
        _set_idx_offset: usize,
    ) -> Vec<Arc<dyn DescriptorSet + Send + Sync>> {
        vec![]
    }
}

impl<T: Set> CollectionUpload for (T,) {
    fn convert(
        &self,
        queue: Arc<Queue>,
        pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
        set_idx_offset: usize,
    ) -> Vec<Arc<dyn DescriptorSet + Send + Sync>> {
        vec![
            self.0.upload(queue, pipeline, set_idx_offset)
        ]
    }
}

impl<T1: Set, T2: Set> CollectionUpload for (T1, T2) {
    fn convert(
        &self,
        queue: Arc<Queue>,
        pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
        set_idx_offset: usize,
    ) -> Vec<Arc<dyn DescriptorSet + Send + Sync>> {
        vec![
            self.0.upload(queue.clone(), pipeline.clone(), set_idx_offset),
            self.1.upload(queue.clone(), pipeline.clone(), set_idx_offset + 1),
        ]
    }
}

impl<T1: Set, T2: Set, T3: Set> CollectionUpload for (T1, T2, T3) {
    fn convert(
        &self,
        queue: Arc<Queue>,
        pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
        set_idx_offset: usize,
    ) -> Vec<Arc<dyn DescriptorSet + Send + Sync>> {
        vec![
            self.0.upload(queue.clone(), pipeline.clone(), set_idx_offset),
            self.1.upload(queue.clone(), pipeline.clone(), set_idx_offset + 1),
            self.2.upload(queue.clone(), pipeline.clone(), set_idx_offset + 2),
        ]
    }
}

impl<T1: Set, T2: Set, T3: Set, T4: Set> CollectionUpload for (T1, T2, T3, T4) {
    fn convert(
        &self,
        queue: Arc<Queue>,
        pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
        set_idx_offset: usize,
    ) -> Vec<Arc<dyn DescriptorSet + Send + Sync>> {
        vec![
            self.0.upload(queue.clone(), pipeline.clone(), set_idx_offset),
            self.1.upload(queue.clone(), pipeline.clone(), set_idx_offset + 1),
            self.2.upload(queue.clone(), pipeline.clone(), set_idx_offset + 2),
            self.3.upload(queue.clone(), pipeline.clone(), set_idx_offset + 3),
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
        set_idx: usize,
    ) -> Arc<dyn DescriptorSet + Send + Sync>;
}

// length 1
impl<T: Data> Set for (T,) {
    fn upload(
        &self,
        queue: Arc<Queue>,
        pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
        set_idx: usize,
    ) -> Arc<dyn DescriptorSet + Send + Sync> {
        let buffer = bufferize_data(queue.clone(), self.0.clone());

        Arc::new(
            PersistentDescriptorSet::start(pipeline, set_idx)
                .add_buffer(buffer)
                .expect(&format!("Panic adding 1st buffer at set idx {}", set_idx))
                .build()
                .expect(&format!("Panic finalizing set at set idx {}", set_idx))
        )
    }
}

impl Set for (Image,) {
    fn upload(
        &self,
        queue: Arc<Queue>,
        pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
        set_idx: usize,
    ) -> Arc<dyn DescriptorSet + Send + Sync> {
        let sampler = default_sampler(queue.device().clone());
        Arc::new(
            PersistentDescriptorSet::start(pipeline, set_idx)
                .add_sampled_image(self.0.clone(), sampler)
                .expect(&format!("Panic adding 1st image at set idx {}", set_idx))
                .build()
                .expect(&format!("Panic finalizing set at set idx {}", set_idx))
        )
    }
}

// length 2
impl<T1: Data, T2: Data> Set for (T1, T2) {
    fn upload(
        &self,
        queue: Arc<Queue>,
        pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
        set_idx: usize,
    ) -> Arc<dyn DescriptorSet + Send + Sync> {
        let buffer1 = bufferize_data(queue.clone(), self.0.clone());
        let buffer2 = bufferize_data(queue.clone(), self.1.clone());

        Arc::new(
            PersistentDescriptorSet::start(pipeline, set_idx)
                .add_buffer(buffer1)
                .expect(&format!("Panic adding 1st buffer at set idx {}", set_idx))
                .add_buffer(buffer2)
                .expect(&format!("Panic adding 2nd buffer at set idx {}", set_idx))
                .build()
                .expect(&format!("Panic finalizing set at set idx {}", set_idx))
        )
    }
}

impl<T: Data> Set for (Image, T) {
    fn upload(
        &self,
        queue: Arc<Queue>,
        pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
        set_idx: usize,
    ) -> Arc<dyn DescriptorSet + Send + Sync> {
        let sampler = default_sampler(queue.device().clone());
        let buffer2 = bufferize_data(queue.clone(), self.1.clone());

        Arc::new(
            PersistentDescriptorSet::start(pipeline, set_idx)
                .add_sampled_image(self.0.clone(), sampler)
                .expect(&format!("Panic adding 1st image at set idx {}", set_idx))
                .add_buffer(buffer2)
                .expect(&format!("Panic adding 2nd buffer at set idx {}", set_idx))
                .build()
                .expect(&format!("Panic finalizing set at set idx {}", set_idx))
        )
    }
}

impl<T: Data> Set for (T, Image) {
    fn upload(
        &self,
        queue: Arc<Queue>,
        pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
        set_idx: usize,
    ) -> Arc<dyn DescriptorSet + Send + Sync> {
        let sampler = default_sampler(queue.device().clone());
        let buffer1 = bufferize_data(queue.clone(), self.0.clone());

        Arc::new(
            PersistentDescriptorSet::start(pipeline, set_idx)
                .add_sampled_image(self.1.clone(), sampler)
                .expect(&format!("Panic adding 1st image at set idx {}", set_idx))
                .add_buffer(buffer1)
                .expect(&format!("Panic adding 2nd buffer at set idx {}", set_idx))
                .build()
                .expect(&format!("Panic finalizing set at set idx {}", set_idx))
        )
    }
}

impl Set for (Image, Image) {
    fn upload(
        &self,
        queue: Arc<Queue>,
        pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
        set_idx: usize,
    ) -> Arc<dyn DescriptorSet + Send + Sync> {
        let sampler = default_sampler(queue.device().clone());

        Arc::new(
            PersistentDescriptorSet::start(pipeline, set_idx)
                .add_sampled_image(self.0.clone(), sampler.clone())
                .expect(&format!("Panic adding 1st image at set idx {}", set_idx))
                .add_sampled_image(self.1.clone(), sampler)
                .expect(&format!("Panic adding 2nd image at set idx {}", set_idx))
                .build()
                .expect(&format!("Panic finalizing set at set idx {}", set_idx))
        )
    }
}

// length 3
impl<T1: Data, T2: Data, T3: Data> Set for (T1, T2, T3) {
    fn upload(
        &self,
        queue: Arc<Queue>,
        pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
        set_idx: usize,
    ) -> Arc<dyn DescriptorSet + Send + Sync> {
        let buffer1 = bufferize_data(queue.clone(), self.0.clone());
        let buffer2 = bufferize_data(queue.clone(), self.1.clone());
        let buffer3 = bufferize_data(queue.clone(), self.2.clone());

        Arc::new(
            PersistentDescriptorSet::start(pipeline, set_idx)
                .add_buffer(buffer1)
                .expect(&format!("Panic adding 1st buffer at set idx {}", set_idx))
                .add_buffer(buffer2)
                .expect(&format!("Panic adding 2nd buffer at set idx {}", set_idx))
                .add_buffer(buffer3)
                .expect(&format!("Panic adding 3rd buffer at set idx {}", set_idx))
                .build()
                .expect(&format!("Panic finalizing set at set idx {}", set_idx))
        )
    }
}

impl<T1: Data, T2: Data> Set for (Image, T1, T2) {
    fn upload(
        &self,
        queue: Arc<Queue>,
        pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
        set_idx: usize,
    ) -> Arc<dyn DescriptorSet + Send + Sync> {
        let sampler = default_sampler(queue.device().clone());
        let buffer2 = bufferize_data(queue.clone(), self.1.clone());
        let buffer3 = bufferize_data(queue.clone(), self.2.clone());

        Arc::new(
            PersistentDescriptorSet::start(pipeline, set_idx)
                .add_sampled_image(self.0.clone(), sampler)
                .expect(&format!("Panic adding 1st image at set idx {}", set_idx))
                .add_buffer(buffer2)
                .expect(&format!("Panic adding 2nd buffer at set idx {}", set_idx))
                .add_buffer(buffer3)
                .expect(&format!("Panic adding 3rd buffer at set idx {}", set_idx))
                .build()
                .expect(&format!("Panic finalizing set at set idx {}", set_idx))
        )
    }
}

impl<T1: Data, T2: Data> Set for (T1, Image, T2) {
    fn upload(
        &self,
        queue: Arc<Queue>,
        pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
        set_idx: usize,
    ) -> Arc<dyn DescriptorSet + Send + Sync> {
        let sampler = default_sampler(queue.device().clone());
        let buffer1 = bufferize_data(queue.clone(), self.0.clone());
        let buffer3 = bufferize_data(queue.clone(), self.2.clone());

        Arc::new(
            PersistentDescriptorSet::start(pipeline, set_idx)
                .add_buffer(buffer1)
                .expect(&format!("Panic adding 1st buffer at set idx {}", set_idx))
                .add_sampled_image(self.1.clone(), sampler)
                .expect(&format!("Panic adding 2nd image at set idx {}", set_idx))
                .add_buffer(buffer3)
                .expect(&format!("Panic adding 3rd buffer at set idx {}", set_idx))
                .build()
                .expect(&format!("Panic finalizing set at set idx {}", set_idx))
        )
    }
}

impl<T1: Data, T2: Data> Set for (T1, T2, Image) {
    fn upload(
        &self,
        queue: Arc<Queue>,
        pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
        set_idx: usize,
    ) -> Arc<dyn DescriptorSet + Send + Sync> {
        let sampler = default_sampler(queue.device().clone());
        let buffer1 = bufferize_data(queue.clone(), self.0.clone());
        let buffer2 = bufferize_data(queue.clone(), self.1.clone());

        Arc::new(
            PersistentDescriptorSet::start(pipeline, set_idx)
                .add_buffer(buffer1)
                .expect(&format!("Panic adding 1st buffer at set idx {}", set_idx))
                .add_buffer(buffer2)
                .expect(&format!("Panic adding 2nd buffer at set idx {}", set_idx))
                .add_sampled_image(self.2.clone(), sampler)
                .expect(&format!("Panic adding 3rd image at set idx {}", set_idx))
                .build()
                .expect(&format!("Panic finalizing set at set idx {}", set_idx))
        )
    }
}

impl<T: Data> Set for (T, Image, Image) {
    fn upload(
        &self,
        queue: Arc<Queue>,
        pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
        set_idx: usize,
    ) -> Arc<dyn DescriptorSet + Send + Sync> {
        let sampler = default_sampler(queue.device().clone());
        let buffer1 = bufferize_data(queue.clone(), self.0.clone());

        Arc::new(
            PersistentDescriptorSet::start(pipeline, set_idx)
                .add_buffer(buffer1)
                .expect(&format!("Panic adding 1st buffer at set idx {}", set_idx))
                .add_sampled_image(self.1.clone(), sampler.clone())
                .expect(&format!("Panic adding 2nd image at set idx {}", set_idx))
                .add_sampled_image(self.2.clone(), sampler)
                .expect(&format!("Panic adding 3rd image at set idx {}", set_idx))
                .build()
                .expect(&format!("Panic finalizing set at set idx {}", set_idx))
        )
    }
}

impl<T: Data> Set for (Image, T, Image) {
    fn upload(
        &self,
        queue: Arc<Queue>,
        pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
        set_idx: usize,
    ) -> Arc<dyn DescriptorSet + Send + Sync> {
        let sampler = default_sampler(queue.device().clone());
        let buffer2 = bufferize_data(queue.clone(), self.1.clone());

        Arc::new(
            PersistentDescriptorSet::start(pipeline, set_idx)
                .add_sampled_image(self.0.clone(), sampler.clone())
                .expect(&format!("Panic adding 1st image at set idx {}", set_idx))
                .add_buffer(buffer2)
                .expect(&format!("Panic adding 2nd buffer at set idx {}", set_idx))
                .add_sampled_image(self.2.clone(), sampler)
                .expect(&format!("Panic adding 3rd image at set idx {}", set_idx))
                .build()
                .expect(&format!("Panic finalizing set at set idx {}", set_idx))
        )
    }
}

impl<T: Data> Set for (Image, Image, T) {
    fn upload(
        &self,
        queue: Arc<Queue>,
        pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
        set_idx: usize,
    ) -> Arc<dyn DescriptorSet + Send + Sync> {
        let sampler = default_sampler(queue.device().clone());
        let buffer3 = bufferize_data(queue.clone(), self.2.clone());

        Arc::new(
            PersistentDescriptorSet::start(pipeline, set_idx)
                .add_sampled_image(self.0.clone(), sampler.clone())
                .expect(&format!("Panic adding 1st image at set idx {}", set_idx))
                .add_sampled_image(self.1.clone(), sampler)
                .expect(&format!("Panic adding 2nd image at set idx {}", set_idx))
                .add_buffer(buffer3)
                .expect(&format!("Panic adding 1st buffer at set idx {}", set_idx))
                .build()
                .expect(&format!("Panic finalizing set at set idx {}", set_idx))
        )
    }
}

impl Set for (Image, Image, Image) {
    fn upload(
        &self,
        queue: Arc<Queue>,
        pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
        set_idx: usize,
    ) -> Arc<dyn DescriptorSet + Send + Sync> {
        let sampler = default_sampler(queue.device().clone());

        Arc::new(
            PersistentDescriptorSet::start(pipeline, set_idx)
                .add_sampled_image(self.0.clone(), sampler.clone())
                .expect(&format!("Panic adding 1st image at set idx {}", set_idx))
                .add_sampled_image(self.1.clone(), sampler.clone())
                .expect(&format!("Panic adding 2nd image at set idx {}", set_idx))
                .add_sampled_image(self.2.clone(), sampler)
                .expect(&format!("Panic adding 3rd image at set idx {}", set_idx))
                .build()
                .expect(&format!("Panic finalizing set at set idx {}", set_idx))
        )
    }
}

// length 4 will be FUN!

pub type Image = Arc<dyn ImageViewAccess + Send + Sync>;

pub trait Data: Send + Sync + Clone + 'static {}
