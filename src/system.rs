use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState};
use vulkano::device::{Device, Queue};
use vulkano::framebuffer::{
    AttachmentDescription, Framebuffer, FramebufferAbstract, RenderPassAbstract,
};
use vulkano::image::{AttachmentImage, ImageViewAccess, SwapchainImage};
use vulkano::pipeline::viewport::Viewport;
use vulkano::sync::GpuFuture;

use std::collections::HashMap;
use std::sync::Arc;

use crate::collection_cache::CollectionCache;
use crate::object::Drawcall;
use crate::pipeline_cache::PipelineCache;
use crate::render_passes::clear_values_for_pass;
use crate::utils::Timer;
use crate::window::Window;

// TODO: make the whole thing less prone to runtime panics. vecs of strings are
// a little sketchy. Maybe make a function that checks the system to ensure
// it'll work?

// A system is a list of passes that takes a bunch of data and produces a frame
// for it.
pub struct System<'a> {
    pub passes: Vec<Pass<'a>>,
    pipeline_caches: Vec<PipelineCache>,
    collection_cache: CollectionCache,
    // stores the vbuf of the screen-filling square used for non-geometry passes
    device: Arc<Device>,
    queue: Arc<Queue>,
    pub output_tag: &'a str,
    cached_images: Option<HashMap<String, Arc<dyn ImageViewAccess + Send + Sync>>>,
    pub custom_images: HashMap<&'a str, Arc<dyn ImageViewAccess + Send + Sync>>,
    partial: Option<PartialRender>,
    pass_timers: Vec<Timer>,
    cmd_buf_timer: Timer,
    present_timer: Timer,
    setup_timer: Timer,
}

struct PartialRender {
    cmd_buf: AutoCommandBufferBuilder,
    images: HashMap<String, Arc<dyn ImageViewAccess + Send + Sync>>,
    framebuffers: Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,
    current_pass: Option<PartialPass>,
}

struct PartialPass {
    dims: [u32; 2],
    pass_idx: usize,
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
    pub fn new(
        queue: Arc<Queue>,
        passes: Vec<Pass<'a>>,
        custom_images: HashMap<&'a str, Arc<dyn ImageViewAccess + Send + Sync>>,
        output_tag: &'a str,
    ) -> Self {
        let device = queue.device().clone();

        let pipeline_caches = pipe_caches_for_passes(device.clone(), &passes);
        let collection_cache = CollectionCache::new(device.clone());
        let pass_timers = passes.iter().map(|pass| Timer::new(pass.name)).collect();

        Self {
            passes,
            pipeline_caches,
            collection_cache,
            device,
            queue,
            output_tag,
            cached_images: None,
            custom_images,
            partial: None,
            pass_timers,
            cmd_buf_timer: Timer::new("command buffer"),
            present_timer: Timer::new("present to window"),
            setup_timer: Timer::new("pass setup"),
        }
    }

    pub fn begin_render(
        &mut self,
        dimensions: [u32; 2],
        dest_image: Arc<dyn ImageViewAccess + Send + Sync>,
    ) {
        self.setup_timer.start();

        // create all images and framebuffers
        let mut images = self.get_images(dimensions);

        // replace destination image with the real one
        images.insert(self.output_tag.to_string(), dest_image);

        // use any custom images to replace existing ones
        for (tag, image) in self.custom_images.iter() {
            images.insert(tag.to_string(), image.clone());
        }

        let framebuffers = framebuffers_for_passes(images.clone(), &self.passes);

        // create the command buffer
        let cmd_buf_builder = AutoCommandBufferBuilder::primary_one_time_submit(
            self.device.clone(),
            self.queue.family(),
        )
        .unwrap();

        // save state
        let partial = PartialRender {
            cmd_buf: cmd_buf_builder,
            images,
            framebuffers,
            current_pass: None,
        };

        self.partial = Some(partial);

        self.setup_timer.stop();
        self.cmd_buf_timer.start();
    }

    pub fn begin_render_window(&mut self, window: &mut Window) {
        let swapchain_image = window.next_image();
        let dims = SwapchainImage::dimensions(&swapchain_image);
        self.begin_render(dims, swapchain_image);
    }

    pub fn next_pass(&mut self) {
        let mut end_prev_pass = false;
        let new_pass_idx = match self.partial {
            Some(PartialRender {
                current_pass: Some(PartialPass { pass_idx, .. }),
                ..
            }) => {
                end_prev_pass = true;
                pass_idx + 1
            },
            _ => 0,
        };

        let pass = &self.passes[new_pass_idx];

        // self.pass_timers[new_pass_idx].start();
        let mut partial = self.partial.take().unwrap();

        let pass_last_img_tag = pass
            .images_created_tags
            .iter()
            .last()
            .expect("no images created in pass");
        let last_img = partial
            .images
            .get(&pass_last_img_tag.to_string())
            .unwrap();
        let pass_dims = [
            last_img.dimensions().width(),
            last_img.dimensions().height(),
        ];

        let framebuffer = partial.framebuffers[new_pass_idx].clone();

        let clear_values = clear_values_for_pass(pass.render_pass.clone());

        if end_prev_pass {
            partial.cmd_buf = partial.cmd_buf.end_render_pass().unwrap();
        }

        partial.cmd_buf = partial
            .cmd_buf
            .begin_render_pass(framebuffer.clone(), false, clear_values)
            .unwrap();

        partial.current_pass = Some(PartialPass {
            dims: pass_dims,
            pass_idx: new_pass_idx,
        });

        self.partial = Some(partial);
    }

