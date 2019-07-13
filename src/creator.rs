use crate::internal_tools::*;
use crate::exposed_tools::*;

#[derive(Clone)]
pub struct VbufCreator {
    device: Arc<Device>,
}

impl VbufCreator {
    pub fn new(device: Arc<Device>) -> Self {
        VbufCreator { device }
    }

    pub fn duplicate(&self) -> Self {
        VbufCreator { device: self.device.clone() }
    }

    pub fn create_vbuf_from_verts(&self, verts: &[Vertex]) -> Arc<VertexBuffer> {
        CpuAccessibleBuffer::from_iter(
            self.device.clone(),
            BufferUsage::all(),
            verts.iter().cloned(),
        )
        .unwrap()
    }
}
