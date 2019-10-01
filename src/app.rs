use vulkano::device::{Device, DeviceExtensions, Queue};
use vulkano::instance::{Instance, PhysicalDevice};
use vulkano::swapchain::Surface;

use vulkano_win::VkSurfaceBuild;

use winit::{EventsLoop, Window, WindowBuilder};

use std::sync::Arc;
use std::collections::HashMap;

use re_ll as ll;

use crate::input::*;
use crate::producer::ProducerCollection;
use crate::system::System;
use crate::template_systems;
use crate::world::*;

pub struct App<'a> {
    events_handler: EventHandler,
    device: Arc<Device>,
    queue: Arc<Queue>,
    pub done: bool,
    world: World,
    vk_window: ll::vk_window::VkWindow,
    system: System<'a>,
    producers: ProducerCollection,
}

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

        // At this point, OpenGL initialization would be finished. However in Vulkan it is not. OpenGL
        // implicitly does a lot of computation whenever you draw. In Vulkan, you have to do all this
        // manually.

        let swapchain_caps = surface.capabilities(physical).unwrap();
        // on my machine this is B8G8R8Unorm

        // create the system
        let (system, producers) = template_systems::forward_with_depth(queue.clone());
        // TODO: which render pass does this refer to?
        let render_pass = system.get_passes()[0].get_render_pass().clone();

        let world = World::new(render_pass.clone(), device.clone());

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
            done: false,
            world,
            vk_window,
            system,
            producers,
        }
    }

    pub fn get_world_com(&self) -> WorldCommunicator {
        self.world.get_communicator()
    }

    // TODO: create a separate module for managing systems. hopefully it would
    // let you do stuff like compose whether multisampling was enabled and AO
    // and all that

    // another idea: "optimizations" for different passes, kinda like vulkano's
    // command buffers and images. For example, if there are never any
    // additional resources, the check can be skipped.
    pub fn set_system(&mut self, system: System<'a>) {
        // for now it assumes a single pass is used in the system
        // TODO: make it so it can figure out which pass belongs to the world
        // and which belongs to the window
        let render_pass = system.get_passes()[0].get_render_pass();
        self.vk_window.set_render_pass(render_pass.clone());
        self.vk_window.rebuild();
        self.world.set_render_pass(render_pass.clone());
        self.system = system;
    }

    pub fn set_producers(&mut self, new_producers: ProducerCollection) {
        self.producers = new_producers;
    }

    pub fn draw_frame(&mut self) {
        self.setup_frame();

        self.draw_objects();

        self.handle_input();
        self.world.update();
    }

    fn handle_input(&mut self) {
        self.producers
            .update(self.events_handler.frame_info.clone());
    }

    pub fn get_frame_info(&mut self) -> FrameInfo {
        self.events_handler.frame_info.clone()
    }

    pub fn print_fps(&self) {
        let fps = self.events_handler.get_fps();
        println!("FPS: {}", fps);
        self.system.print_stats();
    }

    pub fn get_device(&self) -> Arc<Device> {
        self.device.clone()
    }

    pub fn get_queue(&self) -> Arc<Queue> {
        self.queue.clone()
    }

    fn setup_frame(&mut self) {
        let dimensions = self.vk_window.get_dimensions();
        // TODO: move this somewhere else
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
    }

    fn draw_objects(&mut self) {
        let world_renderable_objects = self.world.get_objects();
        let mut all_renderable_objects = HashMap::new();
        all_renderable_objects.insert("geometry", world_renderable_objects);
        let swapchain_image = self.vk_window.next_image();
        let swapchain_fut = self.vk_window.get_future();
        let shared_resources = self.producers.get_shared_resources(self.device.clone());

        // draw_frame returns a future representing the completion of rendering
        let frame_fut = self.system.draw_frame(
            swapchain_image.dimensions(),
            all_renderable_objects,
            shared_resources,
            swapchain_image,
            swapchain_fut,
        );

        self.vk_window.present_image(self.queue.clone(), frame_fut);
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
