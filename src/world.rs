use crate::exposed_tools::*;
use crate::input::*;
use crate::internal_tools::*;
use crate::mesh_gen;
use crate::shaders::*;

use std::collections::HashMap;
use std::path::Path;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

use crate::system::RenderableObject;

use vulkano::buffer::BufferAccess;
pub use vulkano::pipeline::input_assembly::PrimitiveTopology;

extern crate nalgebra_glm as glm;

pub struct World {
    objects: HashMap<String, (ObjectSpec, RenderableObject)>,
    // we need to use an option to get around the borrow checker later
    // soooooorry
    command_recv: Option<Receiver<Command>>,
    // we store a copy of the sender as well so we can clone it and give it
    // out to whoever needs it
    command_send: Sender<Command>,
    render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    device: Arc<Device>,
    mvp: MVP,
    camera: Box<dyn Camera>,
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

// TODO: derive clone and change the builder for this so you can re-use
// halfway-complete builders
// it's useful, i swear!
pub struct ObjectSpec {
    mesh: Box<dyn Mesh>,
    material: Material,
    model_matrix: CameraMatrix,
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
    pub vs: Shader,
    pub fs: Shader,
}

impl World {
    pub fn new(
        render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
        device: Arc<Device>,
        camera: Box<dyn Camera>,
    ) -> Self {
        let (sender, receiver): (Sender<Command>, Receiver<Command>) = mpsc::channel();

        let model: CameraMatrix =
            glm::scale(&glm::Mat4::identity(), &glm::vec3(1.0, 1.0, 1.0)).into();
        let mvp = MVP {
            model,
            view: camera.get_view_matrix(),
            proj: camera.get_projection_matrix(),
        };

        Self {
            objects: HashMap::new(),
            command_recv: Some(receiver),
            command_send: sender,
            render_pass,
            device,
            mvp,
            camera,
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

        // TODO: put this in Shader?
        let vs_entry = spec.material.vs.entry.clone();
        let fs_entry = spec.material.fs.entry.clone();

        let vert_main = unsafe {
            spec.material.vs.module.graphics_entry_point(
                std::ffi::CStr::from_bytes_with_nul_unchecked(b"main\0"),
                vs_entry.vert_input,
                vs_entry.vert_output,
                vs_entry.vert_layout,
                vulkano::pipeline::shader::GraphicsShaderType::Vertex,
            )
        };

        let frag_main = unsafe {
            spec.material.fs.module.graphics_entry_point(
                std::ffi::CStr::from_bytes_with_nul_unchecked(b"main\0"),
                fs_entry.frag_input,
                fs_entry.frag_output,
                fs_entry.frag_layout,
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

        let object = RenderableObject {
            pipeline,
            vbuf,
            additional_resources: None,
        };

        self.objects.insert(id, (spec, object));
    }

    pub fn get_objects(&self) -> Vec<RenderableObject> {
        self.objects.values().map(|(_spec, obj)| obj.clone()).collect()
    }

    pub fn delete_object(&mut self, id: String) {
        self.objects.remove(&id);
    }

    pub fn update(&mut self, frame_info: FrameInfo) {
        self.check_for_commands();
        self.camera.handle_input(frame_info.clone());
        self.update_resources();
        self.mvp.view = self.camera.get_view_matrix();
        self.mvp.proj = self.camera.get_projection_matrix();
    }

    pub fn update_resources(&mut self) {
        let device = self.device.clone();
        let view = self.mvp.view;
        let proj = self.mvp.proj;

        self.objects.values_mut().for_each(|(spec, obj)| {
            let uniform_buffer = vulkano::buffer::cpu_pool::CpuBufferPool::<MVP>::new(
                device.clone(),
                vulkano::buffer::BufferUsage::all(),
            );

            // TODO: separate model matrix from the rest bc it is the only one
            // that changes between objects
            let uniform_buffer_subbuffer = {
                let uniform_data = MVP {
                    model: spec.model_matrix,
                    view: view,
                    proj: proj,
                };
                uniform_buffer.next(uniform_data).unwrap()
            };

            obj.additional_resources = Some(Arc::new(uniform_buffer_subbuffer.clone()))
        });
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

fn vbuf_from_vec<V>(device: Arc<Device>, slice: &[V]) -> Arc<dyn BufferAccess + Send + Sync>
where
    V: vulkano::memory::Content + Send + Sync + Clone + 'static,
{
    CpuAccessibleBuffer::from_iter(device, BufferUsage::all(), slice.iter().cloned()).unwrap()
}

pub struct ObjectSpecBuilder {
    custom_mesh: Option<Box<dyn Mesh>>,
    custom_fill_type: Option<PrimitiveTopology>,
    custom_shaders: Option<(Shader, Shader)>,
    custom_model_matrix: Option<CameraMatrix>
}

impl ObjectSpecBuilder {
    pub fn default() -> Self {
        Self {
            custom_mesh: None,
            custom_fill_type: None,
            custom_shaders: None,
            custom_model_matrix: None,
        }
    }

    pub fn mesh<M: Mesh + 'static>(self, mesh: M) -> Self {
        Self {
            custom_mesh: Some(Box::new(mesh)),
            ..self
        }
    }

    pub fn shaders(self, vs: Shader, fs: Shader) -> Self {
        Self {
            custom_shaders: Some((vs, fs)),
            ..self
        }
    }

    pub fn model_matrix(self, model_matrix: CameraMatrix) -> Self {
        Self {
            custom_model_matrix: Some(model_matrix),
            ..self
        }
    }

    pub fn build(self, device: Arc<Device>) -> ObjectSpec {
        let fill_type = self
            .custom_fill_type
            .unwrap_or(PrimitiveTopology::TriangleList);

        // if you choose to customize shaders, you need to provide both
        let (vs, fs) = self.custom_shaders.unwrap_or_else(|| {
            let vert_path = Path::new(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/shaders/deferred/default_geo_vert.glsl"
            ));

            let frag_path = Path::new(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/shaders/deferred/default_geo_frag.glsl"
            ));

            Shader::load_from_file(device.clone(), &vert_path, &frag_path)
        });

        let material = Material { fill_type, vs, fs };

        // if no mesh is provided, load a cube
        let mesh = self.custom_mesh.unwrap_or_else(|| {
            Box::new(mesh_gen::create_vertices_for_cube(
                [0.0, 0.0, 0.0],
                1.0,
                [1.0, 1.0, 1.0],
            ))
        });

        // if no model matrix is provided, use the identity matrix
        let model_matrix = self.custom_model_matrix.unwrap_or(glm::Mat4::identity().into());

        ObjectSpec { mesh, material, model_matrix }
    }
}
