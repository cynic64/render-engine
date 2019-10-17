use vulkano::buffer::BufferAccess;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::descriptor::DescriptorSet;
use vulkano::device::Device;
use vulkano::image::ImageViewAccess;
use vulkano::pipeline::GraphicsPipelineAbstract;
use vulkano::sampler::{Filter, MipmapMode, Sampler, SamplerAddressMode};

use std::collections::HashMap;
use std::sync::Arc;

use crate::input::get_elapsed;
use crate::pipeline_cache::PipelineSpec;
use crate::system::Pass;

pub struct CollectionCache {
    c_collections: Vec<CachedCollection>,
    sampler: Arc<Sampler>,
    stats: CacheStats,
}

struct CachedCollection {
    spec: PipelineSpec,
    collection: Collection,
}

#[derive(Default)]
struct CacheStats {
    hits: u32,
    misses: u32,
    gen_times: Vec<f32>,
}

impl CollectionCache {
    pub fn new(device: Arc<Device>) -> Self {
        let sampler = Sampler::new(
            device,
            Filter::Linear,
            Filter::Linear,
            MipmapMode::Nearest,
            SamplerAddressMode::Repeat,
            SamplerAddressMode::Repeat,
            SamplerAddressMode::Repeat,
            0.0,
            1.0,
            0.0,
            0.0,
        )
        .unwrap();

        Self {
            c_collections: vec![],
            sampler,
            stats: CacheStats::default(),
        }
    }

    // TODO: replace with a struct that defines a uniform buffer: what spec
    // pipeline is belongs to, what resources it needs, etc.

    // important to remember: 'images' are written to then read later, whereas
    // resources are meant to be read only.
    pub fn get(
        &mut self,
        spec: &PipelineSpec,
        pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
        pass: &Pass,
        images: &HashMap<&str, Arc<dyn ImageViewAccess + Send + Sync>>,
    ) -> Collection {
        let mut collection = None;

        for c_collection in self.c_collections.iter() {
            if c_collection.spec == *spec {
                collection = Some(c_collection.collection.clone());
                self.stats.hits += 1;
            }
        }

        match collection {
            Some(collection) => collection,
            None => {
                self.stats.misses += 1;

                let start_time = std::time::Instant::now();

                let images_needed: Vec<Arc<dyn ImageViewAccess + Send + Sync>> = pass
                    .images_needed_tags
                    .iter()
                    .map(|tag| {
                        images
                            .get(tag)
                            .expect("missing key when getting image")
                            .clone()
                    })
                    .collect();

                let collection = collection_from_images(
                    self.sampler.clone(),
                    pipeline.clone(),
                    &images_needed,
                );

                let c_collection = CachedCollection {
                    spec: spec.clone(),
                    collection: collection.clone(),
                };
                self.c_collections.push(c_collection);

                self.stats.gen_times.push(get_elapsed(start_time));

                collection
            }
        }
    }

    pub fn clear(&mut self) {
        self.c_collections = vec![];
    }

    pub fn print_stats(&self) {
        let avg: f32 =
            self.stats.gen_times.iter().sum::<f32>() / (self.stats.gen_times.len() as f32);
        let percent =
            (self.stats.hits as f32) / ((self.stats.hits + self.stats.misses) as f32) * 100.0;
        println!(
            "Hits: {}, misses: {}, {}%, avg. time taken to gen collection: {}",
            self.stats.hits, self.stats.misses, percent, avg
        );
    }
}

// TODO: this is kinda obsolete now
fn collection_from_images(
    sampler: Arc<Sampler>,
    pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    images: &[Arc<dyn ImageViewAccess + Send + Sync>],
) -> Vec<Arc<dyn DescriptorSet + Send + Sync>> {
    if let Some(image_set) = pds_for_images(sampler, pipeline.clone(), &images) {
        vec![image_set]
    } else {
        vec![]
    }
}

fn pds_for_images(
    sampler: Arc<Sampler>,
    pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    images: &[Arc<dyn ImageViewAccess + Send + Sync>],
) -> Option<Arc<dyn DescriptorSet + Send + Sync>> {
    match images.len() {
        0 => None,
        1 => Some(Arc::new(
            PersistentDescriptorSet::start(pipeline, 0)
                .add_sampled_image(images[0].clone(), sampler)
                .unwrap()
                .build()
                .unwrap(),
        )),
        2 => Some(Arc::new(
            PersistentDescriptorSet::start(pipeline, 0)
                .add_sampled_image(images[0].clone(), sampler.clone())
                .unwrap()
                .add_sampled_image(images[1].clone(), sampler.clone())
                .unwrap()
                .build()
                .unwrap(),
        )),
        3 => Some(Arc::new(
            PersistentDescriptorSet::start(pipeline, 0)
                .add_sampled_image(images[0].clone(), sampler.clone())
                .unwrap()
                .add_sampled_image(images[1].clone(), sampler.clone())
                .unwrap()
                .add_sampled_image(images[2].clone(), sampler.clone())
                .unwrap()
                .build()
                .unwrap(),
        )),
        4 => Some(Arc::new(
            PersistentDescriptorSet::start(pipeline, 0)
                .add_sampled_image(images[0].clone(), sampler.clone())
                .unwrap()
                .add_sampled_image(images[1].clone(), sampler.clone())
                .unwrap()
                .add_sampled_image(images[2].clone(), sampler.clone())
                .unwrap()
                .add_sampled_image(images[3].clone(), sampler.clone())
                .unwrap()
                .build()
                .unwrap(),
        )),
        _ => panic!("pds_for_images does not support more than 4 images!"),
    }
}

// rename to set for buffers? idk
pub fn pds_for_buffers(
    pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    buffers: &[Arc<dyn BufferAccess + Send + Sync>],
    set_idx: usize,
) -> Option<Arc<dyn DescriptorSet + Send + Sync>> {
    match buffers.len() {
        0 => None,
        1 => Some(Arc::new(
            PersistentDescriptorSet::start(pipeline, set_idx)
                .add_buffer(buffers[0].clone())
                .unwrap()
                .build()
                .unwrap(),
        )),
        2 => Some(Arc::new(
            PersistentDescriptorSet::start(pipeline, set_idx)
                .add_buffer(buffers[0].clone())
                .unwrap()
                .add_buffer(buffers[1].clone())
                .unwrap()
                .build()
                .unwrap(),
        )),
        3 => Some(Arc::new(
            PersistentDescriptorSet::start(pipeline, set_idx)
                .add_buffer(buffers[0].clone())
                .unwrap()
                .add_buffer(buffers[1].clone())
                .unwrap()
                .add_buffer(buffers[2].clone())
                .unwrap()
                .build()
                .unwrap(),
        )),
        4 => Some(Arc::new(
            PersistentDescriptorSet::start(pipeline, set_idx)
                .add_buffer(buffers[0].clone())
                .unwrap()
                .add_buffer(buffers[1].clone())
                .unwrap()
                .add_buffer(buffers[2].clone())
                .unwrap()
                .add_buffer(buffers[3].clone())
                .unwrap()
                .build()
                .unwrap(),
        )),
        _ => panic!("pds_for_buffers does not support more than 4 buffers!"),
    }
}

pub type Collection = Vec<Arc<dyn DescriptorSet + Send + Sync>>;
