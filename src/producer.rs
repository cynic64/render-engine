use vulkano::buffer::BufferAccess;
use vulkano::device::Device;
use vulkano::image::traits::ImageViewAccess;

use std::collections::HashMap;
use std::sync::Arc;

use crate::input::FrameInfo;

// trait for data that needs to be passed to the shaders that changes every
// frame. not to be used for specific objects.
// TODO: better docs
pub trait BufferProducer {
    fn update(&mut self, _frame_info: FrameInfo) {}
    fn create_buffer(&self, device: Arc<Device>) -> BufferResource;
    fn name(&self) -> &str;
}

// trait for additional textures that need to be passed to the shaders. again,
// not to be used for specific objects.
pub trait ImageProducer {
    fn update(&mut self, _frame_info: FrameInfo) {}
    fn create_image(&self, device: Arc<Device>) -> ImageResource;
    fn name(&self) -> &str;
}

// TODO: right now the resources this produces and the resources the system
// needs are expected to magically fit together, or else boom! runtime panic.
// Figure out a way to check this stuff at compile time, maybe a trait for
// shared resources or sth.
pub struct ProducerCollection {
    image_producers: Vec<Box<dyn ImageProducer>>,
    buffer_producers: Vec<Box<dyn BufferProducer>>,
}

impl ProducerCollection {
    pub fn new(
        image_producers: Vec<Box<dyn ImageProducer>>,
        buffer_producers: Vec<Box<dyn BufferProducer>>,
    ) -> Self {
        Self {
            buffer_producers,
            image_producers,
        }
    }

    pub fn set_image_producers(&mut self, new_image_producers: Vec<Box<dyn ImageProducer>>) {
        self.image_producers = new_image_producers;
    }

    pub fn set_buffer_producers(&mut self, new_buffer_producers: Vec<Box<dyn BufferProducer>>) {
        self.buffer_producers = new_buffer_producers;
    }

    pub fn update(&mut self, frame_info: FrameInfo) {
        self.image_producers
            .iter_mut()
            .for_each(|prod| prod.update(frame_info.clone()));

        self.buffer_producers
            .iter_mut()
            .for_each(|prod| prod.update(frame_info.clone()));
    }

    pub fn get_shared_resources(&self, device: Arc<Device>) -> SharedResources {
        let images = self
            .image_producers
            .iter()
            .map(|prod| (prod.name(), prod.create_image(device.clone())))
            .collect();

        let buffers = self
            .buffer_producers
            .iter()
            .map(|prod| (prod.name(), prod.create_buffer(device.clone())))
            .collect();

        SharedResources {
            images,
            buffers,
        }
    }
}

pub type ImageResource = Arc<dyn ImageViewAccess + Send + Sync>;
pub type BufferResource = Arc<dyn BufferAccess + Send + Sync>;
pub struct SharedResources<'a> {
    pub images: HashMap<&'a str, ImageResource>,
    pub buffers: HashMap<&'a str, BufferResource>,
}
