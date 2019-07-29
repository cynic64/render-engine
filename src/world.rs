use crate::exposed_tools::*;
use crate::internal_tools::*;

use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

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
    renderpass: Arc<RenderPassAbstract + Send + Sync>,
    device: Arc<Device>,
    default_dynstate: DynamicState,
    mvp: MVP,
    camera: Box<Camera>,
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
    mesh: Vec<Vertex>,
    material: Material,
}

struct Material {
    pub fill_type: PrimitiveTopology,
}

impl World {
    pub fn new(
        renderpass: Arc<RenderPassAbstract + Send + Sync>,
        device: Arc<Device>,
        camera: Box<Camera>,
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

        Self {
            objects: HashMap::new(),
            command_recv: Some(receiver),
            command_send: sender,
            renderpass,
            device,
            default_dynstate: dynamic_state,
            mvp,
            camera,
        }
    }

    pub fn update_renderpass(&mut self, new_renderpass: Arc<RenderPassAbstract + Send + Sync>) {
        self.renderpass = new_renderpass;
    }

    pub fn update_camera(&mut self, camera: Box<Camera>) {
        self.camera = camera;
    }

    pub fn get_communicator(&self) -> WorldCommunicator {
        WorldCommunicator::from_sender(self.command_send.clone())
    }

    pub fn add_object_from_spec(&mut self, id: String, spec: ObjectSpec) {
        let vbuf = vbuf_from_verts(self.device.clone(), &spec.mesh);

        let vs = vs::Shader::load(self.device.clone()).unwrap();
        let fs = fs::Shader::load(self.device.clone()).unwrap();
        let pipeline = Arc::new(
            GraphicsPipeline::start()
                .vertex_input_single_buffer::<Vertex>()
                .vertex_shader(vs.main_entry_point(), ())
                .primitive_topology(spec.material.fill_type)
                .viewports_dynamic_scissors_irrelevant(1)
                .fragment_shader(fs.main_entry_point(), ())
                .render_pass(Subpass::from(self.renderpass.clone(), 0).unwrap())
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

    pub fn update(
        &mut self,
        events: &[Event],
        keys_down: &KeysDown,
        delta: f32,
        dimensions: [u32; 2],
    ) {
        self.check_for_commands();
        self.camera.handle_input(events, keys_down, delta);
        self.mvp.view = self.camera.get_view_matrix();
        self.mvp.proj = self.camera.get_projection_matrix();
        self.update_uniform_buffers();
        self.update_dynstate(dimensions);
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
    pub fn from_mesh(mesh: Vec<Vertex>) -> Self {
        Self {
            mesh,
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
    pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>,
) -> Arc<DescriptorSet + Send + Sync> {
    let uniform_buffer = vulkano::buffer::cpu_pool::CpuBufferPool::<vs::ty::Data>::new(
        device.clone(),
        vulkano::buffer::BufferUsage::all(),
    );

    let uniform_buffer_subbuffer = {
        let uniform_data = vs::ty::Data {
            world: mvp.model,
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

fn vbuf_from_verts(device: Arc<Device>, verts: &[Vertex]) -> Arc<VertexBuffer> {
    CpuAccessibleBuffer::from_iter(device, BufferUsage::all(), verts.iter().cloned()).unwrap()
}

pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "
            #version 450

            layout(location = 0) in vec3 position;
            layout(location = 1) in vec3 color;
            layout(location = 2) in vec3 normal;
            layout(location = 0) out vec3 v_color;
            layout(location = 1) out vec3 v_normal;

            layout(set = 0, binding = 0) uniform Data {
                mat4 world;
                mat4 view;
                mat4 proj;
            } uniforms;

            void main() {
                mat4 worldview = uniforms.view * uniforms.world;
                gl_Position = uniforms.proj * worldview * vec4(position, 1.0);
                v_color = color;
                v_normal = normal;
            }"
    }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "
            #version 450

            layout(location = 0) in vec3 v_color;
            layout(location = 1) in vec3 v_normal;
            layout(location = 0) out vec4 f_color;

            const vec3 LIGHT = vec3(3.0, 2.0, 1.0);

            void main() {
                float brightness = dot(normalize(v_normal), normalize(LIGHT));
                vec3 dark_color = v_color * 0.6;
                vec3 regular_color = v_color;

                f_color = vec4(mix(dark_color, regular_color, brightness), 1.0);
            }
            "
    }
}
