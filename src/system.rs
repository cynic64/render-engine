use vulkano::buffer::{BufferAccess, BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState};
use vulkano::descriptor::descriptor_set::{
    DescriptorSet, PersistentDescriptorSet,
};
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

use crate::producer::SharedResources;
use crate::render_passes;

// TODO: make the whole thing less prone to runtime panics. vecs of strings are
// a little sketchy. Maybe make a function that checks the system to ensure
// it'll work?
// maybe also make it so that one component changes, all the others are forced
// to update too. Because if a producer is added, the shaders -will- have to
// change.

// A system is a list of passes that takes a bunch of data and produces a frame
// for it.
pub struct System<'a> {
    passes: Vec<Pass<'a>>,
    sampler: Arc<Sampler>,
    // stores the vbuf of the screen-filling square used for non-geometry passes
    simple_vbuf: Arc<dyn BufferAccess + Send + Sync>,
    simple_ibuf: Arc<CpuAccessibleBuffer<[u32]>>,
    device: Arc<Device>,
    queue: Arc<Queue>,
    output_tag: &'a str,
}

// TODO: some of the docs below are BS, because in a complex pass the vertex
// shader and fragment combo is not always the same (different objects can have
// different pipelines and all that). Make it consistent!
// maybe rename to PixelPass and GeoPass and include a 3rd option for different
// objects with different pipelines
// How bout PixelPass, HomoGeoPass and HeteroGeoPass?

// A pass is a single operation carried out by a vertex shader and fragment
// shader combination. For example: a geometry pass to draw some objects in 3D
// space it would only have the 'MVP' need, because it wouldn't need any other
// images (like a G-buffer or something similar) Another example: a lighting
// pass taking something previously rendered into a G-buffer. It would have a
// need of 'albedo', 'normal', 'position', or whatever you called the images
// when you created them in a geometry pass.
//
// When used in a system, the system will create all the necessary images first,
//   and add the needed images to a uniform buffer

// Simple means the system will create a square that fills the screen and run
// the shaders on that. Complex is what you'd use for rendering the actual
// geometry, providing your own objects.
pub enum Pass<'a> {
    Complex {
        images_created: Vec<&'a str>,
        images_needed: Vec<&'a str>,
        buffers_needed: Vec<&'a str>,
        render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    },
    Simple {
        images_created: Vec<&'a str>,
        images_needed: Vec<&'a str>,
        buffers_needed: Vec<&'a str>,
        render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
        // because no objects are passed to a Simple Pass, the pipeline is set
        // for the whole pass instead of individual objects
        // the shaders are included in the pipeline, so they are also part of
        // this
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

        // TODO: move these to another file
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

        let simple_ibuf = {
            CpuAccessibleBuffer::from_iter(
                device.clone(),
                BufferUsage::all(),
                [0, 1, 2, 3].iter().cloned(),
            )
            .unwrap()
        };

        Self {
            passes,
            sampler,
            simple_vbuf,
            simple_ibuf,
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
        shared_resources: SharedResources,
        dest_image: Arc<dyn ImageViewAccess + Send + Sync>,
        future: F,
    ) -> Box<dyn GpuFuture>
    where
        F: GpuFuture + 'static,
    {
        // TODO: change vk_window so you submit an image

        // create dynamic state (will be the same for every draw call)
        let dynamic_state = dynamic_state_for_dimensions(dimensions);

        // create all images and framebuffers
        let mut images = images_for_passes(self.device.clone(), dimensions, &self.passes);
        // replace destination image with the real one
        images.insert(self.output_tag, dest_image);

        let framebuffers = framebuffers_for_passes(images.clone(), &self.passes);

        // add all images not produced by passes
        for (image_tag, image) in shared_resources.images.iter() {
            images.insert(image_tag, image.clone());
        }

        // create the command buffer
        let mut cmd_buf_builder = AutoCommandBufferBuilder::primary_one_time_submit(
            self.device.clone(),
            self.queue.family(),
        )
        .unwrap();

        // TODO: rename to pass_idx and restructure some of this
        for (pass_idx, pass) in self.passes.iter().enumerate() {
            let framebuffer = framebuffers[pass_idx].clone();

            let clear_values = render_passes::clear_values_for_pass(pass.get_render_pass().clone());

            cmd_buf_builder = cmd_buf_builder
                .begin_render_pass(framebuffer, false, clear_values)
                .unwrap();

            // TODO: make naming more clear on when it's just the tag and when
            // it's the actual image
            let images_needed: Vec<Arc<dyn ImageViewAccess + Send + Sync>> = pass
                .images_needed_tags()
                .iter()
                .map(|tag| {
                    images
                        .get(tag)
                        .expect("missing key when getting image in system")
                        .clone()
                })
                .collect();

            let buffers_needed: Vec<Arc<dyn BufferAccess + Send + Sync>> = pass
                .buffers_needed_tags()
                .iter()
                .map(|tag| {
                    shared_resources
                        .buffers
                        .get(tag)
                        .expect("missing key when getting resource in system")
                        .clone()
                })
                .collect();

            match pass {
                Pass::Complex { .. } => {
                    let pass_objects = objects[pass_idx].clone();

                    for object in pass_objects.iter() {
                        let collection = collection_from_resources(
                            self.sampler.clone(),
                            object.pipeline.clone(),
                            &images_needed,
                            &buffers_needed,
                        );

                        cmd_buf_builder = cmd_buf_builder
                            .draw_indexed(
                                object.pipeline.clone(),
                                &dynamic_state,
                                vec![object.vbuf.clone()],
                                object.ibuf.clone(),
                                collection,
                                (),
                            )
                            .unwrap();
                    }
                }
                Pass::Simple { pipeline, .. } => {
                    let collection = collection_from_resources(
                        self.sampler.clone(),
                        pipeline.clone(),
                        &images_needed,
                        &buffers_needed,
                    );

                    cmd_buf_builder = cmd_buf_builder
                        .draw_indexed(
                            pipeline.clone(),
                            &dynamic_state,
                            vec![self.simple_vbuf.clone()],
                            self.simple_ibuf.clone(),
                            collection,
                            (),
                        )
                        .unwrap();
                }
            };

            cmd_buf_builder = cmd_buf_builder.end_render_pass().unwrap();
        }

        let final_cmd_buf = cmd_buf_builder.build().unwrap();

        Box::new(
            future
                .then_execute(self.queue.clone(), final_cmd_buf)
                .unwrap(),
        )
    }

    pub fn get_passes(&self) -> &[Pass] {
        &self.passes
    }
}

