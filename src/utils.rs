use vulkano::buffer::{CpuAccessibleBuffer, BufferUsage};
use vulkano::device::Device;

use std::sync::Arc;

pub fn bufferize<T: vulkano::memory::Content + 'static + Send + Sync + Clone>(device: Arc<Device>, slice: &[T]) -> Arc<CpuAccessibleBuffer<[T]>>
{
    CpuAccessibleBuffer::from_iter(device, BufferUsage::all(), slice.iter().cloned()).unwrap()
}
