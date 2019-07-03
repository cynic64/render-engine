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

pub fn winit_event_to_keycode(event: Event) -> Option<VirtualKeyCode> {
    // only matches keydown events
    if let Event::WindowEvent {
        event:
            WindowEvent::KeyboardInput {
                input:
                    winit::KeyboardInput {
                        virtual_keycode: Some(key),
                        state: winit::ElementState::Pressed,
                        ..
                    },
                ..
            },
        ..
    } = event
    {
        Some(key)
    } else {
        None
    }
}
