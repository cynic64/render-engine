use vulkano::framebuffer::{RenderPassAbstract, Subpass};
use vulkano::device::Device;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::input_assembly::PrimitiveTopology;
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer, BufferAccess};

use std::collections::HashMap;
use std::path::Path;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::Arc;

use crate::mesh_gen;
use crate::shaders::*;
use crate::system::{Vertex, RenderableObject};

// the world stores objects and can produce a list of renderable objects
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
    pub shaders: ShaderSystem,
}

impl World {
    pub fn new(
        render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
        device: Arc<Device>,
    ) -> Self {
        let (sender, receiver): (Sender<Command>, Receiver<Command>) = channel();

        Self {
            objects: HashMap::new(),
            command_recv: Some(receiver),
            command_send: sender,
            render_pass,
            device,
        }
    }

    pub fn set_render_pass(
        &mut self,
        new_render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    ) {
        self.render_pass = new_render_pass;
    }

    pub fn get_communicator(&self) -> WorldCommunicator {
        WorldCommunicator::from_sender(self.command_send.clone())
    }

    pub fn add_object_from_spec(&mut self, id: String, spec: ObjectSpec) {
        let vbuf = spec.mesh.create_vbuf(self.device.clone());

        let (vs_main, fs_main) = spec.material.shaders.get_entry_points();

        let pipeline = Arc::new(
            GraphicsPipeline::start()
                .vertex_input_single_buffer::<Vertex>()
                .vertex_shader(vs_main, ())
                .primitive_topology(spec.material.fill_type)
                .viewports_dynamic_scissors_irrelevant(1)
                .fragment_shader(fs_main, ())
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
        self.objects
            .values()
            .map(|(_spec, obj)| obj.clone())
            .collect()
    }

    pub fn delete_object(&mut self, id: &str) {
        self.objects.remove(id);
    }

    pub fn update(&mut self) {
        self.check_for_commands();
    }

    fn check_for_commands(&mut self) {
        let command_recv = self.command_recv.take().unwrap();

        command_recv.try_iter().for_each(|command| match command {
            Command::AddObjectFromSpec { id, spec } => self.add_object_from_spec(id, spec),
            Command::DeleteObject { id } => self.delete_object(&id),
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

    pub fn add_object_from_spec(&mut self, id: &str, spec: ObjectSpec) {
        let command = Command::AddObjectFromSpec { id: id.to_string(), spec };

        self.command_send.send(command).unwrap();
    }

    pub fn delete_object(&mut self, id: &str) {
        let command = Command::DeleteObject { id: id.to_string() };

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
    custom_shaders: Option<ShaderSystem>,
}

impl ObjectSpecBuilder {
    pub fn default() -> Self {
        Self {
            custom_mesh: None,
            custom_fill_type: None,
            custom_shaders: None,
        }
    }

    pub fn mesh<M: Mesh + 'static>(self, mesh: M) -> Self {
        Self {
            custom_mesh: Some(Box::new(mesh)),
            ..self
        }
    }

    pub fn shaders(self, shaders: ShaderSystem) -> Self {
        Self {
            custom_shaders: Some(shaders),
            ..self
        }
    }

    pub fn build(self, device: Arc<Device>) -> ObjectSpec {
        let fill_type = self
            .custom_fill_type
            .unwrap_or(PrimitiveTopology::TriangleList);

        // if you choose to customize shaders, you need to provide both
        let shaders = self.custom_shaders.unwrap_or_else(|| {
            let vert_path = Path::new(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/shaders/forward/default_vert.glsl"
            ));

            let frag_path = Path::new(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/shaders/forward/default_frag.glsl"
            ));

            ShaderSystem::load_from_file(device.clone(), &vert_path, &frag_path)
        });

        let material = Material { fill_type, shaders };

        // if no mesh is provided, load a cube
        let mesh = self.custom_mesh.unwrap_or_else(|| {
            Box::new(mesh_gen::create_vertices_for_cube(
                [0.0, 0.0, 0.0],
                1.0,
            ))
        });

        ObjectSpec {
            mesh,
            material,
        }
    }
}
