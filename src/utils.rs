use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::device::Device;

use std::sync::Arc;

pub fn ibuf_from_vec(device: Arc<Device>, slice: &[u32]) -> Arc<CpuAccessibleBuffer<[u32]>>
{
    CpuAccessibleBuffer::from_iter(device, BufferUsage::all(), slice.iter().cloned()).unwrap()
}
