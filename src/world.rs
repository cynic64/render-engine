use crate::creator::VbufCreator;
use crate::exposed_tools::*;
use crate::internal_tools::*;

use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

pub use vulkano::pipeline::input_assembly::PrimitiveTopology;

pub struct World {
    objects: HashMap<String, Object>,
    vbuf_creator: VbufCreator,
    // we need to use an option to get around the borrow checker later
    // soooooorry
    command_recv: Option<Receiver<Command>>,
    // we store a copy of the sender as well so we can clone it and give it
    // out to whoever needss it
    command_send: Sender<Command>,
    renderpass: Option<Arc<RenderPassAbstract + Send + Sync>>,
    device: Option<Arc<Device>>,
}

#[derive(Clone)]
pub struct WorldCommunicator {
    command_send: Sender<Command>,
}

pub enum Command {
    ObjectFromVbuf { id: String, vbuf: Arc<VertexBuffer> },
    ObjectFromVerts { id: String, verts: Vec<Vertex> },
    DeleteObject { id: String },
    UpdateMaterials,
}

pub struct Object {
    vbuf: Arc<VertexBuffer>,
    material: Material,
}

pub struct Material {
    cached_pipeline: Option<Arc<ConcreteGraphicsPipeline>>,
    fill_type: PrimitiveTopology,
}

impl World {
    pub fn from_creator(vbuf_creator: VbufCreator) -> Self {
        let (sender, receiver): (Sender<Command>, Receiver<Command>) = mpsc::channel();

        Self {
            objects: HashMap::new(),
            vbuf_creator,
            command_recv: Some(receiver),
            command_send: sender,
            renderpass: None,
            device: None,
        }
    }

    pub fn update_renderpass(&mut self, renderpass: Arc<RenderPassAbstract + Send + Sync>) {
        self.renderpass = Some(renderpass);
    }

    pub fn update_device(&mut self, device: Arc<Device>) {
        self.device = Some(device);
    }

    pub fn update_materials(&mut self) {
        if self.renderpass.is_none() || self.device.is_none() {
            panic!("You tried to initialize materials without first setting the renderpass and device!");
        }

        let device = self.device.as_mut().unwrap().clone();
        let renderpass = self.renderpass.as_mut().unwrap().clone();
        self.objects.values_mut().for_each(|object| object.material.create_pipeline(device.clone(), renderpass.clone()));
    }

    pub fn get_communicator(&self) -> WorldCommunicator {
        WorldCommunicator::from_sender(self.command_send.clone())
    }

    pub fn add_object_from_vbuf(&mut self, id: String, vbuf: Arc<VertexBuffer>) {
        let new_object = Object {
            vbuf,
            material: Material::default(),
        };

        self.objects.insert(id, new_object);
    }

    pub fn add_object_from_verts(&mut self, id: String, verts: Vec<Vertex>) {
        let vbuf = self.vbuf_creator.create_vbuf_from_verts(&verts);
        let new_object = Object {
            vbuf,
            material: Material::default(),
        };

        self.objects.insert(id, new_object);
    }

    pub fn add_draw_commands(&self, command_buffer: AutoCommandBufferBuilder, dynamic_state: &DynamicState, uniform_set: Arc<vulkano::descriptor::descriptor_set::DescriptorSet + Send + Sync>) -> AutoCommandBufferBuilder {
        let mut command_buffer_unfinished = command_buffer;
        for object in self.objects.values() {
            command_buffer_unfinished = command_buffer_unfinished
                .draw(
                    object.get_pipeline(),
                    dynamic_state,
                    object.get_vbuf(),
                    uniform_set.clone(),
                    (),
                )
                .unwrap();
        }

        command_buffer_unfinished
    }

    pub fn delete_object(&mut self, id: String) {
        self.objects.remove(&id);
    }

    pub fn check_for_commands(&mut self) {
        let command_recv = self.command_recv.take().unwrap();

        command_recv.try_iter().for_each(|command| match command {
            Command::ObjectFromVbuf { id, vbuf } => self.add_object_from_vbuf(id, vbuf),
            Command::ObjectFromVerts { id, verts } => self.add_object_from_verts(id, verts),
            Command::DeleteObject { id } => self.delete_object(id),
            Command::UpdateMaterials => self.update_materials(),
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

    pub fn add_object_from_vbuf(&mut self, id: String, vbuf: Arc<VertexBuffer>) {
        let command = Command::ObjectFromVbuf { id, vbuf };

        self.command_send.send(command).unwrap();
    }

    pub fn add_object_from_verts(&mut self, id: String, verts: Vec<Vertex>) {
        let command = Command::ObjectFromVerts { id, verts };

        self.command_send.send(command).unwrap();
    }

    pub fn delete_object(&mut self, id: String) {
        let command = Command::DeleteObject { id };

        self.command_send.send(command).unwrap();
    }

    pub fn update_materials(&mut self) {
        let command = Command::UpdateMaterials;

        self.command_send.send(command).unwrap();
    }
}

impl Object {
    fn get_vbuf(&self) -> Arc<VertexBuffer> {
        self.vbuf.clone()
    }

    fn get_pipeline(&self) -> Arc<ConcreteGraphicsPipeline> {
        self.material.get_pipeline()
    }
}

impl Material {
    pub fn default() -> Self {
        Self {
            cached_pipeline: None,
            fill_type: PrimitiveTopology::TriangleList,
        }
    }

    pub fn pipeline_is_cached(&self) -> bool {
        self.cached_pipeline.is_some()
    }

    pub fn create_pipeline(&mut self, device: Arc<Device>, renderpass: Arc<RenderPassAbstract + Send + Sync>) {
        let vs = vs::Shader::load(device.clone()).unwrap();
        let fs = fs::Shader::load(device.clone()).unwrap();

        let pipeline = Arc::new(
            GraphicsPipeline::start()
                .vertex_input_single_buffer()
                .vertex_shader(vs.main_entry_point(), ())
                .primitive_topology(self.fill_type)
                .viewports_dynamic_scissors_irrelevant(1)
                .fragment_shader(fs.main_entry_point(), ())
                .render_pass(Subpass::from(renderpass.clone(), 0).unwrap())
                .depth_stencil_simple_depth()
                .build(device.clone())
                .unwrap(),
        );

        self.cached_pipeline = Some(pipeline);
    }

    pub fn get_pipeline(&self) -> Arc<ConcreteGraphicsPipeline> {
        if let Some(pipeline) = &self.cached_pipeline {
            pipeline.clone()
        } else {
            panic!("You tried to get the pipeline of a material without creating its pipeline first! you evil, evil person.");
        }
    }
}

mod vs {
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
