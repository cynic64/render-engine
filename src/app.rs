extern crate nalgebra_glm as glm;

use crate::camera::*;
use crate::exposed_tools::*;
use crate::input::*;
use crate::internal_tools::*;
use crate::render_passes;
use crate::world::*;
use crate::system;
use crate::template_systems;

use std::collections::HashMap;

pub struct App<'a> {
    events_handler: EventHandler,
    device: Arc<Device>,
    queue: Arc<Queue>,
    render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    pub done: bool,
    command_buffer: Option<AutoCommandBuffer>,
    multisampling_enabled: bool,
    world: World,
    vk_window: ll::vk_window::VkWindow,
    system: system::System<'a>,
}

#[derive(Default, Copy, Clone)]
struct SimpleVertex {
    position: [f32; 2],
}
vulkano::impl_vertex!(SimpleVertex, position);

const MULTISAMPLING_FACTOR: u32 = 4;

impl<'a> App<'a> {
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

        // create the system
        let system = template_systems::forward_msaa_depth(queue.clone());
        let render_pass = system.get_passes()[0].get_render_pass().clone();

        let camera = OrbitCamera::default();

        let world = World::new(render_pass.clone(), device.clone(), Box::new(camera));

        let vk_window = ll::vk_window::VkWindow::new(
            device.clone(),
            queue.clone(),
            surface.clone(),
            render_pass.clone(),
            swapchain_caps.clone(),
        );

        Self {
            events_handler,
            device,
            queue,
            render_pass,
            done: false,
            command_buffer: None,
            multisampling_enabled,
            world,
            vk_window,
            system,
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
        // commented out because of testing the deferred pipeline
        // self.vk_window.update_render_pass(self.render_pass.clone());
        // self.vk_window.rebuild();
        println!("You shouldn't be calling update_render_pass, it's broken atm!");
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
        let world_renderable_objects = self.world.get_objects();
        let all_renderable_objects = vec![world_renderable_objects];
        let swapchain_image = self.vk_window.next_image();
        let swapchain_fut = self.vk_window.get_future();

        let shared_resources: HashMap<&str, Arc<dyn BufferAccess + Send + Sync>> = HashMap::new();

        let frame_fut = self.system.draw_frame(
            self.vk_window.get_dimensions(),
            all_renderable_objects,
            shared_resources,
            swapchain_image,
            swapchain_fut,
        );

        self.vk_window.present_image(self.queue.clone(), frame_fut);
    }

    fn submit_and_check(&mut self) {
    //     self.vk_window
    //         .submit_command_buffer(self.queue.clone(), self.command_buffer.take().unwrap());
    }
}

impl<'a> Default for App<'a> {
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
    PhysicalDevice::enumerate(&instance).next().unwrap()
}

fn get_device_and_queues(
    physical: PhysicalDevice,
    surface: Arc<Surface<Window>>,
) -> (Arc<Device>, vulkano::device::QueuesIter) {
    let queue_family = physical
        .queue_families()
        .find(|&q| {
            // We take the first queue that supports drawing to our window.
            q.supports_graphics() && surface.is_supported(q).unwrap_or(false)
        })
        .unwrap();
    let device_ext = DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::none()
    };
    Device::new(
        physical,
        physical.supported_features(),
        &device_ext,
        [(queue_family, 0.5)].iter().cloned(),
    )
    .unwrap()
}
