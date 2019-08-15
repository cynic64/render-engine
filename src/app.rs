extern crate nalgebra_glm as glm;

use crate::camera::*;
use crate::exposed_tools::*;
use crate::input::*;
use crate::internal_tools::*;
use crate::render_passes;
use crate::world::*;

pub struct App {
    instance: Arc<Instance>,
    events_handler: EventHandler,
    physical_device_index: usize,
    device: Arc<Device>,
    queue: Arc<Queue>,
    render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    pub done: bool,
    command_buffer: Option<AutoCommandBuffer>,
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

        let events_handler = EventHandler::new(events_loop);

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
        // Initialization is finally finished!

        // In the loop below we are going to submit commands to the GPU. Submitting a command produces
        // an object that implements the `GpuFuture` trait, which holds the resources for as long as
        // they are in use by the GPU.

        let camera = OrbitCamera::default();

        let world = World::new(render_pass.clone(), device.clone(), Box::new(camera));

        Self {
            instance: instance.clone(),
            events_handler,
            physical_device_index: physical.index(),
            device,
            queue,
            render_pass,
            done: false,
            command_buffer: None,
            multisampling_enabled,
            world,
            vk_window,
        }
    }

    pub fn update_camera(&mut self, camera: Box<dyn Camera>) {
        self.world.update_camera(camera);
    }

    pub fn get_world_com(&self) -> WorldCommunicator {
        self.world.get_communicator()
    }

    pub fn enable_multisampling(&mut self) {
        self.multisampling_enabled = true;
        self.render_pass =
            render_passes::multisampled_with_depth(self.device.clone(), MULTISAMPLING_FACTOR);
        self.update_render_pass();
    }

    pub fn disable_multisampling(&mut self) {
        self.multisampling_enabled = false;
        self.render_pass = render_passes::with_depth(self.device.clone());
        self.update_render_pass();
    }

    pub fn set_render_pass(&mut self, render_pass: Arc<dyn RenderPassAbstract + Send + Sync>) {
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
        self.setup_frame();

        self.create_command_buffer();
        self.submit_and_check();

        self.update_world();
    }

    fn update_world(&mut self) {
        self.world.update(self.events_handler.frame_info.clone());
    }

    pub fn print_fps(&self) {
        let fps = self.events_handler.get_fps();
        println!("FPS: {}", fps);
    }

    pub fn get_device(&self) -> Arc<Device> {
        self.device.clone()
    }

    fn setup_frame(&mut self) {
        let dimensions = self.vk_window.get_dimensions();
        self.done = self.events_handler.update(dimensions);

        // reset cursor to center
        self.vk_window
            .get_surface()
            .window()
            .set_cursor_position(winit::dpi::LogicalPosition {
                x: (dimensions[0] as f64) / 2.0,
                y: (dimensions[1] as f64) / 2.0,
            })
            .expect("Couldn't re-set cursor position!");

        // it should always be none before drawing the frame anyway, but just make sure
        self.command_buffer = None;
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
