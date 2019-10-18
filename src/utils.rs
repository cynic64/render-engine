use vulkano::buffer::{ImmutableBuffer, BufferUsage};
use vulkano::device::Queue;
use vulkano::memory::Content;

use std::sync::Arc;

pub fn bufferize_slice<T: Content + 'static + Send + Sync + Clone>(queue: Arc<Queue>, slice: &[T]) -> Arc<ImmutableBuffer<[T]>>
{
    ImmutableBuffer::from_iter(slice.iter().cloned(), BufferUsage::all(), queue).unwrap().0
}

pub fn bufferize_data<T: Content + 'static + Send + Sync>(queue: Arc<Queue>, data: T) -> Arc<ImmutableBuffer<T>>
{
    ImmutableBuffer::from_data(data, BufferUsage::all(), queue).unwrap().0
}
