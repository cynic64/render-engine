use vulkano::buffer::{BufferAccess, ImmutableBuffer};
use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState};
use vulkano::descriptor::DescriptorSet;
use vulkano::device::{Device, Queue};
use vulkano::framebuffer::{
    AttachmentDescription, Framebuffer, FramebufferAbstract, RenderPassAbstract,
};
use vulkano::image::{AttachmentImage, ImageViewAccess, SwapchainImage};
use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::GraphicsPipelineAbstract;
use vulkano::sync::GpuFuture;

use std::collections::HashMap;
use std::sync::Arc;

use crate::collection_cache::CollectionCache;
use crate::pipeline_cache::{PipelineCache, PipelineSpec};
use crate::render_passes::clear_values_for_pass;
use crate::window::Window;

// TODO: make the whole thing less prone to runtime panics. vecs of strings are
// a little sketchy. Maybe make a function that checks the system to ensure
// it'll work?

// A system is a list of passes that takes a bunch of data and produces a frame
// for it.
pub struct System<'a> {
    passes: Vec<Pass<'a>>,
    pipeline_caches: Vec<PipelineCache>,
    collection_cache: CollectionCache,
    // stores the vbuf of the screen-filling square used for non-geometry passes
    device: Arc<Device>,
    queue: Arc<Queue>,
    output_tag: &'a str,
}

// In the end all GPU programs come down to feeding a set of shaders some data
// and getting some data back. Vertex shaders take geometry and rasterize it,
// the output of which is stored in an image. If there are multiple outputs from
// the vertex shader, for example color and normal, 2 images will be created.
// The fragment shader then reads from these images to determine the final
// output color for each pixel on the screen. Compute shaders are another story.

// Passes specify which images the vertex shaders to write to and the fragment
// shaders read from. This does NOT mean textures! Data you want to feed your
// shaders from the CPU, whether in the form of buffers or images, should go in
// Object's custom_set field. The images listed in images_needed will be fed to
// the vertex shader of every object drawn.

// Often drawing a frame requires multiple vertex and fragment shaders operating
// in sequence. This what System is for.
pub struct Pass<'a> {
    pub name: &'a str,
    pub images_created_tags: Vec<&'a str>,
    pub images_needed_tags: Vec<&'a str>,
    pub render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
}

impl<'a> System<'a> {
    pub fn new(queue: Arc<Queue>, passes: Vec<Pass<'a>>, output_tag: &'a str) -> Self {
        let device = queue.device().clone();

        let pipeline_caches = pipe_caches_for_passes(device.clone(), &passes);
        let collection_cache = CollectionCache::new(device.clone());

        Self {
            passes,
            pipeline_caches,
            collection_cache,
            device,
            queue,
            output_tag,
        }
    }

    pub fn render<F>(
        &mut self,
        dimensions: [u32; 2],
        objects: HashMap<&str, Vec<RenderableObject>>,
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

        // create the command buffer
        let mut cmd_buf_builder = AutoCommandBufferBuilder::primary_one_time_submit(
            self.device.clone(),
            self.queue.family(),
        )
        .unwrap();

        for (pass_idx, pass) in self.passes.iter().enumerate() {
            let framebuffer = framebuffers[pass_idx].clone();

            let clear_values = clear_values_for_pass(pass.render_pass.clone());

            cmd_buf_builder = cmd_buf_builder
                .begin_render_pass(framebuffer, false, clear_values)
                .unwrap();

            let pass_objects = objects[pass.name].clone();

            for object in pass_objects.iter() {
                let pipeline = self.pipeline_caches[pass_idx].get(&object.pipeline_spec);

                // only stores the images that are fed to every object, not
                // object-specific images and buffers.
                let mut collection = self.collection_cache.get(
                    &object.pipeline_spec,
                    pipeline.clone(),
                    &pass,
                    &images,
                );

                if let Some(set) = &object.custom_set {
                    collection.push(set.clone());
                }

                cmd_buf_builder = cmd_buf_builder
                    .draw_indexed(
                        pipeline,
                        &dynamic_state,
                        vec![object.vbuf.clone()],
                        object.ibuf.clone(),
                        collection,
                        (),
                    )
                    .unwrap();
            }
        }

        cmd_buf_builder = cmd_buf_builder.end_render_pass().unwrap();

        // uniforms usualy change between frames, no point caching them between
        // frames
        self.collection_cache.clear();

        let final_cmd_buf = cmd_buf_builder.build().unwrap();

        Box::new(
            future
                .then_execute(self.queue.clone(), final_cmd_buf)
                .unwrap(),
        )
    }

    pub fn render_to_window(&mut self, window: &mut Window, objects: HashMap<&str, Vec<RenderableObject>>) {
        let swapchain_image = window.next_image();
        let swapchain_fut = window.get_future();

        // render returns a future representing the completion of rendering
        let frame_fut = self.render(
            SwapchainImage::dimensions(&swapchain_image),
            objects,
            swapchain_image,
            swapchain_fut,
        );

        window.present_future(frame_fut);
    }

    pub fn pipeline_for_spec(
        &mut self,
        pass_idx: usize,
        spec: &PipelineSpec,
    ) -> Arc<dyn GraphicsPipelineAbstract + Send + Sync> {
        self.pipeline_caches[pass_idx].get(spec)
    }

    pub fn get_passes(&self) -> &[Pass] {
        &self.passes
    }

    pub fn print_stats(&self) {
        (0..self.passes.len()).for_each(|idx| {
            println!("Pipeline cache stats for pass {}:", self.passes[idx].name);
            self.pipeline_caches[idx].print_stats();
            println!();
            println!("Collection cache stats:");
            self.collection_cache.print_stats();
            println!();
            println!();
        })
    }
}

#[derive(Clone)]
pub struct RenderableObject {
    pub pipeline_spec: PipelineSpec,
    pub vbuf: Arc<dyn BufferAccess + Send + Sync>,
    pub ibuf: Arc<ImmutableBuffer<[u32]>>,
    pub custom_set: Option<Arc<dyn DescriptorSet + Send + Sync>>,
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
        for (image_idx, &image_tag) in pass.images_created_tags.iter().enumerate() {
            let desc = pass
                .render_pass
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
        let images_tags_created = &pass.images_created_tags;
        let images = images_tags_created
            .iter()
            .map(|tag| {
                images
                    .get(tag)
                    .expect("Couldn't get image when creating framebuffers for passes")
                    .clone()
            })
            .collect();

        let framebuffer = fb_from_images(pass.render_pass.clone(), images);
        framebuffers.push(framebuffer);
    }

    framebuffers
}

fn pipe_caches_for_passes(device: Arc<Device>, passes: &[Pass]) -> Vec<PipelineCache> {
    passes
        .iter()
        .map(|pass| PipelineCache::new(device.clone(), pass.render_pass.clone()))
        .collect()
}