#[derive(Clone)]
pub struct RenderableObject {
    pub pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    pub vbuf: Arc<dyn BufferAccess + Send + Sync>,
    pub ibuf: Arc<CpuAccessibleBuffer<[u32]>>,
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

fn collection_from_resources(
    sampler: Arc<Sampler>,
    pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    images: &[Arc<dyn ImageViewAccess + Send + Sync>],
    buffers: &[Arc<dyn BufferAccess + Send + Sync>],
) -> Vec<Arc<dyn DescriptorSet + Send + Sync>> {
    let resource_set_idx = if images.len() >= 1 { 1 } else { 0 };

    let image_set = pds_for_images(sampler, pipeline.clone(), &images);

    let resource_set = pds_for_buffers(pipeline.clone(), &buffers, resource_set_idx);

    // either no images and no buffers were needed, or images but no buffers, or
    // buffers but no images, or buffers and images. this handles every case and
    // converts each into a vector of just the needed resources
    match (image_set, resource_set) {
        (None, None) => vec![],
        (Some(real_image_set), None) => vec![real_image_set.clone()],
        (None, Some(real_resource_set)) => vec![real_resource_set.clone()],
        (Some(real_image_set), Some(real_resource_set)) => {
            vec![real_image_set.clone(), real_resource_set.clone()]
        }
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

fn pds_for_buffers(
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

fn images_for_passes<'a>(
    device: Arc<Device>,
    dimensions: [u32; 2],
    passes: &'a [Pass],
) -> HashMap<&'a str, Arc<dyn ImageViewAccess + Send + Sync>> {
    // for now this ignores the fact that the output image is special and
    // provided from outside System. any users of this function should replace
    // that image with the real one afterwards.
    let mut images = HashMap::new();
    for pass in passes.iter() {
        for (image_idx, &image_tag) in pass.images_created_tags().iter().enumerate() {
            let desc = pass
                .get_render_pass()
                .attachment_desc(image_idx)
                .expect("Couldn't get the attachment description when creating images for passes");
            let image = create_image_for_desc(device.clone(), dimensions, desc);
            images.insert(image_tag, image);
        }
    }

    images
}

fn framebuffers_for_passes<'a>(
    images: HashMap<&'a str, Arc<dyn ImageViewAccess + Send + Sync>>,
    passes: &'a [Pass],
) -> Vec<Arc<dyn FramebufferAbstract + Send + Sync>> {
    let mut framebuffers = vec![];

    for pass in passes.iter() {
        let images_tags_created = pass.images_created_tags();
        let images = images_tags_created
            .iter()
            .map(|tag| {
                images
                    .get(tag)
                    .expect("Couldn't get image when creating framebuffers for passes")
                    .clone()
            })
            .collect();

        let framebuffer = fb_from_images(pass.get_render_pass(), images);
        framebuffers.push(framebuffer);
    }

    framebuffers
}

impl<'a> Pass<'a> {
    pub fn images_created_tags(&self) -> &[&str] {
        match self {
            Pass::Complex { images_created, .. } => images_created,
            Pass::Simple { images_created, .. } => images_created,
        }
    }

    pub fn images_needed_tags(&self) -> &[&str] {
        match self {
            Pass::Complex { images_needed, .. } => images_needed,
            Pass::Simple { images_needed, .. } => images_needed,
        }
    }

    pub fn get_render_pass(&self) -> Arc<dyn RenderPassAbstract + Send + Sync> {
        match self {
            Pass::Complex { render_pass, .. } => render_pass.clone(),
            Pass::Simple { render_pass, .. } => render_pass.clone(),
        }
    }

    pub fn buffers_needed_tags(&self) -> &[&str] {
        match self {
            Pass::Complex {
                buffers_needed, ..
            } => buffers_needed,
            Pass::Simple {
                buffers_needed, ..
            } => buffers_needed,
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct SimpleVertex {
    position: [f32; 2],
}
vulkano::impl_vertex!(SimpleVertex, position);

// TODO: maybe rename to vertex3D?
#[derive(Default, Debug, Clone)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
}
vulkano::impl_vertex!(Vertex, position, normal);
