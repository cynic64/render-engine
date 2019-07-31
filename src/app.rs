extern crate nalgebra_glm as glm;

use crate::camera::*;
use crate::exposed_tools::*;
use crate::internal_tools::*;
use crate::world::*;
use crate::render_passes;

pub struct App {
    instance: Arc<Instance>,
    events_loop: EventsLoop,
    physical_device_index: usize,
    device: Arc<Device>,
    queue: Arc<Queue>,
    render_pass: Arc<RenderPassAbstract + Send + Sync>,
    pub done: bool,
    pub dimensions: [u32; 2],
    command_buffer: Option<AutoCommandBuffer>,
    pub unprocessed_events: Vec<Event>,
    pub unprocessed_keydown_events: Vec<VirtualKeyCode>,
    pub unprocessed_keyup_events: Vec<VirtualKeyCode>,
    pub keys_down: KeysDown,
    pub delta: f32,
    last_frame_time: std::time::Instant,
    start_time: std::time::Instant,
    frames_drawn: u32,
    multisampling_enabled: bool,
    // MVP
    world: World,
    vk_window: ll::vk_window::VkWindow,
}

const MULTISAMPLING_FACTOR: u32 = 4;

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

        surface.window().hide_cursor(true);

        let (device, mut queues) = get_device_and_queues(physical, surface.clone());

        // Since we can request multiple queues, the `queues` variable is in fact an iterator. In this
        // example we use only one queue, so we just retrieve the first and only element of the
        // iterator and throw it away.
        let queue = queues.next().unwrap();

        // the user can later enable multisampling with app.enable_multisampling()
        let multisampling_enabled = false;

        // At this point, OpenGL initialization would be finished. However in Vulkan it is not. OpenGL
        // implicitly does a lot of computation whenever you draw. In Vulkan, you have to do all this
        // manually.

        let swapchain_caps = surface.capabilities(physical).unwrap();
        // on my machine this is B8G8R8Unorm
        // default to using the standard render_pass. The only other option (for now)
        // is the multisampled one.
        let render_pass = render_passes::basic(device.clone());

        let vk_window = ll::vk_window::VkWindow::new(
            device.clone(),
            queue.clone(),
            surface.clone(),
            render_pass.clone(),
            swapchain_caps.clone(),
        );
        let dimensions = vk_window.get_dimensions();

        // Initialization is finally finished!

        // In the loop below we are going to submit commands to the GPU. Submitting a command produces
        // an object that implements the `GpuFuture` trait, which holds the resources for as long as
        // they are in use by the GPU.

        let keys_down = KeysDown::all_false();

        let camera = OrbitCamera::default();

        let world = World::new(render_pass.clone(), device.clone(), Box::new(camera));

        Self {
            instance: instance.clone(),
            events_loop,
            physical_device_index: physical.index(),
            device,
            queue,
            render_pass,
            done: false,
            dimensions,
            command_buffer: None,
            unprocessed_events: vec![],
            unprocessed_keydown_events: vec![],
            unprocessed_keyup_events: vec![],
            keys_down,
            delta: 0.0,
            last_frame_time: std::time::Instant::now(),
            start_time: std::time::Instant::now(),
            frames_drawn: 0,
            multisampling_enabled,
            world,
            vk_window,
        }
    }

    pub fn update_camera(&mut self, camera: Box<Camera>) {
        self.world.update_camera(camera);
    }

    pub fn get_world_com(&self) -> WorldCommunicator {
        self.world.get_communicator()
    }

    pub fn enable_multisampling(&mut self) {
        self.multisampling_enabled = true;
        self.render_pass = render_passes::multisampled_with_depth(self.device.clone(), MULTISAMPLING_FACTOR);
        self.update_render_pass();
    }

    pub fn disable_multisampling(&mut self) {
        self.multisampling_enabled = false;
        self.render_pass = render_passes::with_depth(self.device.clone());
        self.update_render_pass();
    }

    pub fn set_render_pass(&mut self, render_pass: Arc<RenderPassAbstract + Send + Sync>) {
        self.render_pass = render_pass;
        self.update_render_pass();
    }

    fn update_render_pass(&mut self) {
        // call this whenever you change the renderr pass
        self.vk_window.update_render_pass(self.render_pass.clone());
        self.vk_window.rebuild();
        self.world.update_render_pass(self.render_pass.clone());
    }

    pub fn draw_frame(&mut self) {
        self.clear_unprocessed_events();
        self.setup_frame();

        self.create_command_buffer();
        self.submit_and_check();

        self.delta = get_elapsed(self.last_frame_time);
        self.handle_input();
        self.update_world();
        self.last_frame_time = std::time::Instant::now();
        self.frames_drawn += 1;
    }

    fn update_world(&mut self) {
        self.world.update(
            &self.unprocessed_events,
            &self.keys_down,
            self.delta,
            self.vk_window.get_dimensions(),
        );
    }

    pub fn print_fps(&self) {
        let fps = (self.frames_drawn as f32) / get_elapsed(self.start_time);
        println!("FPS: {}", fps);
    }

    pub fn get_device(&self) -> Arc<Device> {
        self.device.clone()
    }

    fn setup_frame(&mut self) {
        self.dimensions = self.vk_window.get_dimensions();

        // it should always be none before drawing the frame anyway, but just make sure
        self.command_buffer = None;
    }

    pub fn handle_input(&mut self) {
        let mut done = false;
        let mut unprocessed_keydown_events = vec![];
        let mut unprocessed_keyup_events = vec![];
        let mut unprocessed_events = vec![];

        self.events_loop.poll_events(|ev| {
            match ev.clone() {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => done = true,
                Event::WindowEvent {
                    event: WindowEvent::KeyboardInput { .. },
                    ..
                } => {
                    if let Some(keyboard_input) = winit_event_to_keycode(&ev) {
                        match keyboard_input {
                            KeyboardInput {
                                virtual_keycode: Some(key),
                                state: winit::ElementState::Pressed,
                                ..
                            } => unprocessed_keydown_events.push(key),
                            KeyboardInput {
                                virtual_keycode: Some(key),
                                state: winit::ElementState::Released,
                                ..
                            } => unprocessed_keyup_events.push(key),
                            _ => {}
                        }
                    }
                }
                _ => {}
            };
            unprocessed_events.push(ev.clone());
        });

        // for avoiding problems with borrow checker
        // append all new keydown events to the list, as well as updating keys_down
        unprocessed_keydown_events.iter().for_each(|&keycode| {
            self.unprocessed_keydown_events.push(keycode);
            // yeah, this sucks
            // a possible solution: make keys_down a list of VirtualKeyCodes instead
            match keycode {
                VirtualKeyCode::A => self.keys_down.a = true,
                VirtualKeyCode::B => self.keys_down.b = true,
                VirtualKeyCode::C => self.keys_down.c = true,
                VirtualKeyCode::D => self.keys_down.d = true,
                VirtualKeyCode::E => self.keys_down.e = true,
                VirtualKeyCode::F => self.keys_down.f = true,
                VirtualKeyCode::G => self.keys_down.g = true,
                VirtualKeyCode::H => self.keys_down.h = true,
                VirtualKeyCode::I => self.keys_down.i = true,
                VirtualKeyCode::J => self.keys_down.j = true,
                VirtualKeyCode::K => self.keys_down.k = true,
                VirtualKeyCode::L => self.keys_down.l = true,
                VirtualKeyCode::M => self.keys_down.m = true,
                VirtualKeyCode::N => self.keys_down.n = true,
                VirtualKeyCode::O => self.keys_down.o = true,
                VirtualKeyCode::P => self.keys_down.p = true,
                VirtualKeyCode::Q => self.keys_down.q = true,
                VirtualKeyCode::R => self.keys_down.r = true,
                VirtualKeyCode::S => self.keys_down.s = true,
                VirtualKeyCode::T => self.keys_down.t = true,
                VirtualKeyCode::U => self.keys_down.u = true,
                VirtualKeyCode::V => self.keys_down.v = true,
                VirtualKeyCode::W => self.keys_down.w = true,
                VirtualKeyCode::X => self.keys_down.x = true,
                VirtualKeyCode::Y => self.keys_down.y = true,
                VirtualKeyCode::Z => self.keys_down.z = true,
                _ => {}
            }
        });
        unprocessed_keyup_events.iter().for_each(|&keycode| {
            self.unprocessed_keyup_events.push(keycode);
            // yeah, this sucks
            // a possible solution: make keys_down a list of VirtualKeyCodes instead
            match keycode {
                VirtualKeyCode::A => self.keys_down.a = false,
                VirtualKeyCode::B => self.keys_down.b = false,
                VirtualKeyCode::C => self.keys_down.c = false,
                VirtualKeyCode::D => self.keys_down.d = false,
                VirtualKeyCode::E => self.keys_down.e = false,
                VirtualKeyCode::F => self.keys_down.f = false,
                VirtualKeyCode::G => self.keys_down.g = false,
                VirtualKeyCode::H => self.keys_down.h = false,
                VirtualKeyCode::I => self.keys_down.i = false,
                VirtualKeyCode::J => self.keys_down.j = false,
                VirtualKeyCode::K => self.keys_down.k = false,
                VirtualKeyCode::L => self.keys_down.l = false,
                VirtualKeyCode::M => self.keys_down.m = false,
                VirtualKeyCode::N => self.keys_down.n = false,
                VirtualKeyCode::O => self.keys_down.o = false,
                VirtualKeyCode::P => self.keys_down.p = false,
                VirtualKeyCode::Q => self.keys_down.q = false,
                VirtualKeyCode::R => self.keys_down.r = false,
                VirtualKeyCode::S => self.keys_down.s = false,
                VirtualKeyCode::T => self.keys_down.t = false,
                VirtualKeyCode::U => self.keys_down.u = false,
                VirtualKeyCode::V => self.keys_down.v = false,
                VirtualKeyCode::W => self.keys_down.w = false,
                VirtualKeyCode::X => self.keys_down.x = false,
                VirtualKeyCode::Y => self.keys_down.y = false,
                VirtualKeyCode::Z => self.keys_down.z = false,
                _ => {}
            }
        });
        self.done = done;

        // reset cursor and change camera view
        self.vk_window
            .get_surface()
            .window()
            .set_cursor_position(winit::dpi::LogicalPosition {
                x: CURSOR_RESET_POS_X as f64,
                y: CURSOR_RESET_POS_Y as f64,
            })
            .expect("Couldn't re-set cursor position!");

        self.unprocessed_events = unprocessed_events;
    }

    fn clear_unprocessed_events(&mut self) {
        self.unprocessed_events = vec![];
        self.unprocessed_keydown_events = vec![];
    }

    fn create_command_buffer(&mut self) {
        let clear_values = render_passes::clear_values_for_pass(self.render_pass.clone());

        let framebuffer = self.vk_window.next_framebuffer();

        let command_buffer = ll::command_buffer::create_command_buffer(
            self.device.clone(),
            self.queue.clone(),
            framebuffer,
            &clear_values,
            &self.world.get_objects(),
        );
        self.command_buffer = Some(command_buffer);
    }

    fn submit_and_check(&mut self) {
        self.vk_window
            .submit_command_buffer(self.queue.clone(), self.command_buffer.take().unwrap());
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
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

mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "
            #version 450

            layout(location = 0) in vec3 position;
            layout(location = 1) in vec3 color;
            layout(location = 2) in vec3 normal;
            layout(location = 0) out vec3 v_color;
            layout(location = 1) out vec3 v_normal;

            layout(set = 0, binding = 0) uniform Data {
                mat4 world;
                mat4 view;
                mat4 proj;
            } uniforms;

            void main() {
                mat4 worldview = uniforms.view * uniforms.world;
                gl_Position = uniforms.proj * worldview * vec4(position, 1.0);
                v_color = color;
                v_normal = normal;
            }"
    }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "
            #version 450

            layout(location = 0) in vec3 v_color;
            layout(location = 1) in vec3 v_normal;
            layout(location = 0) out vec4 f_color;

            const vec3 LIGHT = vec3(3.0, 2.0, 1.0);

            void main() {
                float brightness = dot(normalize(v_normal), normalize(LIGHT));
                vec3 dark_color = v_color * 0.6;
                vec3 regular_color = v_color;

                f_color = vec4(mix(dark_color, regular_color, brightness), 1.0);
            }
            "
    }
}
