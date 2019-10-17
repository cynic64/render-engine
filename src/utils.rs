use vulkano::buffer::{CpuAccessibleBuffer, BufferUsage, BufferAccess};
use vulkano::device::Device;
use vulkano::memory::Content;

use std::sync::Arc;

pub fn bufferize_slice<T: Content + 'static + Send + Sync + Clone>(device: Arc<Device>, slice: &[T]) -> Arc<CpuAccessibleBuffer<[T]>>
{
    CpuAccessibleBuffer::from_iter(device, BufferUsage::all(), slice.iter().cloned()).unwrap()
}

pub fn bufferize_data<T: Content + 'static + Send + Sync>(device: Arc<Device>, data: T) -> Arc<dyn BufferAccess + Send + Sync> {
    let pool = vulkano::buffer::cpu_pool::CpuBufferPool::new(
        device.clone(),
        vulkano::buffer::BufferUsage::all(),
    );
    Arc::new(pool.next(data).unwrap())
}
