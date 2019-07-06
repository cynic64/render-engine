use super::*;

extern crate nalgebra_glm as glm;

type ConcreteGraphicsPipeline = GraphicsPipeline<
    SingleBufferDefinition<Vertex>,
    Box<PipelineLayoutAbstract + Send + Sync + 'static>,
    Arc<RenderPassAbstract + Send + Sync + 'static>,
>;

mod camera;
use camera::Camera;

pub struct App {
    instance: Arc<Instance>,
    pub events_loop: EventsLoop,
    surface: Arc<Surface<Window>>,
    physical_device_index: usize,
    device: Arc<Device>,
    queue: Arc<Queue>,
    swapchain: Option<Arc<Swapchain<Window>>>,
    images: Vec<Arc<SwapchainImage<Window>>>,
    renderpass: Arc<RenderPassAbstract + Send + Sync>,
    pipeline: Arc<ConcreteGraphicsPipeline>,
    dynamic_state: DynamicState,
    framebuffers: Vec<Arc<FramebufferAbstract + Send + Sync>>,
    must_rebuild_swapchain: bool,
    previous_frame_end: Box<GpuFuture>,
    pub done: bool,
    pub dimensions: [u32; 2],
    vertex_buffers: Vec<Arc<VertexBuffer>>,
    frame_data: FrameData,
    pub unprocessed_events: Vec<VirtualKeyCode>,
    start_time: std::time::Instant,
    frames_drawn: u32,
    vbuf_creator: VbufCreator,
    swapchain_caps: vulkano::swapchain::Capabilities,
    image_format: vulkano::format::Format,
    multisampling_enabled: bool,
    vertex_shader: vs::Shader,
    fragment_shader: fs::Shader,
    available_renderpasses: AvailableRenderPasses,
    // MVP
    model: glm::Mat4,
    view: [[f32; 4]; 4],
    projection: glm::Mat4,
    uniform_buffer: vulkano::buffer::cpu_pool::CpuBufferPool<vs::ty::Data>,
    camera: Camera,
}

struct AvailableRenderPasses {
    multisampled_renderpass: Arc<RenderPassAbstract + Send + Sync>,
    standard_renderpass: Arc<RenderPassAbstract + Send + Sync>,
}

struct FrameData {
    image_num: Option<usize>,
    acquire_future: Option<vulkano::swapchain::SwapchainAcquireFuture<Window>>,
    command_buffer: Option<AutoCommandBuffer>,
}

