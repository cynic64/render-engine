extern crate vulkano;

use vulkano::framebuffer::RenderPassAbstract;
use vulkano::device::Device;
use vulkano::format::Format;

use std::sync::Arc;

type RenderPass = Arc<RenderPassAbstract + Send + Sync>;

// TODO: let user provide own format for color buffers
const DEFAULT_COLOR_FORMAT: Format = vulkano::format::Format::B8G8R8A8Unorm;
const DEFAULT_DEPTH_FORMAT: Format = vulkano::format::Format::D16Unorm;

pub fn multisampled_with_depth(device: Arc<Device>, factor: u32) -> RenderPass {
    Arc::new(
        vulkano::single_pass_renderpass!(
            device.clone(),
            attachments: {
                resolve_color: {
                    load: Clear,
                    store: Store,
                    format: DEFAULT_COLOR_FORMAT,
                    samples: 1,
                },
                multisampled_color: {
                    load: Clear,
                    store: DontCare,
                    format: DEFAULT_COLOR_FORMAT,
                    samples: factor,
                },
                multisampled_depth: {
                    load: Clear,
                    store: DontCare,
                    format: DEFAULT_DEPTH_FORMAT,
                    samples: factor,
                },
                resolve_depth: {
                    load: DontCare,
                    store: DontCare,
                    format: DEFAULT_DEPTH_FORMAT,
                    samples: 1,
                    initial_layout: ImageLayout::Undefined,
                    final_layout: ImageLayout::DepthStencilAttachmentOptimal,
                }
            },
            pass: {
                color: [multisampled_color],
                depth_stencil: {multisampled_depth},
                resolve: [resolve_color]
            }
        )
        .unwrap(),
    )
}

pub fn with_depth(device: Arc<Device>) -> RenderPass {
    Arc::new(
        vulkano::single_pass_renderpass!(
            device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: DEFAULT_COLOR_FORMAT,
                    samples: 1,
                },
                depth: {
                    load: Clear,
                    store: Store,
                    format: DEFAULT_DEPTH_FORMAT,
                    samples: 1,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {depth}
            }
        )
        .unwrap(),
    )
}

pub fn basic(device: Arc<Device>) -> RenderPass {
    Arc::new(
        vulkano::single_pass_renderpass!(
            device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: DEFAULT_COLOR_FORMAT,
                    samples: 1,
                },
                depth: {
                    load: Clear,
                    store: Store,
                    format: DEFAULT_DEPTH_FORMAT,
                    samples: 1,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {depth}
            }
        )
            .unwrap(),
    )
}
