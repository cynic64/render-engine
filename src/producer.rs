use vulkano::device::Device;
use vulkano::buffer::BufferAccess;

use crate::input::FrameInfo;

use std::sync::Arc;
use std::collections::HashMap;

// trait for data that needs to be passed to the shaders that changes every
// frame. not to be used for specific objects.
// TODO: better docs
pub trait ResourceProducer {
    fn update(&mut self, _frame_info: FrameInfo) {}
    fn create_buffer(&self, device: Arc<Device>) -> Arc<dyn BufferAccess + Send + Sync>;
    fn name(&self) -> &str;
}

// TODO: right now the resources this produces and the resources the system
// needs are expected to magically fit together, or else boom! runtime panic.
// Figure out a way to check this stuff at compile time, maybe a trait for
// shared resources or sth.
pub struct ProducerCollection {
    producers: Vec<Box<dyn ResourceProducer>>,
}

impl ProducerCollection {
    pub fn new(producers: Vec<Box<dyn ResourceProducer>>) -> Self {
        Self {
            producers,
        }
    }

    pub fn set_producers(&mut self, new_producers: Vec<Box<dyn ResourceProducer>>) {
        self.producers = new_producers;
    }

    pub fn update(&mut self, frame_info: FrameInfo) {
        self.producers.iter_mut().for_each(|prod| prod.update(frame_info.clone()));
    }

    pub fn get_shared_resources(&self, device: Arc<Device>) -> SharedResources {
        self.producers
            .iter()
            .map(|prod| (prod.name(), prod.create_buffer(device.clone())))
            .collect()
    }
}

pub type Resource = Arc<dyn BufferAccess + Send + Sync>;
pub type SharedResources<'a> = HashMap<&'a str, Resource>;