impl App {
    pub fn new() -> Self {
        let instance = get_instance();
        let physical = get_physical_device(&instance);

        // The objective of this example is to draw a triangle on a window. To do so, we first need to
        // create the window.
        //
        // This is done by creating a `WindowBuilder` from the `winit` crate, then calling the
        // `build_vk_surface` method provided by the `VkSurfaceBuild` trait from `vulkano_win`. If you
        // ever get an error about `build_vk_surface` being undefined in one of your projects, this
        // probably means that you forgot to import this trait.
        //
        // This returns a `vulkano::swapchain::Surface` object that contains both a cross-platform winit
        // window and a cross-platform Vulkan surface that represents the surface of the window.
        let events_loop = EventsLoop::new();
        let surface = WindowBuilder::new()
            .build_vk_surface(&events_loop, instance.clone())
            .unwrap();

        let (device, mut queues) = get_device_and_queues(physical, surface.clone());

        // Since we can request multiple queues, the `queues` variable is in fact an iterator. In this
        // example we use only one queue, so we just retrieve the first and only element of the
        // iterator and throw it away.
        let queue = queues.next().unwrap();

        // We don't need to initialize the swapchain or images write now because by setting must_rebuild_swapchain
        // to true, they will be automatically rebuilt before the first frame is drawn.
        let swapchain = None;
        let images = vec![];
        let must_rebuild_swapchain = true;

        // the user can later enable multisampling with app.enable_multisampling()
        let multisampling_enabled = false;

        // The next step is to create the shaders.
        //
        // The raw shader creation API provided by the vulkano library is unsafe, for various reasons.
        //
        // An overview of what the `vulkano_shaders::shader!` macro generates can be found in the
        // `vulkano-shaders` crate docs. You can view them at https://docs.rs/vulkano-shaders/
        //
        // TODO: explain this in details
        let vs = vs::Shader::load(device.clone()).unwrap();
        let fs = fs::Shader::load(device.clone()).unwrap();

        // At this point, OpenGL initialization would be finished. However in Vulkan it is not. OpenGL
        // implicitly does a lot of computation whenever you draw. In Vulkan, you have to do all this
        // manually.

        let swapchain_caps = surface.capabilities(physical).unwrap();
        // on my machine this is B8G8R8Unorm
        let image_format = swapchain_caps.supported_formats[0].0;
        let dimensions = swapchain_caps.current_extent.unwrap_or([1024, 768]);

        let available_renderpasses = create_available_renderpasses(device.clone(), image_format);

        // default to using the standard renderpass. The only other option (for now)
        // is the multisampled one.
        let renderpass = available_renderpasses.standard_renderpass.clone();

        // Before we draw we have to create what is called a pipeline. This is similar to an OpenGL
        // program, but much more specific.
        let pipeline = Arc::new(
            GraphicsPipeline::start()
                // We need to indicate the layout of the vertices.
                // The type `SingleBufferDefinition` actually contains a template parameter corresponding
                // to the type of each vertex. But in this code it is automatically inferred.
                .vertex_input_single_buffer()
                // A Vulkan shader can in theory contain multiple entry points, so we have to specify
                // which one. The `main` word of `main_entry_point` actually corresponds to the name of
                // the entry point.
                .vertex_shader(vs.main_entry_point(), ())
                // The content of the vertex buffer describes a list of triangles.
                .triangle_list()
                // Use a resizable viewport set to draw over the entire window
                .viewports_dynamic_scissors_irrelevant(1)
                // See `vertex_shader`.
                .fragment_shader(fs.main_entry_point(), ())
                // We have to indicate which subpass of which render pass this pipeline is going to be used
                // in. The pipeline will only be usable from this particular subpass.
                .render_pass(Subpass::from(renderpass.clone(), 0).unwrap())
                // Now that our builder is filled, we call `build()` to obtain an actual pipeline.
                .build(device.clone())
                .unwrap(),
        );

        // Dynamic viewports allow us to recreate just the viewport when the window is resized
        // Otherwise we would have to recreate the whole pipeline.
        let viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions: [dimensions[0] as f32, dimensions[1] as f32],
            depth_range: 0.0..1.0,
        };
        let dynamic_state = DynamicState {
            line_width: None,
            viewports: Some(
                vec![viewport]),
            scissors: None,
        };

        // The render pass we created above only describes the layout of our framebuffers. Before we
        // can draw we also need to create the actual framebuffers.
        //
        // Since we need to draw to multiple images, we are going to create a different framebuffer for
        // each image.
        let framebuffers = vec![];

        // Initialization is finally finished!

        // In the loop below we are going to submit commands to the GPU. Submitting a command produces
        // an object that implements the `GpuFuture` trait, which holds the resources for as long as
        // they are in use by the GPU.
        //
        // Destroying the `GpuFuture` blocks until the GPU is finished executing it. In order to avoid
        // that, we store the submission of the previous frame here.
        let previous_frame_end = Box::new(sync::now(device.clone())) as Box<GpuFuture>;

        let vbuf_creator = VbufCreator::new(device.clone());

        // mvp
        let camera = Camera::default();
        let model = glm::scale(&glm::Mat4::identity(), &glm::vec3(1.0, 1.0, 1.0));
        let view: [[f32; 4]; 4] = camera.get_view_matrix().into();
        let projection = glm::perspective(
            // aspect ratio
            16. / 9.,
            // fov
            1.0,
            // near
            0.1,
            // far
            100_000_000.,
        );