    pub fn draw_object<T: Drawcall>(&mut self, object: T) {
        let mut partial = self.partial.take().unwrap();
        let cur_pass = partial.current_pass.take().unwrap();

        let pass_idx = cur_pass.pass_idx;
        let pass = &self.passes[pass_idx];
        let images = &partial.images;

        // TODO: dynamic state is re-created for every object, shouldn't be
        let dynamic_state = if let Some(dynstate) = object.custom_dynstate() {
            dynstate
        } else {
            dynamic_state_for_dimensions(cur_pass.dims)
        };

        let pipeline = self.pipeline_caches[pass_idx].get(object.pipe_spec());

        let mut collection =
            self.collection_cache
                .get(object.pipe_spec(), pipeline.clone(), &pass, &images);

        let mut object_sets =
            object.collection(self.queue.clone(), pipeline.clone(), collection.len());

        collection.append(&mut object_sets);

        partial.cmd_buf = partial.cmd_buf
            .draw_indexed(
                pipeline,
                &dynamic_state,
                vec![object.vbuf()],
                object.ibuf(),
                collection,
                (),
            )
            .expect(&format!("error building cmd buf, in pass {}", pass.name));

        partial.current_pass = Some(cur_pass);

        self.partial = Some(partial);
    }

    pub fn finish_render<F: GpuFuture + 'static>(&mut self, future: F) -> Box<dyn GpuFuture> {
        let partial = self.partial.take().unwrap();
        let final_cmd_buf = partial.cmd_buf
            .end_render_pass()
            .unwrap()
            .build()
            .unwrap();

        Box::new(
            future
                .then_execute(self.queue.clone(), final_cmd_buf)
                .unwrap(),
        )
    }

    pub fn finish_to_window(&mut self, window: &mut Window) {
        let swapchain_fut = window.get_future();

        // render returns a future representing the completion of rendering
        let frame_fut = self.finish_render(
            swapchain_fut,
        );

        self.present_timer.start();
        window.present_future(frame_fut);
        self.present_timer.stop();
    }

    pub fn get_passes(&self) -> &[Pass] {
        &self.passes
    }

    pub fn print_stats(&self) {
        println!();

        self.cmd_buf_timer.print();
        self.present_timer.print();
        self.setup_timer.print();
        self.pass_timers.iter().for_each(|timer| timer.print());

        println!();

        (0..self.passes.len()).for_each(|idx| {
            println!("Pipeline cache stats for pass {}:", self.passes[idx].name);
            self.pipeline_caches[idx].print_stats();
            println!();
            println!();
        });

        println!();
    }

    fn get_images(
        &mut self,
        dimensions: [u32; 2],
    ) -> HashMap<String, Arc<dyn ImageViewAccess + Send + Sync>> {
        // gets images to be drawn to either by using cached ones or creating
        // new ones

        // if there is a cache, make sure its dimensions are the same as what we want
        if let Some(cached) = &self.cached_images {
            let cached_vk_dims = cached.get(self.output_tag).unwrap().dimensions();
            let cached_dimensions = [cached_vk_dims.width(), cached_vk_dims.height()];

            if cached_dimensions != dimensions {
                self.cached_images = None;
            }
        }

        if let Some(cached) = &self.cached_images {
            cached.clone()
        } else {
            let new = images_for_passes(self.device.clone(), dimensions, &self.passes);
            self.cached_images = Some(new.clone());
            new
        }
    }
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
) -> HashMap<String, Arc<dyn ImageViewAccess + Send + Sync>> {
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

            // FIXME: yeah this needs a better solution
            let image = if image_tag.contains("lowres") {
                create_image_for_desc(device.clone(), [512, 512], desc)
            } else {
                create_image_for_desc(device.clone(), dimensions, desc)
            };

            images.insert(image_tag.to_string(), image);
        }
    }

    images
}

fn framebuffers_for_passes<'a>(
    images: HashMap<String, Arc<dyn ImageViewAccess + Send + Sync>>,
    passes: &'a [Pass],
) -> Vec<Arc<dyn FramebufferAbstract + Send + Sync>> {
    let mut framebuffers = vec![];

    for pass in passes.iter() {
        let images_tags_created = &pass.images_created_tags;
        let images = images_tags_created
            .iter()
            .map(|tag| {
                images
                    .get(&tag.to_string())
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
