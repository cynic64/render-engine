use render_engine::mesh::{Mesh, PrimitiveTopology};
use render_engine::object::ObjectPrototype;
use render_engine::render_passes;
use render_engine::system::{Pass, System};
use render_engine::utils::load_texture;
use render_engine::window::Window;
use render_engine::Format;

use nalgebra_glm::*;

use std::collections::HashMap;

use tests_render_engine::mesh::{VPos2D, VPosColor2D};
use tests_render_engine::{relative_path, Matrix4};

fn main() {
    // initialize window
    let (mut window, queue) = Window::new();
    let device = queue.device().clone();

    // create system
    let render_pass = render_passes::basic(device.clone());
    let mut system = System::new(
        queue.clone(),
        vec![Pass {
            name: "geometry",
            images_created_tags: vec!["color"],
            images_needed_tags: vec![],
            render_pass: render_pass.clone(),
        }],
        // custom images, we use none
        HashMap::new(),
        "color",
    );

    window.set_render_pass(render_pass.clone());

    // create data for model matrix
    let model_data: Matrix4 = scale(&Mat4::identity(), &vec3(0.1, 0.1, 0.1)).into();

    // create objects
    let object1 = ObjectPrototype {
        vs_path: relative_path("shaders/data-testing/vert_model.glsl"),
        fs_path: relative_path("shaders/data-testing/frag_model.glsl"),
        fill_type: PrimitiveTopology::TriangleList,
        read_depth: false,
        write_depth: false,
        mesh: Mesh {
            vertices: vec![
                VPosColor2D {
                    position: [0.0, -1.0],
                    color: [1.0, 0.0, 0.0],
                },
                VPosColor2D {
                    position: [-1.0, 1.0],
                    color: [0.0, 1.0, 0.0],
                },
                VPosColor2D {
                    position: [1.0, 1.0],
                    color: [0.0, 0.0, 1.0],
                },
            ],
            indices: vec![0, 1, 2],
        },
        collection: ((model_data,),),
        custom_dynamic_state: None,
    }
    .build(queue.clone(), render_pass.clone());

    let texture = load_texture(
        queue.clone(),
        &relative_path("textures/rust-logo.png"),
        Format::B8G8R8A8Unorm,
    );

    let object2 = ObjectPrototype {
        vs_path: relative_path("shaders/data-testing/vert_tex.glsl"),
        fs_path: relative_path("shaders/data-testing/frag_tex.glsl"),
        fill_type: PrimitiveTopology::TriangleList,
        read_depth: false,
        write_depth: false,
        mesh: Mesh {
            vertices: vec![
                VPos2D {
                    position: [0.0, -1.0],
                },
                VPos2D {
                    position: [-1.0, 1.0],
                },
                VPos2D {
                    position: [1.0, 1.0],
                },
            ],
            indices: vec![0, 1, 2],
        },
        collection: ((texture,),),
        custom_dynamic_state: None,
    }
    .build(queue.clone(), render_pass.clone());

    // used in main loop
    while !window.update() {
        // draw
        system.start_window(&mut window);
        system.add_object(&object2);
        system.add_object(&object1);
        system.finish_to_window(&mut window);
    }

    println!("FPS: {}", window.get_fps());
}