        let uniform_buffer = vulkano::buffer::cpu_pool::CpuBufferPool::<vs::ty::Data>::new(
            device.clone(),
            vulkano::buffer::BufferUsage::all(),
        );

        App {
            instance: instance.clone(),
            events_loop,
            surface,
            physical_device_index: physical.index(),
            device,
            queue,
            swapchain,
            images,
            renderpass,
            pipeline,
            dynamic_state,
            framebuffers,
            must_rebuild_swapchain,
            previous_frame_end,
            done: false,
            dimensions: [0, 0],
            vertex_buffers: vec![],
            unprocessed_events: vec![],
            frame_data: FrameData {
                image_num: None,
                acquire_future: None,
                command_buffer: None,
            },
            start_time: std::time::Instant::now(),
            frames_drawn: 0,
            vbuf_creator,
            swapchain_caps,
            image_format,
            multisampling_enabled,
            vertex_shader: vs,
            fragment_shader: fs,
            available_renderpasses,
            model,
            view,
            projection,
            uniform_buffer,
            camera,
        }
    }

    pub fn enable_multisampling(&mut self) {
        self.multisampling_enabled = true;
        self.update_dimensions();
        self.renderpass = self.available_renderpasses.multisampled_renderpass.clone();
        self.rebuild_pipeline();
        self.rebuild_swapchain();
    }

    pub fn disable_multisampling(&mut self) {
        self.multisampling_enabled = false;
        self.update_dimensions();
        self.renderpass = self.available_renderpasses.standard_renderpass.clone();
        self.rebuild_pipeline();
        self.rebuild_swapchain();
    }

    pub fn create_new_vbuf_creator(&self) -> VbufCreator {
        VbufCreator::new(self.device.clone())
    }

    pub fn clear_vertex_buffers(&mut self) {
        self.vertex_buffers = vec![];
    }

    pub fn new_vbuf_from_verts(&mut self, verts: &[Vertex]) {
        // creates a new vertex buffer from the given vertices and appends it to the list of vertices
        let vertex_buffer = self.vbuf_creator.create_vbuf_from_verts(verts);
        self.vertex_buffers.push(vertex_buffer);
    }

    pub fn draw_frame(&mut self) {
        self.clear_unprocessed_events();
        self.setup_frame();

        self.create_command_buffer();
        self.submit_and_check();

        self.handle_input();
        self.frames_drawn += 1;
    }

    pub fn vert_from_pixel_coords(&self, pixel: &PixelCoord, color: [f32; 4]) -> Vertex {
        // to convert pixel to screen coordinate (-1..1), divide by resolution (-1..1) -> (0..1),
        // multiply by 2 (0..1) -> (0..2) and subtract 1 (0..2) -> (-1..1)
        let screen_x = (pixel.x as f32) / (self.dimensions[0] as f32) * 2.0 - 1.0;
        let screen_y = (pixel.y as f32) / (self.dimensions[1] as f32) * 2.0 - 1.0;

        Vertex {
            position: [screen_x, screen_y, 0.0],
            color,
        }
    }

    pub fn print_fps(&self) {
        let fps = (self.frames_drawn as f32) / get_elapsed(self.start_time);
        println!("FPS: {}", fps);
    }

    fn setup_frame(&mut self) {
        // wipes frame_data, then brings everything up to the point where vertex buffers can be
        // created and the command buffer can be submitted.
        self.update_dimensions();

        self.frame_data = FrameData {
            image_num: None,
            acquire_future: None,
            command_buffer: None,
        };

        self.free_unused_resources();

        // Whenever the window resizes we need to recreate everything dependent on the window size.
        // In this example that includes the swapchain, the framebuffers and the dynamic state viewport.
        if self.must_rebuild_swapchain {
            self.rebuild_swapchain();
            self.must_rebuild_swapchain = false;
        }

        // Before we can draw on the output, we have to *acquire* an image from the swapchain. If
        // no image is available (which happens if you submit draw commands too quickly), then the
        // function will block.
        // This operation returns the index of the image that we are allowed to draw upon.
        //
        // This function can block if no image is available. The parameter is an optional timeout
        // after which the function call will return an error.
        self.acquire_next_image();
    }

    pub fn handle_input(&mut self) {
        let mut done = false;
        let mut must_rebuild_swapchain = false;
        let mut unprocessed_events = vec![];
        let mut x_movement = 0.0;
        let mut y_movement = 0.0;

        // for avoiding closure borrow problems
        let dimensions = self.dimensions;

        self.events_loop.poll_events(|ev| match ev {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => done = true,
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => must_rebuild_swapchain = true,

            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position: p, .. },
                ..
            } => {
                let (x_diff, y_diff) = (
                    p.x - (dimensions[0] as f64 / 2.0),
                    p.y - (dimensions[1] as f64 / 2.0),
                );
                x_movement = x_diff as f32;
                y_movement = y_diff as f32;
            },

            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { .. },
                ..
            } => {
                if let Some(keycode) = winit_event_to_keycode(ev) {
                    unprocessed_events.push(keycode);
                }
            }
            _ => (),
        });

        // for avoiding problems with borrow checker
        unprocessed_events
            .iter()
            .for_each(|&keycode| self.unprocessed_events.push(keycode));
        self.must_rebuild_swapchain = must_rebuild_swapchain;
        self.done = done;

        // reset cursor and change camera view
        self.surface
            .window()
            .set_cursor_position(winit::dpi::LogicalPosition {
                x: self.dimensions[0] as f64 / 2.0,
                y: self.dimensions[1] as f64 / 2.0,
            })
            .expect("Couldn't re-set cursor position!");
        self.camera.mouse_move(x_movement as f32, y_movement as f32);
        self.view = self.camera.get_view_matrix().into();
    }

    fn clear_unprocessed_events(&mut self) {
        self.unprocessed_events = vec![];
    }

    fn create_command_buffer(&mut self) {
        let uniform_buffer_subbuffer = {
            let uniform_data = vs::ty::Data {
                world: self.model.into(),
                view: self.view,
                proj: self.projection.into(),
            };

            self.uniform_buffer.next(uniform_data).unwrap()
        };

        let uniform_set = Arc::new(
            vulkano::descriptor::descriptor_set::PersistentDescriptorSet::start(
                self.pipeline.clone(),
                0,
            )
            .add_buffer(uniform_buffer_subbuffer)
            .unwrap()
            .build()
            .unwrap(),
        );

        let clear_values = if self.multisampling_enabled {
            vec![[0.2, 0.2, 0.2, 1.0].into(), [0.2, 0.2, 0.2, 1.0].into()]
        } else {
            vec![[0.2, 0.2, 0.2, 1.0].into()]
        };

        let mut command_buffer_unfinished = AutoCommandBufferBuilder::primary_one_time_submit(
            self.device.clone(),
            self.queue.family(),
        )
        .unwrap()
        // Before we can draw, we have to *enter a render pass*. There are two methods to do
        // this: `draw_inline` and `draw_secondary`. The latter is a bit more advanced and is
        // not covered here.
        //
        // The third parameter builds the list of values to clear the attachments with. The API
        // is similar to the list of attachments when building the framebuffers, except that
        // only the attachments that use `load: Clear` appear in the list.
        .begin_render_pass(
            self.framebuffers[self.frame_data.image_num.expect(
                "
---------------------------------------------------------------------------------------------
    [create_command_buffer]    (begin_renderpass)
->  When trying to create the command buffer, found that frame_data.image_num is None.
->  Maybe acquire_next_image was not called.
---------------------------------------------------------------------------------------------
                ",
            )]
            .clone(),
            false,
            clear_values,
        )
        .unwrap();

        // add draw calls for every vertex buffer onto the command buffer
        for vertex_buffer in self.vertex_buffers.iter() {
            // We are now inside the first subpass of the render pass. We add a draw command.
            //
            // The last two parameters contain the list of resources to pass to the shaders.
            // Since we used an `EmptyPipeline` object, the objects have to be `()`.
            command_buffer_unfinished = command_buffer_unfinished
                .draw(
                    self.pipeline.clone(),
                    &self.dynamic_state,
                    vertex_buffer.clone(),
                    uniform_set.clone(),
                    (),
                )
                .unwrap();
        }

        let command_buffer_finished = command_buffer_unfinished
            // We leave the render pass by calling `draw_end`. Note that if we had multiple
            // subpasses we could have called `next_inline` (or `next_secondary`) to jump to the
            // next subpass.
            .end_render_pass()
            .unwrap()
            // Finish building the command buffer by calling `build`.
            .build()
            .unwrap();

        self.frame_data.command_buffer = Some(command_buffer_finished);
    }

    fn submit_and_check(&mut self) {
        let future = self
            .frame_data
            .acquire_future
            .take()
            .expect(
                "
---------------------------------------------------------------------------------------------
    [submit_and_check]    (acquire_future.take())
->  When trying to submit the command buffer and present to the swapchain, found that
->  acquire_future is None.
->  Maybe acquire_next_image was not called.
---------------------------------------------------------------------------------------------
                ",
            )
            .then_execute(
                self.queue.clone(),
                self.frame_data.command_buffer.take().expect(
                    "
---------------------------------------------------------------------------------------------
    [submit_and_check]    (command_buffer.take())
->  When trying to submit the command buffer and present to the swapchain, found that
->  command_buffer is None.
->  Maybe create_command_buffer was not called.
---------------------------------------------------------------------------------------------
                ",
                ),
            )
            .unwrap()
            // The color output is now expected to contain our triangle. But in order to show it on
            // the screen, we have to *present* the image by calling `present`.
            //
            // This function does not actually present the image immediately. Instead it submits a
            // present command at the end of the queue. This means that it will only be presented once
            // the GPU has finished executing the command buffer that draws the triangle.
            .then_swapchain_present(
                self.queue.clone(),
                self.swapchain.as_ref().expect(
                    "
---------------------------------------------------------------------------------------------
    [submit_and_check]    (then_swapchain_present)
-> When trying to submit the command buffer and present it to the swapchain, found that
-> the swapchain does not exist.
-> Unless you're trying to something really weird, the internal implementation probably
-> fucked up, because this shouldn't happen.
---------------------------------------------------------------------------------------------
                    ").clone(),
                self.frame_data.image_num.expect(
                    "
---------------------------------------------------------------------------------------------
    [submit_and_check]    (image_num.expect())
->  When trying to submit the command buffer and present to the swapchain, found that
->  image_num is None.
->  Maybe acquire_next_image was not called.
---------------------------------------------------------------------------------------------
                ",
                ),
            )
            .then_signal_fence_and_flush();

        match future {
            Ok(future) => {
                self.previous_frame_end = Box::new(future) as Box<_>;
            }
            Err(FlushError::OutOfDate) => {
                self.must_rebuild_swapchain = true;
                self.previous_frame_end = Box::new(sync::now(self.device.clone())) as Box<_>;
            }
            Err(e) => {
                println!("{:?}", e);
                self.previous_frame_end = Box::new(sync::now(self.device.clone())) as Box<_>;
            }
        }
    }

    fn free_unused_resources(&mut self) {
        // It is important to call this function from time to time, otherwise resources will keep
        // accumulating and you will eventually reach an out of memory error.
        // Calling this function polls various fences in order to determine what the GPU has
        // already processed, and frees the resources that are no longer needed.
        self.previous_frame_end.cleanup_finished();
    }

    fn rebuild_swapchain(&mut self) {
        self.dynamic_state.viewports = Some(
            vec![
                Viewport {
                    origin: [0.0, 0.0],
                    dimensions: [self.dimensions[0] as f32, self.dimensions[1] as f32],
                    depth_range: 0.0..1.0,
                },
            ]);

        let tuple = match &self.swapchain {
            // the swapchain already exists and is just out of date, meaning we can
            // re-build the old one rather than making a whole new one.
            Some(_swapchain) => self.create_swapchain_and_images_from_existing_swapchain(),
            None => self.create_swapchain_and_images_from_scratch(),
        };

        let new_swapchain: Arc<Swapchain<Window>> = tuple.0;
        let new_images: Vec<Arc<SwapchainImage<Window>>> = tuple.1;

        self.swapchain = Some(new_swapchain);
        self.images = new_images;

        // Because framebuffers contains an Arc on the old swapchain, we need to
        // recreate framebuffers as well.
        self.framebuffers = vec![];
        self.rebuild_framebuffers();
    }

    fn rebuild_framebuffers(&mut self) {
        if self.multisampling_enabled {
            self.framebuffers = self
                .images
                .iter()
                .map(|image| {
                    let multisampled_color =
                        vulkano::image::attachment::AttachmentImage::transient_multisampled(
                            self.device.clone(),
                            self.dimensions,
                            4,
                            self.image_format,
                        )
                        .unwrap();

                    let fba: Arc<vulkano::framebuffer::FramebufferAbstract + Send + Sync> = Arc::new(
                        vulkano::framebuffer::Framebuffer::start(self.renderpass.clone())
                            .add(multisampled_color.clone())
                            .unwrap()
                            .add(image.clone())
                            .unwrap()
                             .build()
                            .unwrap(),
                    );

                    fba
                })
                .collect::<Vec<_>>();
        } else {
            self.framebuffers = self
                .images
                .iter()
                .map(|image| {
                    let fba: Arc<vulkano::framebuffer::FramebufferAbstract + Send + Sync> = Arc::new(
                        vulkano::framebuffer::Framebuffer::start(self.renderpass.clone())
                            .add(image.clone())
                            .unwrap()
                            .build()
                            .unwrap(),
                    );

                    fba
                })
                .collect::<Vec<_>>();
        }
    }

    fn rebuild_pipeline(&mut self) {
        // the purpose of this function is to be called after the render pass or another
        // parameter for the graphics pipeline is changed, and the pipeline must be
        // rebuilt.
        self.pipeline = Arc::new(
            GraphicsPipeline::start()
                .vertex_input_single_buffer()
                .vertex_shader(self.vertex_shader.main_entry_point(), ())
                .triangle_list()
                .viewports_dynamic_scissors_irrelevant(1)
                .fragment_shader(self.fragment_shader.main_entry_point(), ())
                .render_pass(Subpass::from(self.renderpass.clone(), 0).unwrap())
                .build(self.device.clone())
                .unwrap()
        );
    }

    fn get_dimensions(&self) -> Option<[u32; 2]> {
        if let Some(dimensions) = self.surface.window().get_inner_size() {
            let dimensions: (u32, u32) = dimensions
                .to_physical(self.surface.window().get_hidpi_factor())
                .into();

            Some([dimensions.0, dimensions.1])
        } else {
            None
        }
    }

    fn update_dimensions(&mut self) {
        if let Some(dimensions) = self.get_dimensions() {
            self.dimensions = dimensions;
        }
    }

    fn acquire_next_image(&mut self) {
        let (image_num, acquire_future) =
            match swapchain::acquire_next_image(self.swapchain.as_ref().expect(
                "
---------------------------------------------------------------------------------------------
    [acquire_next_image]    (self.swapchain.expect)
-> When trying to acquire the next image, found that the swapchain does not exist.
-> Unless you're trying to something really weird, the internal implementation probably
-> fucked up, because this shouldn't happen.
---------------------------------------------------------------------------------------------
                ").clone(), None) {
                Ok(r) => r,
                Err(AcquireError::OutOfDate) => {
                    println!("Swapchain out of date when trying to acquire next image");
                    self.must_rebuild_swapchain = true;
                    return;
                }
                Err(err) => panic!("{:?}", err),
            };

        self.frame_data.image_num = Some(image_num);
        self.frame_data.acquire_future = Some(acquire_future);
    }

    fn create_swapchain_and_images_from_existing_swapchain(&mut self) -> (Arc<Swapchain<Window>>, Vec<Arc<SwapchainImage<Window>>>) {
        let swapchain = self.swapchain.as_ref().expect(
            "
---------------------------------------------------------------------------------------------
    [create_swapchain_and_images_from_existing_swapchain]    (self.swapchain.expect)
-> When creating a new swapchain from an existing one (usually done because of a window
-> resize), found that the swapchain doesn't exist. You probably fucked up and called this
-> from somewhere where the app had no existing swapchain. Use
-> create_swapchain_and_images_from_scratch for that.
---------------------------------------------------------------------------------------------
            ")
            .clone();

        let mut last_result = None;
        while last_result.is_none() {
            self.update_dimensions();
            last_result = create_swapchain_and_images_from_existing_swapchain(swapchain.clone(), self.dimensions);
        };

        last_result.unwrap()
    }

    fn create_swapchain_and_images_from_scratch(&self) -> (Arc<Swapchain<Window>>, Vec<Arc<SwapchainImage<Window>>>) {
        match Swapchain::new(
            self.device.clone(),
            self.surface.clone(),
            self.swapchain_caps.min_image_count,
            self.image_format,
            self.dimensions,
            1,
            self.swapchain_caps.supported_usage_flags,
            &self.queue,
            SurfaceTransform::Identity,
            self.swapchain_caps.supported_composite_alpha.iter().next().unwrap(),
            PresentMode::Fifo,
            true,
            None,
        ) {
            Ok(r) => r,
            // This error tends to happen when the user is manually resizing the window.
            // Simply restarting the loop is the easiest way to fix this issue.
            Err(SwapchainCreationError::UnsupportedDimensions) => panic!("SwapchainCreationError::UnsupportedDimensions when creating initial swapchain. Should never happen."),
            Err(err) => panic!("{:?}", err),
        }
    }
}

fn get_instance() -> Arc<Instance> {
    // When we create an instance, we have to pass a list of extensions that we want to enable.
    //
    // All the window-drawing functionalities are part of non-core extensions that we need
    // to enable manually. To do so, we ask the `vulkano_win` crate for the list of extensions
    // required to draw to a window.
    let extensions = vulkano_win::required_extensions();

    // Now creating the instance.
    Instance::new(None, &extensions, None).unwrap()
}

fn get_physical_device(instance: &Arc<Instance>) -> PhysicalDevice {
    // In a real application, there are three things to take into consideration:
    //
    // - Some devices may not support some of the optional features that may be required by your
    //   application. You should filter out the devices that don't support your app.
    //
    // - Not all devices can draw to a certain surface. Once you create your window, you have to
    //   choose a device that is capable of drawing to it.
    //
    // - You probably want to leave the choice between the remaining devices to the user.
    //
    // For the sake of the example we are just going to use the first device, which should work
    // most of the time.
    PhysicalDevice::enumerate(&instance).next().unwrap()
}

fn get_device_and_queues(
    physical: PhysicalDevice,
    surface: Arc<Surface<Window>>,
) -> (Arc<Device>, vulkano::device::QueuesIter) {
    // The next step is to choose which GPU queue will execute our draw commands.
    //
    // Devices can provide multiple queues to run commands in parallel (for example a draw queue
    // and a compute queue), similar to CPU threads. This is something you have to have to manage
    // manually in Vulkan.
    //
    // In a real-life application, we would probably use at least a graphics queue and a transfers
    // queue to handle data transfers in parallel. In this example we only use one queue.
    //
    // We have to choose which queues to use early on, because we will need this info very soon.
    let queue_family = physical
        .queue_families()
        .find(|&q| {
            // We take the first queue that supports drawing to our window.
            q.supports_graphics() && surface.is_supported(q).unwrap_or(false)
        })
        .unwrap();

    // Now initializing the device. This is probably the most important object of Vulkan.
    //
    // We have to pass five parameters when creating a device:
    //
    // - Which physical device to connect to.
    //
    // - A list of optional features and extensions that our program needs to work correctly.
    //   Some parts of the Vulkan specs are optional and must be enabled manually at device
    //   creation. In this example the only thing we are going to need is the `khr_swapchain`
    //   extension that allows us to draw to a window.
    //
    // - A list of layers to enable. This is very niche, and you will usually pass `None`.
    //
    // - The list of queues that we are going to use. The exact parameter is an iterator whose
    //   items are `(Queue, f32)` where the floating-point represents the priority of the queue
    //   between 0.0 and 1.0. The priority of the queue is a hint to the implementation about how
    //   much it should prioritize queues between one another.
    //
    // The list of created queues is returned by the function alongside with the device.
    let device_ext = DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::none()
    };
    // let (device, mut queues) = Device::new(physical, physical.supported_features(), &device_ext,
    //     [(queue_family, 0.5)].iter().cloned()).unwrap();
    Device::new(
        physical,
        physical.supported_features(),
        &device_ext,
        [(queue_family, 0.5)].iter().cloned(),
    )
    .unwrap()
}

fn create_swapchain_and_images_from_existing_swapchain(old_swapchain: Arc<Swapchain<Window>>, dimensions: [u32; 2]) -> Option<(Arc<Swapchain<Window>>, Vec<Arc<SwapchainImage<Window>>>)> {
    match old_swapchain.recreate_with_dimension(dimensions) {
        Ok(r) => Some(r),
        Err(SwapchainCreationError::UnsupportedDimensions) => {
            // this happens sometimes :\
            println!("Unsupported dimensions: {:?}", dimensions);
            None
        },
        Err(err) => panic!("{:?}", err),
    }
}

fn create_available_renderpasses(device: Arc<Device>, format: vulkano::format::Format) -> AvailableRenderPasses {
    let multisampled_renderpass = Arc::new(
        vulkano::single_pass_renderpass!(
            device.clone(),
            attachments: {
                multisampled_color: {
                    load: Clear,
                    store: DontCare,
                    format: format,
                    samples: 4,
                },
                resolve_color: {
                    load: Clear,
                    store: Store,
                    format: format,
                    samples: 1,
                }
            },
            pass: {
                color: [multisampled_color],
                depth_stencil: {},
                resolve: [resolve_color]
            }
        ).unwrap()
    );

    let standard_renderpass: Arc<RenderPassAbstract + Send + Sync> = Arc::new(
        vulkano::single_pass_renderpass!(
            device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: format,
                    samples: 1,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {}
            }
        )
        .unwrap(),
    );

    AvailableRenderPasses {
        multisampled_renderpass,
        standard_renderpass,
    }
}

mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "
            #version 450

            layout(location = 0) in vec3 position;
            layout(location = 1) in vec4 color;
            layout(location = 0) out vec4 v_color;

            layout(set = 0, binding = 0) uniform Data {
                mat4 world;
                mat4 view;
                mat4 proj;
            } uniforms;

            void main() {
                mat4 worldview = uniforms.view * uniforms.world;
                gl_Position = uniforms.proj * worldview * vec4(position, 1.0);
                v_color = color;
            }"
    }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "
            #version 450

            layout(location = 0) in vec4 v_color;
            layout(location = 0) out vec4 f_color;

            void main() {
                f_color = v_color;
            }
            "
    }
}
