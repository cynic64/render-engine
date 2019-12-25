use render_engine::collection::Data;
use render_engine::input::{get_elapsed, VirtualKeyCode};
use render_engine::mesh::PrimitiveTopology;
use render_engine::object::{Drawcall, Object, ObjectPrototype};
use render_engine::render_passes;
use render_engine::system::{Pass, System};
use render_engine::utils::Timer;
use render_engine::window::Window;
use render_engine::{Format, Image};

use vulkano::command_buffer::DynamicState;
use vulkano::pipeline::viewport::Viewport;

use std::collections::HashMap;
use std::sync::Arc;

use nalgebra_glm::*;

use tests_render_engine::mesh::{
    add_tangents_multi, convert_meshes, fullscreen_quad, load_obj, load_textures, merge, only_pos,
    only_pos_from_ptnt, wireframe,
};
use tests_render_engine::{relative_path, FlyCamera, Matrix4};

const SHADOW_MAP_DIMS: [u32; 2] = [6_144, 1024];
const PATCH_DIMS: [f32; 2] = [1024.0, 1024.0];

fn main() {
    // initialize window
    let (mut window, queue) = Window::new();
    let device = queue.device().clone();

    // create system
    let patched_shadow: Image = vulkano::image::AttachmentImage::sampled(
        device.clone(),
        SHADOW_MAP_DIMS,
        Format::D32Sfloat,
    )
    .unwrap();
    let shadow_blur: Image = vulkano::image::AttachmentImage::sampled(
        device.clone(),
        // SHADOW_MAP_DIMS,
        SHADOW_MAP_DIMS,
        Format::D32Sfloat,
    )
    .unwrap();
    let mut custom_images = HashMap::new();
    custom_images.insert("shadow_map", patched_shadow);
    custom_images.insert("shadow_map_blur", shadow_blur);

    let render_pass = render_passes::read_depth(device.clone());
    let rpass_shadow = render_passes::only_depth(device.clone());
    let rpass_shadow_blur = render_passes::only_depth(device.clone());
    let rpass_cubeview = render_passes::basic(device.clone());
    let rpass_prepass = render_passes::only_depth(device.clone());

    let mut system = System::new(
        queue.clone(),
        vec![
            // renders to shadow cubemap
            Pass {
                name: "shadow",
                images_created_tags: vec!["shadow_map"],
                images_needed_tags: vec![],
                render_pass: rpass_shadow.clone(),
            },
            // blurs shadow cubemap
            Pass {
                name: "shadow_blur",
                images_created_tags: vec!["shadow_map_blur"],
                images_needed_tags: vec!["shadow_map"],
                render_pass: rpass_shadow_blur.clone(),
            },
            // depth prepass
            Pass {
                name: "depth_prepass",
                images_created_tags: vec!["depth_prepass"],
                images_needed_tags: vec![],
                render_pass: rpass_prepass.clone(),
            },
            // displays any depth buffer for debugging
            Pass {
                name: "depth_viewer",
                images_created_tags: vec!["depth_view"],
                images_needed_tags: vec!["depth_prepass"],
                render_pass: rpass_cubeview.clone(),
            },
            // final pass
            Pass {
                name: "geometry",
                images_created_tags: vec!["color", "depth_prepass"],
                images_needed_tags: vec!["shadow_map_blur"],
                render_pass: render_pass.clone(),
            },
        ],
        custom_images,
        "color",
    );

    window.set_render_pass(render_pass.clone());

    // initialize camera
    let mut camera = FlyCamera::default();
    camera.yaw = 0.0;
    camera.position = vec3(0.0, 10.0, 0.0);
    let camera_data = camera.get_data();

    // light
    let light = MovingLight::new();
    let light_data = light.get_data();

    // a model buffer with .1 scale, used for a couple different objects
    let model_data: Matrix4 = scale(&Mat4::identity(), &vec3(0.1, 0.1, 0.1)).into();

    // a default material, at some point I want to get rid of Material
    // altogether and just use textures
    let material_data = Material {
        ambient: [1.0, 1.0, 1.0, 1.0],
        diffuse: [1.0, 1.0, 1.0, 1.0],
        specular: [1.0, 1.0, 1.0, 1.0],
        shininess: [32.0, 0.0, 0.0, 0.0],
        use_texture: [1.0, 1.0, 1.0, 1.0],
    };

    // load obj
    let (models, materials) =
        load_obj(&relative_path("meshes/sponza/sponza.obj")).expect("Couldn't load OBJ file");

    // convert to meshes and load textures
    let meshes = add_tangents_multi(&convert_meshes(&models));
    let textures = load_textures(queue.clone(), &relative_path("meshes/sponza/"), &materials);

    // create objects for the geometry pass
    let mut geo_objects: Vec<Object<_>> = meshes
        .iter()
        .enumerate()
        .map(|(idx, mesh)| {
            let model = &models[idx];

            let mat_idx = if let Some(idx) = model.mesh.material_id {
                idx
            } else {
                println!("Model {} has no material id! Using 0.", model.name);
                0
            };
            let textures = textures[mat_idx].clone();

            ObjectPrototype {
                vs_path: relative_path("shaders/pretty/vert.glsl"),
                fs_path: relative_path("shaders/pretty/all_frag.glsl"),
                fill_type: PrimitiveTopology::TriangleList,
                read_depth: true,
                write_depth: true,
                mesh: mesh.clone(),
                collection: (
                    (material_data.clone(), model_data),
                    textures,
                    (camera_data.clone(), light_data.clone()),
                ),
                custom_dynamic_state: None,
            }
            .build(queue.clone())
        })
        .collect();

    println!("Objects Loaded: {}", geo_objects.len());

    // shadow stuff
    // create fullscreen quad to debug cubemap
    let quad_display = fullscreen_quad(
        queue.clone(),
        relative_path("shaders/pretty/fullscreen_vert.glsl"),
        relative_path("shaders/pretty/display_cubemap_frag.glsl"),
    );

    // and to blur shadow map
    let mut quad_blur = fullscreen_quad(
        queue.clone(),
        relative_path("shaders/pretty/fullscreen_vert.glsl"),
        relative_path("shaders/pretty/blur_frag.glsl"),
    );
    quad_blur.pipeline_spec.write_depth = true;

    // merge meshes for use in depth prepass and shadow casting
    let merged_mesh = merge(&meshes);
    let merged_mesh_pos_only = only_pos_from_ptnt(&merged_mesh);

    let shadow_cast_base = ObjectPrototype {
        vs_path: relative_path("shaders/pretty/shadow_cast_vert.glsl"),
        fs_path: relative_path("shaders/pretty/shadow_cast_frag.glsl"),
        fill_type: PrimitiveTopology::TriangleList,
        read_depth: true,
        write_depth: true,
        mesh: merged_mesh_pos_only.clone(),
        // convert_to_shadow_casters adds proper collections
        collection: (),
        custom_dynamic_state: None,
    }
    .build(queue.clone());

    let mut depth_prepass_object = ObjectPrototype {
        vs_path: relative_path("shaders/pretty/depth_prepass_vert.glsl"),
        fs_path: relative_path("shaders/pretty/depth_prepass_frag.glsl"),
        fill_type: PrimitiveTopology::TriangleList,
        read_depth: true,
        write_depth: true,
        mesh: merged_mesh_pos_only,
        collection: ((model_data,), (camera_data.clone(),)),
        custom_dynamic_state: None,
    }
    .build(queue.clone());

    // create mesh for light (just a sphere)
    // we need 2 objects: one for the depth prepass and one for the geometry stage
    let light_mesh = {
        let (models, _materials) =
            load_obj(&relative_path("meshes/sphere.obj")).expect("Couldn't load OBJ file");
        convert_meshes(&[models[0].clone()]).remove(0)
    };

    let mut light_object_prepass = ObjectPrototype {
        vs_path: relative_path("shaders/pretty/depth_prepass_vert.glsl"),
        fs_path: relative_path("shaders/pretty/depth_prepass_frag.glsl"),
        fill_type: PrimitiveTopology::TriangleList,
        read_depth: true,
        write_depth: true,
        mesh: only_pos(&light_mesh),
        collection: ((model_data,), (camera_data.clone(),)),
        custom_dynamic_state: None,
    }
    .build(queue.clone());

    let mut light_object_geo = ObjectPrototype {
        vs_path: relative_path("shaders/pretty/vert.glsl"),
        fs_path: relative_path("shaders/pretty/light_frag.glsl"),
        fill_type: PrimitiveTopology::TriangleList,
        read_depth: true,
        write_depth: true,
        mesh: light_mesh,
        collection: (
            (material_data.clone(), model_data),
            // take the textures of the first object just to fill the space
            // maybe eventually give the light its own vertex shader
            textures[0].clone(),
            (camera_data.clone(), light_data.clone()),
        ),
        custom_dynamic_state: None,
    }
    .build(queue.clone());

    // create wireframe mesh
    let wireframe_mesh = wireframe(&only_pos_from_ptnt(&merged_mesh));
    let mut wireframe_object = ObjectPrototype {
        // the light vertex shader does exactly the same we need to do, just
        // converts the position to screen space and nothing else, so we re-use
        // it
        vs_path: relative_path("shaders/pretty/light_vert.glsl"),
        fs_path: relative_path("shaders/pretty/wireframe_frag.glsl"),
        fill_type: PrimitiveTopology::LineList,
        read_depth: true,
        write_depth: true,
        mesh: wireframe_mesh,
        collection: ((model_data,), (camera_data,)),
        custom_dynamic_state: None,
    }
    .build(queue.clone());

    let mut all_objects: HashMap<&str, Vec<Arc<dyn Drawcall>>> = HashMap::new();

    // used in main loop
    let mut timer_setup = Timer::new("Setup time");
    let mut timer_draw = Timer::new("Overall draw time");

    all_objects.insert("depth_viewer", vec![Arc::new(quad_display)]);
    all_objects.insert("shadow_blur", vec![Arc::new(quad_blur)]);

    let mut view_mode: i32 = 0;
    let mut update_view = false;
    let mut draw_wireframe = false;
    let mut cursor_grabbed = true;

    while !window.update() {
        timer_setup.start();

        // convert merged mesh into 6 casters, one for each cubemap face
        let shadow_casters = convert_to_shadow_casters(shadow_cast_base.clone(), light.get_data());
        // update camera, but only if we're grabbing the cursor
        if cursor_grabbed {
            camera.update(window.get_frame_info());
        }
        let camera_data = camera.get_data();

        // update light
        let light_data = light.get_data();

        // update depth prepass objects' collections
        (depth_prepass_object.collection.1).0 = camera_data.clone();
        (light_object_prepass.collection.1).0 = camera_data.clone();

        // the light has moved, we need to update its model matrix
        let light_model_data: Matrix4 = scale(
            &translate(&Mat4::identity(), &make_vec3(&light_data.position)),
            &vec3(0.03, 0.03, 0.03),
        )
        .into();
        (light_object_prepass.collection.0).0 = light_model_data;

        all_objects.insert(
            "depth_prepass",
            vec![
                Arc::new(depth_prepass_object.clone()),
                Arc::new(light_object_prepass.clone()),
            ],
        );

        if window
            .get_frame_info()
            .keydowns
            .contains(&VirtualKeyCode::C)
            || update_view
        {
            view_mode += 1;
            update_view = false;

            match view_mode {
                0 => {
                    // default: everything enabled
                    system.output_tag = "color";
                }
                1 => {
                    // pure white
                    geo_objects.iter_mut().for_each(|obj| {
                        obj.pipeline_spec.fs_path = relative_path("shaders/pretty/white_frag.glsl");
                    });
                    system.output_tag = "color";
                }
                2 => {
                    // depth only
                    system.output_tag = "depth_view";
                }
                3 => {
                    // diffuse_only
                    geo_objects.iter_mut().for_each(|obj| {
                        obj.pipeline_spec.fs_path =
                            relative_path("shaders/pretty/diffuse_only_frag.glsl");
                    });
                    system.output_tag = "color";
                }
                4 => {
                    // diffuse and light direction
                    geo_objects.iter_mut().for_each(|obj| {
                        obj.pipeline_spec.fs_path =
                            relative_path("shaders/pretty/diffuse_and_light_frag.glsl");
                    });
                    system.output_tag = "color";
                }
                5 => {
                    // diffuse and light distance + direction
                    geo_objects.iter_mut().for_each(|obj| {
                        obj.pipeline_spec.fs_path =
                            relative_path("shaders/pretty/diffuse_light_distance_frag.glsl");
                    });
                    system.output_tag = "color";
                }
                6 => {
                    // diffuse and specular
                    geo_objects.iter_mut().for_each(|obj| {
                        obj.pipeline_spec.fs_path =
                            relative_path("shaders/pretty/diffuse_and_spec.glsl");
                    });
                    system.output_tag = "color";
                }
                7 => {
                    // diffuse, specular, normal mapping
                    geo_objects.iter_mut().for_each(|obj| {
                        obj.pipeline_spec.fs_path =
                            relative_path("shaders/pretty/diffuse_spec_normal.glsl");
                    });
                    system.output_tag = "color";
                }
                8 => {
                    // shadows only
                    geo_objects.iter_mut().for_each(|obj| {
                        obj.pipeline_spec.fs_path =
                            relative_path("shaders/pretty/shadows_only.glsl");
                    });
                    system.output_tag = "color";
                }
                9 => {
                    // diffuse + spec + normal mapping + shadows
                    geo_objects.iter_mut().for_each(|obj| {
                        obj.pipeline_spec.fs_path =
                            relative_path("shaders/pretty/shadows_and_color.glsl");
                    });
                    system.output_tag = "color";
                }
                10 => {
                    // diffuse + spec + normal mapping + shadows + tonemapping
                    geo_objects.iter_mut().for_each(|obj| {
                        obj.pipeline_spec.fs_path = relative_path("shaders/pretty/all_frag.glsl");
                    });
                    system.output_tag = "color";
                }
                11 => {
                    // specular only
                    geo_objects.iter_mut().for_each(|obj| {
                        obj.pipeline_spec.fs_path =
                            relative_path("shaders/pretty/specular_only.glsl");
                    });
                    system.output_tag = "color";
                }
                12 => {
                    // specular only, low shininess
                    geo_objects.iter_mut().for_each(|obj| {
                        obj.pipeline_spec.fs_path =
                            relative_path("shaders/pretty/specular_only_2.glsl");
                    });
                    system.output_tag = "color";
                }
                13 => {
                    // normals
                    geo_objects.iter_mut().for_each(|obj| {
                        obj.pipeline_spec.fs_path =
                            relative_path("shaders/pretty/normals_only.glsl");
                    });
                    system.output_tag = "color";
                }
                _ => {
                    geo_objects.iter_mut().for_each(|obj| {
                        obj.pipeline_spec.fs_path = relative_path("shaders/pretty/all_frag.glsl");
                    });
                    view_mode = 0;
                    system.output_tag = "color";
                }
            }
        }

        if window
            .get_frame_info()
            .keydowns
            .contains(&VirtualKeyCode::V)
        {
            view_mode -= 2;
            update_view = true;
        }

        if window
            .get_frame_info()
            .keydowns
            .contains(&VirtualKeyCode::Escape)
        {
            cursor_grabbed = !cursor_grabbed;
            if cursor_grabbed {
                window.get_surface().window().hide_cursor(true);
                window.set_recenter(true);
            } else {
                window.get_surface().window().hide_cursor(false);
                window.set_recenter(false);
            }
        }

        (light_object_geo.collection.0).1 = light_model_data;

        geo_objects
            .iter_mut()
            .for_each(|obj| obj.collection.2 = (camera_data.clone(), light_data.clone()));

        if draw_wireframe {
            (wireframe_object.collection.1).0 = camera_data.clone();
            // geometry_object_list.push(cur_wireframe_object.clone());
        }

        if window
            .get_frame_info()
            .keydowns
            .contains(&VirtualKeyCode::R)
        {
            draw_wireframe = !draw_wireframe;
        }

        all_objects.insert(
            "geometry",
            geo_objects
                .iter()
                .map(|obj| {
                    let dc: Arc<dyn Drawcall> = Arc::new(obj.clone());
                    dc
                })
                .collect(),
        );
        all_objects.insert(
            "shadow",
            shadow_casters
                .iter()
                .map(|obj| {
                    let dc: Arc<dyn Drawcall> = Arc::new(obj.clone());
                    dc
                })
                .collect(),
        );

        /*
        let mut cur_wireframe_object = wireframe_object.clone();
        cur_wireframe_object.custom_sets.push(camera_set.clone());
        */

        timer_setup.stop();

        // draw
        timer_draw.start();
        system.render_to_window(&mut window, all_objects.clone());
        timer_draw.stop();
    }

    system.print_stats();
    println!("FPS: {}", window.get_fps());
    println!("Avg. delta: {} ms", window.get_avg_delta() * 1_000.0);
    timer_setup.print();
    timer_draw.print();
}

