use render_engine::collection::{Data, Set};
use render_engine::input::get_elapsed;
use render_engine::mesh::PrimitiveTopology;
use render_engine::object::{Object, ObjectPrototype};
use render_engine::render_passes;
use render_engine::system::{Pass, System};
use render_engine::window::Window;
use render_engine::Image;

use std::collections::HashMap;
use std::env;
use std::path::Path;

use nalgebra_glm::{scale, vec3, Mat4};

use tests_render_engine::mesh::{add_tangents_multi, convert_meshes, load_obj, load_textures};
use tests_render_engine::{relative_path, CameraData, FlyCamera, Matrix4};

fn main() {
    // get path to load_obj
    let args: Vec<String> = env::args().collect();
    let path = if args.len() < 2 {
        println!("No path given to load!");
        return;
    } else {
        Path::new(&args[1])
    };

    // initialize window
    let (mut window, queue) = Window::new();
    let device = queue.device().clone();

    // create system
    let render_pass = render_passes::multisampled_with_depth(device.clone(), 4);
    let mut system = System::new(
        queue.clone(),
        vec![Pass {
            name: "geometry",
            images_created_tags: vec![
                "resolve_color",
                "multisampled_color",
                "multisampled_depth",
            ],
            images_needed_tags: vec![],
            render_pass: render_pass.clone(),
        }],
        // custom images, we use none
        HashMap::new(),
        "resolve_color",
    );

    window.set_render_pass(render_pass.clone());

    // initialize camera
    let mut camera = FlyCamera::default();

    // light
    let moving_light = MovingLight::new();
    let light_data = moving_light.get_data();

    // load meshes and materials
    let (models, materials) = load_obj(&path).expect("Couldn't open OBJ file");
    let meshes = add_tangents_multi(&convert_meshes(&models));
    let textures_path = path.parent().expect("Given path has no parent!");
    println!("Searching for textures in {:?}", textures_path);
    let texture_sets = load_textures(queue.clone(), textures_path, &materials);

    let default_material = Material {
        ambient: [1.0, 1.0, 1.0, 0.0],
        diffuse: [0.0, 0.0, 1.0, 0.0],
        specular: [1.0, 1.0, 1.0, 0.0],
        shininess: [32.0, 0.0, 0.0, 0.0],
        use_texture: [0.0, 0.0, 0.0, 0.0],
    };
    let model_mat: Matrix4 = Mat4::identity().into();

    // combine the meshes and textures to create a list of renderable objects

    // i don't think the type annotation is necessary is here, but i included it
    // anyway to show how information about the object's uniforms is stored in
    // its type.
    let mut objects: Vec<
        Object<(
            // material and model matrix
            Set<(Material, Matrix4)>,
            // textures (diffuse, specular, normal)
            Set<(Image, Image, Image)>,
            // camera matrices and light position
            Set<(CameraData, Light)>,
        )>,
    > = meshes
        .into_iter()
        .enumerate()
        .map(|(idx, mesh)| {
            let model = &models[idx];

            let mat_idx = if let Some(idx) = model.mesh.material_id {
                idx
            } else {
                println!("Model {} has no material id! Using 0.", model.name);
                0
            };

            let material = if model.mesh.material_id.is_some() && mat_idx < materials.len() {
                Material::from_tobj(&materials[mat_idx])
            } else {
                default_material.clone()
            };

            let textures = texture_sets[mat_idx].clone();

            let object = ObjectPrototype {
                vs_path: relative_path("shaders/obj-viewer/vert.glsl"),
                fs_path: relative_path("shaders/obj-viewer/frag.glsl"),
                fill_type: PrimitiveTopology::TriangleList,
                read_depth: true,
                write_depth: true,
                mesh,
                collection: (
                    (material.clone(), model_mat),
                    textures,
                    (camera.get_data(), light_data.clone()),
                ),
                custom_dynamic_state: None,
            }
            .build(queue.clone(), render_pass.clone());

            object
        })
        .collect();

    println!("Objects Loaded: {}", objects.len());

    // used in main loop
    while !window.update() {
        // get updated info on camera and light
        camera.update(window.get_frame_info());
        let camera_data = camera.get_data();
        let light_data = moving_light.get_data();

        // update collections
        objects.iter_mut().for_each(|obj| {
            obj.collection.2.data.0 = camera_data.clone();
            obj.collection.2.data.1 = light_data.clone();
            obj.collection.2.upload(device.clone());
        });

        // draw
        system.start_window(&mut window);

        for object in objects.iter() {
            system.add_object(object);
        }

        system.finish_to_window(&mut window);
    }

    println!("FPS: {}", window.get_fps());
}

#[derive(Clone)]
struct Light {
    direction: [f32; 4],
    power: f32,
}

impl Data for Light {}

struct MovingLight {
    start_time: std::time::Instant,
}

impl MovingLight {
    fn new() -> Self {
        Self {
            start_time: std::time::Instant::now(),
        }
    }

    fn get_data(&self) -> Light {
        let time = get_elapsed(self.start_time) / 4.0;
        Light {
            direction: [time.sin(), 2.0, time.cos(), 0.0],
            power: 1.0,
        }
    }
}

#[derive(Clone)]
struct Material {
    ambient: [f32; 4],
    diffuse: [f32; 4],
    specular: [f32; 4],
    shininess: [f32; 4],
    use_texture: [f32; 4],
}

impl Data for Material {}

impl Material {
    fn from_tobj(material: &tobj::Material) -> Self {
        let (amb, diff, spec) = (material.ambient, material.diffuse, material.specular);
        let shine = material.shininess;

        let use_tex = if contains_textures(&material) {
            1.0
        } else {
            0.0
        };

        Material {
            ambient: [amb[0], amb[1], amb[2], 1.0],
            diffuse: [diff[0], diff[1], diff[2], 1.0],
            specular: [spec[0], spec[1], spec[2], 1.0],
            shininess: [shine, 0.0, 0.0, 0.0],
            use_texture: [use_tex, 0.0, 0.0, 0.0],
        }
    }
}

fn contains_textures(material: &tobj::Material) -> bool {
    material.normal_texture != "" || material.specular_texture != "" || material.diffuse_texture != ""
}
