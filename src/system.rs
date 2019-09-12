use vulkano::buffer::BufferAccess;
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, DynamicState,
};
use vulkano::descriptor::descriptor_set::{DescriptorSet, PersistentDescriptorSet};
use vulkano::device::{Device, Queue};
use vulkano::framebuffer::{
    AttachmentDescription, Framebuffer, FramebufferAbstract, RenderPassAbstract, RenderPassDesc,
};
use vulkano::image::{AttachmentImage, ImageViewAccess};
use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::GraphicsPipelineAbstract;
use vulkano::sampler::{Filter, MipmapMode, Sampler, SamplerAddressMode};
use vulkano::sync::GpuFuture;

use std::collections::HashMap;
use std::sync::Arc;

use crate::render_passes;

// A system is a list of passes that takes a bunch of data and produces a frame
// for it.
pub struct System<'a> {
    pub passes: Vec<Pass<'a>>,
    sampler: Arc<Sampler>,
}

// A pass is a single operation carried out by a vertex shadeer and fragment
// shader combination.
// For example: a geometry pass to draw some objects in 3D space
//   it would only have the 'MVP' need, because it wouldn't need any other images
//   (like a G-buffer or something similar)
// Another example: a lighting pass taking something previously rendered into a
//   G-buffer. It would have a need of 'albedo', 'normal', 'position', or whatever
//   you called the images when you created them in a geometry pass.
//
// When used in a system, the system will create all the necessary images first,
//   and add the needed images to a uniform buffer
//   'mvp' is a special kind of needed image, it'll add the mvp instead
//   this is temporary, it's a shitty solution
pub struct Pass<'a> {
    pub images_created: Vec<&'a str>,
    pub images_needed: Vec<&'a str>,
    pub resources_needed: Vec<&'a str>,
    pub render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
}

impl<'a> System<'a> {
    pub fn new(device: Arc<Device>, passes: Vec<Pass<'a>>) -> Self {
        let sampler = Sampler::new(
            device.clone(),
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

        Self { passes, sampler }
    }

    pub fn draw_frame<F>(
        &mut self,
        device: Arc<Device>,
        queue: Arc<Queue>,
        dimensions: [u32; 2],
        // TODO: switch to a hashmap for this, with keys being the pass and a
        // list of objects for each
        objects: Vec<Vec<RenderableObject>>,
        shared_resources: HashMap<&str, Arc<dyn BufferAccess + Send + Sync>>,
        output_tag: &str,
        dest_image: Arc<dyn ImageViewAccess + Send + Sync>,
        future: F,
    ) -> Box<dyn GpuFuture>
    where
        F: GpuFuture + 'static,
    {
        // returns a command buffer that can be submitted to the swapchain
        // TODO: change vk_window so you submit an image rather than a command buffer

        // create dynamic state (will be the same for every draw call)
        let dynamic_state = dynamic_state_for_dimensions(dimensions);

        // create all images and framebuffers
        let mut images: HashMap<&str, Arc<dyn ImageViewAccess + Send + Sync>> = HashMap::new();
        let mut framebuffers: Vec<Arc<dyn FramebufferAbstract + Send + Sync>> = vec![];
        for pass in self.passes.iter() {
            let mut images_for_pass = vec![];
            for (image_idx, image_tag) in pass.images_created.iter().enumerate() {
                // TODO: add a range check
                let image = if *image_tag == output_tag {
                    dest_image.clone()
                } else {
                    let desc = pass.render_pass.attachment_desc(image_idx).expect("pls no");
                    create_image_for_desc(device.clone(), dimensions, desc)
                };

                images.insert(image_tag, image.clone());
                images_for_pass.push(image.clone());
            }

            let framebuffer = fb_from_images(pass.render_pass.clone(), images_for_pass);
            framebuffers.push(framebuffer);
        }

        // create the command buffer
        let mut cmd_buf_builder = AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), queue.family()).unwrap();
        for (idx, pass) in self.passes.iter().enumerate() {
            let framebuffer = framebuffers[idx].clone();
            let objects = &objects[idx];

            let clear_values = render_passes::clear_values_for_pass(pass.render_pass.clone());

            cmd_buf_builder = cmd_buf_builder
                    .begin_render_pass(framebuffer, false, clear_values)
                    .unwrap();

            for object in objects.iter() {
                // TODO: don't make a set for every object
                // create a descriptor set with samplers for each of the needed images
                let images_needed: Vec<_> = pass
                    .images_needed
                    .iter()
                    .map(|tag| images.get(tag).expect("missing key").clone())
                    .collect();

                let mut resources_needed: Vec<_> = pass
                    .resources_needed
                    .iter()
                    .map(|tag| shared_resources.get(tag).expect("missing key").clone())
                    .collect();

                if let Some(additional_resources) = &object.additional_resources {
                    resources_needed.push(additional_resources.clone());
                }

                let image_set =
                    pds_for_images(self.sampler.clone(), object.pipeline.clone(), images_needed);
                let resource_set = pds_for_resources(object.pipeline.clone(), resources_needed);
                let sets_collection = match (image_set, resource_set) {
                    (None, None) => vec![],
                    (Some(real_image_set), None) => vec![real_image_set.clone()],
                    (None, Some(real_resource_set)) => vec![real_resource_set.clone()],
                    (Some(real_image_set), Some(real_resource_set)) => {
                        vec![real_image_set.clone(), real_resource_set.clone()]
                    }
                };

                cmd_buf_builder = cmd_buf_builder
                    .draw(
                        object.pipeline.clone(),
                        &dynamic_state,
                        vec![object.vbuf.clone()],
                        sets_collection,
                        (),
                    )
                    .unwrap();
            }

            cmd_buf_builder = cmd_buf_builder.end_render_pass().unwrap();
        }

