use crate::exposed_tools::*;
use crate::input::*;
use crate::internal_tools::*;
use crate::shaders::*;

use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::path::Path;

use vulkano::buffer::BufferAccess;
pub use vulkano::pipeline::input_assembly::PrimitiveTopology;
use vulkano::pipeline::GraphicsPipelineAbstract;

use ll::command_buffer::ConcreteObject;

extern crate nalgebra_glm as glm;

pub struct World {
    objects: HashMap<String, ConcreteObject>,
    // we need to use an option to get around the borrow checker later
    // soooooorry
    command_recv: Option<Receiver<Command>>,
    // we store a copy of the sender as well so we can clone it and give it
    // out to whoever needs it
    command_send: Sender<Command>,
    render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    device: Arc<Device>,
    default_dynstate: DynamicState,
    mvp: MVP,
    camera: Box<dyn Camera>,
    default_vs: Shader,
    default_fs: Shader,
}

#[derive(Clone)]
pub struct WorldCommunicator {
    command_send: Sender<Command>,
}

pub enum Command {
    AddObjectFromSpec { id: String, spec: ObjectSpec },
    DeleteObject { id: String },
}

// the ObjectSpec is an abstract definition of the object, the DrawableObject
// contains all the concrete things needed to actually draw the object like
// the pipeline and vertex shaders
pub struct ObjectSpec {
    mesh: Box<dyn Mesh>,
    material: Material,
}

pub trait Mesh {
    fn create_vbuf(&self, device: Arc<Device>) -> Arc<dyn BufferAccess + Send + Sync>;
}

impl<V> Mesh for Vec<V>
where
    V: vulkano::memory::Content + Send + Sync + Clone + 'static,
{
    fn create_vbuf(&self, device: Arc<Device>) -> Arc<dyn BufferAccess + Send + Sync> {
        vbuf_from_vec(device, &self)
    }
}

// will eventually contain a shader and all other info the pipeline needs
// maybe a MaterialSpec would be useful too, cause it wouldn't require a vulkan instance... idk
struct Material {
    pub fill_type: PrimitiveTopology,
}

impl World {
    pub fn new(
        render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
        device: Arc<Device>,
        camera: Box<dyn Camera>,
    ) -> Self {
        let (sender, receiver): (Sender<Command>, Receiver<Command>) = mpsc::channel();

        // set the dynamic state to a dummy value
        let viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions: [0.0, 0.0],
            depth_range: 0.0..1.0,
        };
        let dynamic_state = DynamicState {
            line_width: None,
            viewports: Some(vec![viewport]),
            scissors: None,
        };

        let model: CameraMatrix =
            glm::scale(&glm::Mat4::identity(), &glm::vec3(1.0, 1.0, 1.0)).into();
        let mvp = MVP {
            model,
            view: camera.get_view_matrix(),
            proj: camera.get_projection_matrix(),
        };

