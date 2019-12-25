use render_engine::object::ObjectPrototype;
use render_engine::render_passes;
use render_engine::system::{Pass, System};
use render_engine::window::Window;
use render_engine::mesh::{PrimitiveTopology, Mesh};
use render_engine::utils::Timer;

use std::collections::HashMap;

use tests_render_engine::mesh::VPosColor2D;
use tests_render_engine::relative_path;

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

    // load, create pipeline spec and set for model matrix
    let object = ObjectPrototype {
        vs_path: relative_path("shaders/triangle/vert.glsl"),
        fs_path: relative_path("shaders/triangle/frag.glsl"),
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
        collection: (),
        custom_dynamic_state: None,
    }
    .build(queue.clone(), render_pass.clone());

    let mut start_window_timer = Timer::new("Start window");
    let mut add_object_timer = Timer::new("Add object");
    let mut finish_window_timer = Timer::new("Finish to window");
    let mut update_timer = Timer::new("Handle events");
    let mut total_timer = Timer::new("Total");

    update_timer.start();
    while !window.update() {
        update_timer.stop();

        total_timer.start();

        // draw
        start_window_timer.start();
        system.start_window(&mut window);
        start_window_timer.stop();

        add_object_timer.start();
        system.add_object(&object);
        add_object_timer.stop();

        finish_window_timer.start();
        system.finish_to_window(&mut window);
        finish_window_timer.stop();

        total_timer.stop();
        update_timer.start();
    }

    println!("FPS: {}", window.get_fps());
    println!("Avg delta: {}ms", window.get_avg_delta() * 1_000.0);
    system.print_stats();

    println!("-----");
    start_window_timer.print();
    add_object_timer.print();
    finish_window_timer.print();
    update_timer.print();
    total_timer.print();
}