#[allow(dead_code)]
#[derive(Clone)]
struct Light {
    position: [f32; 4],
    strength: f32,
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
        let time = get_elapsed(self.start_time) / 16.0;
        Light {
            position: [time.sin() * 100.0, 10.0, 0.0, 0.0],
            strength: 1.0,
        }
    }
}

fn convert_to_shadow_casters(
    base_object: Object<()>,
    light_data: Light,
) -> Vec<Object<((Matrix4,), (Matrix4,), (Matrix4,), (Light,))>> {
    // if you want to make point lamps cast shadows, you need shadow cubemaps
    // render-engine doesn't support geometry shaders, so the easiest way to do
    // this is to convert one object into 6 different ones, one for each face of
    // the cubemap, that each render to a different part of a 2D texture.
    // for now this function assumes a 6x1 patch layout
    let view_directions = [
        vec3(1.0, 0.0, 0.0),
        vec3(-1.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        vec3(0.0, -1.0, 0.0),
        vec3(0.0, 0.0, 1.0),
        vec3(0.0, 0.0, -1.0),
    ];

    let up_directions = [
        vec3(0.0, -1.0, 0.0),
        vec3(0.0, -1.0, 0.0),
        vec3(0.0, 0.0, 1.0),
        vec3(0.0, 0.0, -1.0),
        vec3(0.0, -1.0, 0.0),
        vec3(0.0, -1.0, 0.0),
    ];

    let patch_positions = [
        [0.0, 0.0],
        [1.0, 0.0],
        [2.0, 0.0],
        [3.0, 0.0],
        [4.0, 0.0],
        [5.0, 0.0],
    ];

    let (near, far) = (1.0, 250.0);
    // pi / 2 = 90 deg., 1.0 = aspect ratio
    // we use a fov 1% too big to make sure sampling doesn't go between patches
    let proj_data: Matrix4 = perspective(1.0, std::f32::consts::PI / 2.0 * 1.01, near, far).into();

    let model_data: Matrix4 = scale(&Mat4::identity(), &vec3(0.1, 0.1, 0.1)).into();

    let light_pos = make_vec3(&light_data.position);

    view_directions
        .iter()
        .zip(&up_directions)
        .zip(&patch_positions)
        .map(|((dir, up), patch_pos): ((&Vec3, &Vec3), &[f32; 2])| {
            let view_data: Matrix4 = look_at(&light_pos, &(light_pos + dir), up).into();

            // dynamic state for the current cubemap face, represents which part
            // of the patched texture we draw to
            let margin = 0.0;
            let origin = [
                patch_pos[0] * PATCH_DIMS[0] + margin,
                patch_pos[1] * PATCH_DIMS[1] + margin,
            ];
            let dynamic_state = dynamic_state_for_bounds(
                origin,
                [PATCH_DIMS[0] - margin * 2.0, PATCH_DIMS[1] - margin * 2.0],
            );

            Object {
                pipeline_spec: base_object.pipeline_spec.clone(),
                vbuf: base_object.vbuf.clone(),
                ibuf: base_object.ibuf.clone(),
                collection: (
                    (model_data,),
                    (proj_data,),
                    (view_data,),
                    (light_data.clone(),),
                ),
                custom_dynamic_state: Some(dynamic_state),
            }
        })
        .collect()
}

fn dynamic_state_for_bounds(origin: [f32; 2], dimensions: [f32; 2]) -> DynamicState {
    DynamicState {
        line_width: None,
        viewports: Some(vec![Viewport {
            origin,
            dimensions,
            depth_range: 0.0..1.0,
        }]),
        scissors: None,
    }
}

#[allow(dead_code)]
#[derive(Clone)]
struct Material {
    ambient: [f32; 4],
    diffuse: [f32; 4],
    specular: [f32; 4],
    shininess: [f32; 4],
    use_texture: [f32; 4],
}

impl Data for Material {}
