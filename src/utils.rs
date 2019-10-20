use vulkano::buffer::{BufferUsage, ImmutableBuffer};
use vulkano::device::Queue;
use vulkano::format::Format;
use vulkano::image::{Dimensions, ImageViewAccess, ImmutableImage};
use vulkano::memory::Content;
use vulkano::sync::GpuFuture;

use std::path::Path;
use std::sync::Arc;

pub fn bufferize_slice<T: Content + 'static + Send + Sync + Clone>(
    queue: Arc<Queue>,
    slice: &[T],
) -> Arc<ImmutableBuffer<[T]>> {
    ImmutableBuffer::from_iter(slice.iter().cloned(), BufferUsage::all(), queue)
        .unwrap()
        .0
}

pub fn bufferize_data<T: Content + 'static + Send + Sync>(
    queue: Arc<Queue>,
    data: T,
) -> Arc<ImmutableBuffer<T>> {
    ImmutableBuffer::from_data(data, BufferUsage::all(), queue)
        .unwrap()
        .0
}

pub fn load_texture(queue: Arc<Queue>, path: &Path) -> Arc<dyn ImageViewAccess + Send + Sync> {
    let (texture, tex_future) = {
        let image = image::open(path).unwrap().to_rgba();
        let (width, height) = image.dimensions();
        let image_data = image.into_raw().clone();

        ImmutableImage::from_iter(
            image_data.iter().cloned(),
            Dimensions::Dim2d {
                width,
                height,
            },
            Format::R8G8B8A8Unorm,
            queue.clone(),
        )
        .unwrap()
    };

    tex_future
        .then_signal_fence_and_flush()
        .unwrap()
        .wait(None)
        .unwrap();

    texture
}
