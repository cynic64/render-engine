use vulkano::device::Device;
use vulkano::format::{ClearValue, Format};
use vulkano::framebuffer::{LoadOp, RenderPassAbstract, RenderPassDesc};

use std::sync::Arc;

type RenderPass = Arc<dyn RenderPassAbstract + Send + Sync>;

// TODO: let user provide own format for color buffers
const DEFAULT_COLOR_FORMAT: Format = vulkano::format::Format::B8G8R8A8Unorm;
const DEFAULT_DEPTH_FORMAT: Format = vulkano::format::Format::D32Sfloat;

// TODO: resolve_depth is not needed. I think, at least - programs run without
// it, but make sure no jaggedness in introduced by removing it.

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

pub fn multisampled(device: Arc<Device>, factor: u32) -> RenderPass {
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
                }
            },
            pass: {
                color: [multisampled_color],
                depth_stencil: {},
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

pub fn read_depth(device: Arc<Device>) -> RenderPass {
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
                    load: Load,
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

pub fn only_depth(device: Arc<Device>) -> RenderPass {
    Arc::new(
        vulkano::single_pass_renderpass!(
            device.clone(),
            attachments: {
                depth: {
                    load: Clear,
                    store: Store,
                    format: DEFAULT_DEPTH_FORMAT,
                    samples: 1,
                }
            },
            pass: {
                color: [],
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
                }
            },
            pass: {
                color: [color],
                depth_stencil: {}
            }
        )
        .unwrap(),
    )
}

// TODO: add every format to this
pub fn clear_values_for_pass(
    render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
) -> Vec<ClearValue> {
    render_pass
        .attachment_descs()
        .map(|desc| match desc.load {
            LoadOp::Clear => match desc.format {
                Format::B8G8R8A8Unorm => [0.0, 0.0, 0.0, 1.0].into(),
                Format::R8G8B8A8Unorm => [0.0, 0.0, 0.0, 1.0].into(),
                Format::R32G32B32A32Sfloat => [0.0, 0.0, 0.0, 0.0].into(),
                Format::R16G16B16A16Sfloat => [0.0, 0.0, 0.0, 0.0].into(),
                Format::D16Unorm => 1f32.into(),
                Format::D32Sfloat => 1f32.into(),
                // TODO: make the panic print the bad format
                _ => panic!("You provided a format that the clear values couldn't be guessed for!"),
            },
            LoadOp::DontCare => ClearValue::None,
            LoadOp::Load => ClearValue::None,
        })
        .collect()
}
