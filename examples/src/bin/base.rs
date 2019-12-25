use render_engine::mesh::PrimitiveTopology;
use render_engine::render_passes;
use render_engine::system::{Pass, System};
use render_engine::window::Window;
use render_engine::object::ObjectPrototype;
use render_engine::utils::Timer;

use nalgebra_glm::*;

use std::collections::HashMap;

use tests_render_engine::mesh::{convert_meshes, load_obj};
use tests_render_engine::{relative_path, OrbitCamera, Matrix4};

fn main() {
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

    // create buffer and set for model matrix
    let model_data: Matrix4 = Mat4::identity().into();

    // initialize camera
    let mut camera = OrbitCamera::default();
    let camera_data = camera.get_data();

    // load, create pipeline spec and set for model matrix
    // only load 1st object
    let (mut models, _materials) = load_obj(&relative_path("meshes/dragon.obj")).expect("couldn't load OBJ");
    let mesh = convert_meshes(&[models.remove(0)]).remove(0);
    dbg!["done loading meshes"];

    let mut object = ObjectPrototype {
        vs_path: relative_path("shaders/base/vert.glsl"),
        fs_path: relative_path("shaders/base/frag.glsl"),
        fill_type: PrimitiveTopology::TriangleList,
        read_depth: true,
        write_depth: true,
        mesh,
        collection: (
            (model_data,),
            (camera_data,),
        ),
        custom_dynamic_state: None,
    }
    .build(queue.clone(), render_pass.clone());

    let mut camera_timer = Timer::new("Camera uniform buffer");

    while !window.update() {
        // update camera and camera buffer
        camera_timer.start();
        camera.update(window.get_frame_info());

        let camera_data = camera.get_data();

        object.collection.1.data.0 = camera_data;

        object.collection.1.upload(device.clone());
        camera_timer.stop();

        // draw
        system.start_window(&mut window);
        system.add_object(&object);
        system.finish_to_window(&mut window);
    }

    println!("FPS: {}", window.get_fps());
    println!("Avg. delta: {}", window.get_avg_delta() * 1_000.0);
    window.print_avg_update();
    system.print_stats();
    camera_timer.print();
}
