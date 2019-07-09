pub use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
pub use vulkano::command_buffer::{AutoCommandBuffer, AutoCommandBufferBuilder, DynamicState};
pub use vulkano::descriptor::PipelineLayoutAbstract;
pub use vulkano::device::{Device, DeviceExtensions, Queue};
pub use vulkano::framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract, Subpass};
pub use vulkano::image::SwapchainImage;
pub use vulkano::instance::{Instance, PhysicalDevice};
pub use vulkano::pipeline::vertex::SingleBufferDefinition;
pub use vulkano::pipeline::viewport::Viewport;
pub use vulkano::pipeline::GraphicsPipeline;
pub use vulkano::swapchain;
pub use vulkano::swapchain::{
    AcquireError, PresentMode, Surface, SurfaceTransform, Swapchain, SwapchainCreationError,
};
pub use vulkano::sync;
pub use vulkano::sync::{FlushError, GpuFuture};

pub use vulkano_win::VkSurfaceBuild;

pub use winit::{Event, EventsLoop, Window, WindowBuilder, WindowEvent};

pub use std::sync::Arc;

use super::*;

pub struct SwapchainAndImages {
    pub swapchain: Arc<Swapchain<Window>>,
    pub images: Vec<Arc<SwapchainImage<Window>>>,
}

pub fn winit_event_to_keycode(event: &Event) -> Option<winit::KeyboardInput> {
    // only matches key press/release events
    if let Event::WindowEvent {
        event: WindowEvent::KeyboardInput { input, .. },
        ..
    } = event
    {
        if input.virtual_keycode.is_some() {
            Some(*input)
        } else {
            None
        }
    } else {
        None
    }
}

pub fn winit_event_to_mouse_movement(event: &Event) -> Option<(f32, f32)> {
    if let Event::WindowEvent {
        event: WindowEvent::CursorMoved { position: p, .. },
        ..
    } = event
    {
        let (x_diff, y_diff) = (
            p.x - (CURSOR_RESET_POS_X as f64),
            p.y - (CURSOR_RESET_POS_Y as f64),
        );
        let x_movement = x_diff as f32;
        let y_movement = y_diff as f32;

        Some((x_movement, y_movement))
    } else {
        None
    }
}