        let final_cmd_buf = cmd_buf_builder.build().unwrap();

        Box::new(
            future
                .then_execute(
                    queue.clone(),
                    final_cmd_buf,
                )
                .unwrap(),
        )
    }
}

#[derive(Clone)]
pub struct RenderableObject {
    pub pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    pub vbuf: Arc<dyn BufferAccess + Send + Sync>,
    pub additional_resources: Option<Arc<dyn BufferAccess + Send + Sync>>
}

fn create_image_for_desc(
    device: Arc<Device>,
    dimensions: [u32; 2],
    desc: AttachmentDescription,
) -> Arc<dyn ImageViewAccess + Send + Sync> {
    AttachmentImage::sampled_multisampled(device.clone(), dimensions, desc.samples, desc.format)
        .unwrap()
}

fn dynamic_state_for_dimensions(dimensions: [u32; 2]) -> DynamicState {
    let viewport = Viewport {
        origin: [0.0, 0.0],
        dimensions: [dimensions[0] as f32, dimensions[1] as f32],
        depth_range: 0.0..1.0,
    };

    DynamicState {
        line_width: None,
        viewports: Some(vec![viewport]),
        scissors: None,
    }
}

fn pds_for_images(
    sampler: Arc<Sampler>,
    pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    images: Vec<Arc<dyn ImageViewAccess + Send + Sync>>,
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

fn pds_for_resources(
    pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    resources: Vec<Arc<dyn BufferAccess + Send + Sync>>,
) -> Option<Arc<dyn DescriptorSet + Send + Sync>> {
    match resources.len() {
        0 => None,
        1 => Some(Arc::new(
            PersistentDescriptorSet::start(pipeline, 0)
                .add_buffer(resources[0].clone())
                .unwrap()
                .build()
                .unwrap(),
        )),
        2 => Some(Arc::new(
            PersistentDescriptorSet::start(pipeline, 0)
                .add_buffer(resources[0].clone())
                .unwrap()
                .add_buffer(resources[1].clone())
                .unwrap()
                .build()
                .unwrap(),
        )),
        3 => Some(Arc::new(
            PersistentDescriptorSet::start(pipeline, 0)
                .add_buffer(resources[0].clone())
                .unwrap()
                .add_buffer(resources[1].clone())
                .unwrap()
                .add_buffer(resources[2].clone())
                .unwrap()
                .build()
                .unwrap(),
        )),
        4 => Some(Arc::new(
            PersistentDescriptorSet::start(pipeline, 0)
                .add_buffer(resources[0].clone())
                .unwrap()
                .add_buffer(resources[1].clone())
                .unwrap()
                .add_buffer(resources[2].clone())
                .unwrap()
                .add_buffer(resources[3].clone())
                .unwrap()
                .build()
                .unwrap(),
        )),
        _ => panic!("pds_for_resources does not support more than 4 resources!"),
    }
}

fn fb_from_images(
    render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    images: Vec<Arc<dyn ImageViewAccess + Send + Sync>>,
) -> Arc<dyn FramebufferAbstract + Send + Sync> {
    match images.len() {
        0 => panic!("You cannot create a framebuffer with 0 images!"),
        1 => Arc::new(
            Framebuffer::start(render_pass)
                .add(images[0].clone())
                .unwrap()
                .build()
                .unwrap(),
        ),
        2 => Arc::new(
            Framebuffer::start(render_pass)
                .add(images[0].clone())
                .unwrap()
                .add(images[1].clone())
                .unwrap()
                .build()
                .unwrap(),
        ),
        3 => Arc::new(
            Framebuffer::start(render_pass)
                .add(images[0].clone())
                .unwrap()
                .add(images[1].clone())
                .unwrap()
                .add(images[2].clone())
                .unwrap()
                .build()
                .unwrap(),
        ),
        4 => Arc::new(
            Framebuffer::start(render_pass)
                .add(images[0].clone())
                .unwrap()
                .add(images[1].clone())
                .unwrap()
                .add(images[2].clone())
                .unwrap()
                .add(images[3].clone())
                .unwrap()
                .build()
                .unwrap(),
        ),
        _ => panic!("Creating a framebuffer from more than 4 images is unsupported!"),
    }
}
