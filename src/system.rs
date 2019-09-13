use vulkano::buffer::{BufferAccess, BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState};
use vulkano::descriptor::descriptor_set::{DescriptorSet, PersistentDescriptorSet};
use vulkano::device::{Device, Queue};
use vulkano::framebuffer::{
    AttachmentDescription, Framebuffer, FramebufferAbstract, RenderPassAbstract,
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
    // stores the vbuf of the screen-filling square used for non-geometry passes
    simple_vbuf: Arc<dyn BufferAccess + Send + Sync>,
    device: Arc<Device>,
    queue: Arc<Queue>,
    output_tag: &'a str,
}

// A pass is a single operation carried out by a vertex shader and fragment
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

// TODO: maybe use a trait for this instead
// Simple means the system will create a square that fills the screen and run
// the shaders on that. Complex is what you'd use for rendering the actual
// geometry, providing your own objects.
pub enum Pass<'a> {
    Complex {
        images_created: Vec<&'a str>,
        images_needed: Vec<&'a str>,
        resources_needed: Vec<&'a str>,
        render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    },
    Simple {
        images_created: Vec<&'a str>,
        images_needed: Vec<&'a str>,
        resources_needed: Vec<&'a str>,
        render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
        // because no objects are passed to a SimplePass, the pipeline is set for
        // the whole pass instead of individual objects
        pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    },
}

impl<'a> System<'a> {
    pub fn new(queue: Arc<Queue>, passes: Vec<Pass<'a>>, output_tag: &'a str) -> Self {
        let device = queue.device().clone();

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

        let simple_vbuf = {
            CpuAccessibleBuffer::from_iter(
                device.clone(),
                BufferUsage::all(),
                [
                    SimpleVertex {
                        position: [-1.0, -1.0],
                    },
                    SimpleVertex {
                        position: [-1.0, 1.0],
                    },
                    SimpleVertex {
                        position: [1.0, -1.0],
                    },
                    SimpleVertex {
                        position: [1.0, 1.0],
                    },
                ]
                .iter()
                .cloned(),
            )
            .unwrap()
        };

        Self {
            passes,
            sampler,
            simple_vbuf,
            device,
            queue,
            output_tag,
        }
    }

    pub fn draw_frame<F>(
        &mut self,
        dimensions: [u32; 2],
        // TODO: switch to a hashmap for this, with keys being the pass and a
        // list of objects for each
        objects: Vec<Vec<RenderableObject>>,
        shared_resources: HashMap<&str, Arc<dyn BufferAccess + Send + Sync>>,
        dest_image: Arc<dyn ImageViewAccess + Send + Sync>,
        future: F,
    ) -> Box<dyn GpuFuture>
    where
        F: GpuFuture + 'static,
    {
        // TODO: change vk_window so you submit an image rather than a command buffer

        // create dynamic state (will be the same for every draw call)
        let dynamic_state = dynamic_state_for_dimensions(dimensions);

        // create all images and framebuffers
        let mut images: HashMap<&str, Arc<dyn ImageViewAccess + Send + Sync>> = HashMap::new();
        let mut framebuffers: Vec<Arc<dyn FramebufferAbstract + Send + Sync>> = vec![];
        for pass in self.passes.iter() {
            let mut images_for_pass = vec![];
            for (image_idx, image_tag) in pass.get_images_created().iter().enumerate() {
                // TODO: add a range check
                let image = if *image_tag == self.output_tag {
                    dest_image.clone()
                } else {
                    let desc = pass.get_render_pass().attachment_desc(image_idx).expect("pls no");
                    create_image_for_desc(self.device.clone(), dimensions, desc)
                };

                images.insert(image_tag, image.clone());
                images_for_pass.push(image.clone());
            }

            let framebuffer = fb_from_images(pass.get_render_pass().clone(), images_for_pass);
            framebuffers.push(framebuffer);
        }

        // create the command buffer
        let mut cmd_buf_builder =
            AutoCommandBufferBuilder::primary_one_time_submit(self.device.clone(), self.queue.family())
                .unwrap();
        for (idx, pass) in self.passes.iter().enumerate() {
            let framebuffer = framebuffers[idx].clone();

            let clear_values = render_passes::clear_values_for_pass(pass.get_render_pass().clone());

            cmd_buf_builder = cmd_buf_builder
                .begin_render_pass(framebuffer, false, clear_values)
                .unwrap();

            // if it's a complex pass take the objects we were given, if it's a
            // simple one just use a screen-villing vbuf
            let pass_objects = match pass {
                Pass::Complex { .. } => objects[idx].clone(),
                Pass::Simple { pipeline, ..} => vec![RenderableObject {
                    pipeline: pipeline.clone(),
                    vbuf: self.simple_vbuf.clone(),
                    additional_resources: None,
                }],
            };

            for object in pass_objects.iter() {
                // TODO: don't make a set for every object
                // create a descriptor set with samplers for each of the needed images
                let images_needed: Vec<_> = pass
                    .get_images_needed()
                    .iter()
                    .map(|tag| images.get(tag).expect("missing key").clone())
                    .collect();

                let mut resources_needed: Vec<_> = pass
                    .get_resources_needed()
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

        Box::new(future.then_execute(self.queue.clone(), final_cmd_buf).unwrap())
    }

    pub fn get_passes(&self) -> &[Pass] {
        &self.passes
    }
}

#[derive(Clone)]
pub struct RenderableObject {
    pub pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    pub vbuf: Arc<dyn BufferAccess + Send + Sync>,
    pub additional_resources: Option<Arc<dyn BufferAccess + Send + Sync>>,
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

impl<'a> Pass<'a> {
    pub fn get_images_created(&self) -> Vec<&str> {
        match self {
            Pass::Complex { images_created, .. } => images_created.clone(),
            Pass::Simple { images_created, .. } => images_created.clone(),
        }
    }

    pub fn get_images_needed(&self) -> Vec<&str> {
        match self {
            Pass::Complex { images_needed, .. } => images_needed.clone(),
            Pass::Simple { images_needed, .. } => images_needed.clone(),
        }
    }

    pub fn get_resources_needed(&self) -> Vec<&str> {
        match self {
            Pass::Complex {
                resources_needed, ..
            } => resources_needed.clone(),
            Pass::Simple {
                resources_needed, ..
            } => resources_needed.clone(),
        }
    }

    pub fn get_render_pass(&self) -> Arc<dyn RenderPassAbstract + Send + Sync> {
        match self {
            Pass::Complex { render_pass, .. } => render_pass.clone(),
            Pass::Simple { render_pass, .. } => render_pass.clone(),
        }
    }
}

#[derive(Default, Debug, Clone)]
struct SimpleVertex {
    position: [f32; 2],
}
vulkano::impl_vertex!(SimpleVertex, position);
