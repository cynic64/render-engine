use crate::creator::VbufCreator;
use crate::exposed_tools::*;
use crate::internal_tools::*;

use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

pub use vulkano::pipeline::input_assembly::PrimitiveTopology;

use ll::command_buffer_creator::ConcreteObject;

pub struct World {
    objects: HashMap<String, ConcreteObject>,
    vbuf_creator: VbufCreator,
    // we need to use an option to get around the borrow checker later
    // soooooorry
    command_recv: Option<Receiver<Command>>,
    // we store a copy of the sender as well so we can clone it and give it
    // out to whoever needs it
    command_send: Sender<Command>,
    renderpass: Arc<RenderPassAbstract + Send + Sync>,
    device: Arc<Device>,
    default_uniform_set: Arc<DescriptorSet + Send + Sync>,
    default_dynamic_state: DynamicState,
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
    pub fn new(vbuf_creator: VbufCreator, renderpass: Arc<RenderPassAbstract + Send + Sync>, device: Arc<Device>, default_uniform_set: Arc<DescriptorSet + Send + Sync>, default_dynamic_state: DynamicState) -> Self {
        let (sender, receiver): (Sender<Command>, Receiver<Command>) = mpsc::channel();

        Self {
            objects: HashMap::new(),
            vbuf_creator,
            command_recv: Some(receiver),
            command_send: sender,
            renderpass,
            device,
            default_uniform_set,
            default_dynamic_state,
        }
    }

    pub fn update_renderpass(&mut self, new_renderpass: Arc<RenderPassAbstract + Send + Sync>) {
        self.renderpass = new_renderpass;
    }

    pub fn update_device(&mut self, device: Arc<Device>) {
        self.device = device;
    }

    pub fn update_default_uniform_set(&mut self, uniform_set: Arc<DescriptorSet + Send + Sync>) {
        self.default_uniform_set = uniform_set.clone();

        self.objects.values_mut().for_each(|obj| obj.uniform_set = uniform_set.clone());
    }

    pub fn update_default_dynamic_state(&mut self, dynamic_state: DynamicState) {
        self.default_dynamic_state = dynamic_state.clone();

        self.objects.values_mut().for_each(|obj| obj.dynamic_state = dynamic_state.clone());
    }

    pub fn get_communicator(&self) -> WorldCommunicator {
        WorldCommunicator::from_sender(self.command_send.clone())
    }

    pub fn add_object_from_spec(&mut self, id: String, spec: ObjectSpec) {
        let vbuf = self.vbuf_creator.create_vbuf_from_verts(&spec.mesh);

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

        let object = ConcreteObject {
            pipeline,
            dynamic_state: self.default_dynamic_state.clone(),
            vertex_buffer: vbuf,
            uniform_set: self.default_uniform_set.clone(),
        };

        self.objects.insert(id, object);
    }

    pub fn get_objects(&self) -> Vec<ConcreteObject> {
        self.objects.values().map(|x| x.clone()).collect()
    }

    pub fn delete_object(&mut self, id: String) {
        self.objects.remove(&id);
    }

    pub fn check_for_commands(&mut self) {
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
