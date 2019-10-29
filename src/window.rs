use vulkano::device::{Device, DeviceExtensions, Queue};
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::image::SwapchainImage;
use vulkano::instance::{Instance, PhysicalDevice};
use vulkano::swapchain::{SwapchainAcquireFuture, Surface};
use vulkano::sync::GpuFuture;

use vulkano_win::VkSurfaceBuild;

use winit::{EventsLoop, WindowBuilder};

use std::sync::Arc;

use re_ll::vk_window::VkWindow;

use crate::input::{EventHandler, FrameInfo};
use crate::render_passes;

pub struct Window {
    vk_window: VkWindow,
    event_handler: EventHandler,
    queue: Arc<Queue>,
    recenter: bool,
}

impl Window {
    pub fn new() -> (Self, Arc<Queue>) {
        // defaults to a basic render pass
        let instance = get_instance();
        let queue = get_queue(instance.clone());
        let device = queue.device().clone();

        let events_loop = EventsLoop::new();
        let surface = WindowBuilder::new()
            .build_vk_surface(&events_loop, instance.clone())
            .unwrap();

        let event_handler = EventHandler::new(events_loop);

        surface.window().hide_cursor(true);

        let physical = PhysicalDevice::enumerate(&instance).next().unwrap();
        let swapchain_caps = surface.capabilities(physical).unwrap();

        let render_pass = render_passes::basic(device.clone());

        let vk_window = VkWindow::new(
            queue.device().clone(),
            queue.clone(),
            surface.clone(),
            render_pass.clone(),
            swapchain_caps.clone(),
        );

        let window = Self {
            vk_window,
            event_handler,
            queue: queue.clone(),
            recenter: true,
        };

        (window, queue)
    }

    pub fn present_future<F: GpuFuture + 'static>(&mut self, future: F) {
        self.vk_window.present_image(self.queue.clone(), future);
    }

    pub fn next_image(&mut self) -> Arc<SwapchainImage<winit::Window>> {
        self.vk_window.next_image()
    }

    pub fn get_future(&mut self) -> SwapchainAcquireFuture<winit::Window> {
        self.vk_window.get_future()
    }

    pub fn update(&mut self) -> bool {
        // returns whether to exit the program or not
        // TODO: return an enum or move the done-checking to its own function
        let done = self.event_handler.update(self.get_dimensions());
        if self.recenter {
            self.recenter_cursor();
        }

        done
    }

    pub fn get_surface(&self) -> Arc<Surface<winit::Window>> {
        self.vk_window.get_surface()
    }

    pub fn set_recenter(&mut self, state: bool) {
        self.recenter = state;
    }

    fn recenter_cursor(&mut self) {
        let dimensions = self.get_dimensions();

        self.vk_window
            .get_surface()
            .window()
            .set_cursor_position(winit::dpi::LogicalPosition {
                x: (dimensions[0] as f64) / 2.0,
                y: (dimensions[1] as f64) / 2.0,
            })
            .expect("Couldn't re-set cursor position!");
    }

    pub fn get_dimensions(&self) -> [u32; 2] {
        self.vk_window.get_dimensions()
    }

    pub fn get_fps(&self) -> f32 {
        // TODO: move fps counting to Window instead of EventHandler
        self.event_handler.get_fps()
    }

    pub fn get_avg_delta(&self) -> f32 {
        self.event_handler.avg_delta()
    }

    pub fn get_frame_info(&self) -> FrameInfo {
        self.event_handler.frame_info.clone()
    }

    pub fn set_render_pass(&mut self, new_render_pass: Arc<dyn RenderPassAbstract + Send + Sync>) {
        self.vk_window.set_render_pass(new_render_pass);
        self.vk_window.rebuild();
    }
}

fn get_queue(instance: Arc<Instance>) -> Arc<Queue> {
    // gets some queue that will be used for everything else
    let physical = PhysicalDevice::enumerate(&instance).next().unwrap();

    let queue_family = physical
        .queue_families()
        .find(|&q| q.supports_graphics())
        .unwrap();

    let device_ext = DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::none()
    };
    let (_device, mut queues) = Device::new(
        physical,
        physical.supported_features(),
        &device_ext,
        [(queue_family, 0.5)].iter().cloned(),
    )
    .unwrap();

    queues.next().unwrap()
}

fn get_instance() -> Arc<Instance> {
    let extensions = vulkano_win::required_extensions();

    Instance::new(None, &extensions, None).unwrap()
}