        let vert_path = Path::new(
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/examples/vulkan/shaders/vert.glsl"
            )
        );

        let frag_path = Path::new(
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/examples/vulkan/shaders/frag.glsl"
            )
        );

        let (vs, fs) = Shader::load_from_file(device.clone(), &vert_path, &frag_path);

        Self {
            objects: HashMap::new(),
            command_recv: Some(receiver),
            command_send: sender,
            render_pass,
            device,
            default_dynstate: dynamic_state,
            mvp,
            camera,
            default_vs: vs,
            default_fs: fs,
        }
    }

    pub fn update_render_pass(
        &mut self,
        new_renderpass: Arc<dyn RenderPassAbstract + Send + Sync>,
    ) {
        self.render_pass = new_renderpass;
    }

    pub fn update_camera(&mut self, camera: Box<dyn Camera>) {
        self.camera = camera;
    }

    pub fn get_communicator(&self) -> WorldCommunicator {
        WorldCommunicator::from_sender(self.command_send.clone())
    }

    pub fn add_object_from_spec(&mut self, id: String, spec: ObjectSpec) {
        let vbuf = spec.mesh.create_vbuf(self.device.clone());

        let vs_entry = self.default_vs.entry.clone();
        let fs_entry = self.default_fs.entry.clone();

        let vert_main = unsafe {
            self.default_vs.module.graphics_entry_point(
                std::ffi::CStr::from_bytes_with_nul_unchecked(b"main\0"),
                vs_entry.vert_input,
                vs_entry.vert_output,
                vs_entry.vert_layout,
                vulkano::pipeline::shader::GraphicsShaderType::Vertex,
            )
        };

        let frag_main = unsafe {
            self.default_fs.module.graphics_entry_point(
                std::ffi::CStr::from_bytes_with_nul_unchecked(b"main\0"),
                fs_entry.vert_input,
                fs_entry.vert_output,
                fs_entry.vert_layout,
                vulkano::pipeline::shader::GraphicsShaderType::Fragment,
            )
        };

        let pipeline = Arc::new(
            GraphicsPipeline::start()
                .vertex_input_single_buffer::<Vertex>()
                .vertex_shader(vert_main, ())
                .primitive_topology(spec.material.fill_type)
                .viewports_dynamic_scissors_irrelevant(1)
                .fragment_shader(frag_main, ())
                .render_pass(Subpass::from(self.render_pass.clone(), 0).unwrap())
                .depth_stencil_simple_depth()
                .build(self.device.clone())
                .unwrap(),
        );

        let uniform_set = uniform_for_mvp(self.device.clone(), &self.mvp, pipeline.clone());

        let object = ConcreteObject {
            pipeline,
            dynamic_state: self.default_dynstate.clone(),
            vertex_buffer: vbuf,
            uniform_set,
        };

        self.objects.insert(id, object);
    }

    pub fn get_objects(&self) -> Vec<ConcreteObject> {
        self.objects.values().map(|x| x.clone()).collect()
    }

    pub fn delete_object(&mut self, id: String) {
        self.objects.remove(&id);
    }

    pub fn update(&mut self, frame_info: FrameInfo) {
        self.check_for_commands();
        self.camera.handle_input(frame_info.clone());
        self.mvp.view = self.camera.get_view_matrix();
        self.mvp.proj = self.camera.get_projection_matrix();
        self.update_uniform_buffers();
        self.update_dynstate(frame_info.dimensions);
    }

    fn update_uniform_buffers(&mut self) {
        let mvp = self.mvp.clone();
        let device = self.device.clone();
        self.objects.values_mut().for_each(|x| {
            x.uniform_set = uniform_for_mvp(device.clone(), &mvp, x.pipeline.clone());
        });
    }

    fn update_dynstate(&mut self, dimensions: [u32; 2]) {
        let viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions: [dimensions[0] as f32, dimensions[1] as f32],
            depth_range: 0.0..1.0,
        };
        let dynamic_state = DynamicState {
            line_width: None,
            viewports: Some(vec![viewport]),
            scissors: None,
        };
        self.default_dynstate = dynamic_state.clone();
        self.objects
            .values_mut()
            .for_each(|x| x.dynamic_state = dynamic_state.clone());
    }

    fn check_for_commands(&mut self) {
        let command_recv = self.command_recv.take().unwrap();

        command_recv.try_iter().for_each(|command| match command {
            Command::AddObjectFromSpec { id, spec } => self.add_object_from_spec(id, spec),
            Command::DeleteObject { id } => self.delete_object(id),
        });

        self.command_recv = Some(command_recv);
    }
}

impl WorldCommunicator {
    pub fn from_sender(sender: Sender<Command>) -> Self {
        Self {
            command_send: sender,
        }
    }

    pub fn add_object_from_spec(&mut self, id: String, spec: ObjectSpec) {
        let command = Command::AddObjectFromSpec { id, spec };

        self.command_send.send(command).unwrap();
    }

    pub fn delete_object(&mut self, id: String) {
        let command = Command::DeleteObject { id };

        self.command_send.send(command).unwrap();
    }
}

impl ObjectSpec {
    pub fn from_mesh<M: Mesh + 'static>(mesh: M) -> Self {
        Self {
            mesh: Box::new(mesh),
            material: Material::default(),
        }
    }

    pub fn switch_fill_type(&mut self, new_primitive_topology: PrimitiveTopology) {
        self.material.fill_type = new_primitive_topology;
    }
}

impl Material {
    pub fn default() -> Self {
        Self {
            fill_type: PrimitiveTopology::TriangleList,
        }
    }
}

fn uniform_for_mvp(
    device: Arc<Device>,
    mvp: &MVP,
    pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
) -> Arc<dyn DescriptorSet + Send + Sync> {
    let uniform_buffer = vulkano::buffer::cpu_pool::CpuBufferPool::<MVP>::new(
        device.clone(),
        vulkano::buffer::BufferUsage::all(),
    );

    let uniform_buffer_subbuffer = {
        let uniform_data = MVP {
            model: mvp.model,
            view: mvp.view,
            proj: mvp.proj,
        };

        uniform_buffer.next(uniform_data).unwrap()
    };

    Arc::new(
        vulkano::descriptor::descriptor_set::PersistentDescriptorSet::start(pipeline.clone(), 0)
            .add_buffer(uniform_buffer_subbuffer)
            .unwrap()
            .build()
            .unwrap(),
    )
}

fn vbuf_from_vec<V>(device: Arc<Device>, slice: &[V]) -> Arc<dyn BufferAccess + Send + Sync>
where
    V: vulkano::memory::Content + Send + Sync + Clone + 'static,
{
    CpuAccessibleBuffer::from_iter(device, BufferUsage::all(), slice.iter().cloned()).unwrap()
}
