use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::device::Device;

// TODO: maybe define vertex here instead of in system?
use std::fs::File;
use std::io::{BufRead, BufReader, Error};
use std::path::Path;
use std::sync::Arc;

use crate::system::{SimpleVertex, Vertex};
use crate::world::Mesh;

#[rustfmt::skip]
//                                                          0              1                 2                    3                   4                   5                   6                   7
const CUBE_CORNER_POSITIONS: [[f32; 3]; 8] = [ [-1.0, -1.0, -1.0], [ 1.0, -1.0, -1.0], [ 1.0,  1.0, -1.0], [-1.0,  1.0, -1.0], [-1.0, -1.0,  1.0], [ 1.0, -1.0,  1.0], [ 1.0,  1.0,  1.0], [-1.0,  1.0,  1.0], ];

#[rustfmt::skip]
const CUBE_VERTICES: [Vertex; 36] = [ Vertex { position: CUBE_CORNER_POSITIONS[0], normal: [0.0, 0.0, -1.0] }, Vertex { position: CUBE_CORNER_POSITIONS[1], normal: [0.0, 0.0, -1.0] }, Vertex { position: CUBE_CORNER_POSITIONS[3], normal: [0.0, 0.0, -1.0] }, Vertex { position: CUBE_CORNER_POSITIONS[3], normal: [0.0, 0.0, -1.0] }, Vertex { position: CUBE_CORNER_POSITIONS[1], normal: [0.0, 0.0, -1.0] }, Vertex { position: CUBE_CORNER_POSITIONS[2], normal: [0.0, 0.0, -1.0] }, Vertex { position: CUBE_CORNER_POSITIONS[1], normal: [1.0, 0.0, 1.0] }, Vertex { position: CUBE_CORNER_POSITIONS[5], normal: [1.0, 0.0, 1.0] }, Vertex { position: CUBE_CORNER_POSITIONS[2], normal: [1.0, 0.0, 1.0] }, Vertex { position: CUBE_CORNER_POSITIONS[2], normal: [1.0, 0.0, 1.0] }, Vertex { position: CUBE_CORNER_POSITIONS[5], normal: [1.0, 0.0, 1.0] }, Vertex { position: CUBE_CORNER_POSITIONS[6], normal: [1.0, 0.0, 1.0] }, Vertex { position: CUBE_CORNER_POSITIONS[5], normal: [0.0, 0.0, 1.0] }, Vertex { position: CUBE_CORNER_POSITIONS[4], normal: [0.0, 0.0, 1.0] }, Vertex { position: CUBE_CORNER_POSITIONS[6], normal: [0.0, 0.0, 1.0] }, Vertex { position: CUBE_CORNER_POSITIONS[6], normal: [0.0, 0.0, 1.0] }, Vertex { position: CUBE_CORNER_POSITIONS[4], normal: [0.0, 0.0, 1.0] }, Vertex { position: CUBE_CORNER_POSITIONS[7], normal: [0.0, 0.0, 1.0] }, Vertex { position: CUBE_CORNER_POSITIONS[4], normal: [-1.0, 0.0, 0.0] }, Vertex { position: CUBE_CORNER_POSITIONS[0], normal: [-1.0, 0.0, 0.0] }, Vertex { position: CUBE_CORNER_POSITIONS[7], normal: [-1.0, 0.0, 0.0] }, Vertex { position: CUBE_CORNER_POSITIONS[7], normal: [-1.0, 0.0, 0.0] }, Vertex { position: CUBE_CORNER_POSITIONS[0], normal: [-1.0, 0.0, 0.0] }, Vertex { position: CUBE_CORNER_POSITIONS[3], normal: [-1.0, 0.0, 0.0] }, Vertex { position: CUBE_CORNER_POSITIONS[3], normal: [0.0, 1.0, 1.0] }, Vertex { position: CUBE_CORNER_POSITIONS[2], normal: [0.0, 1.0, 1.0] }, Vertex { position: CUBE_CORNER_POSITIONS[7], normal: [0.0, 1.0, 1.0] }, Vertex { position: CUBE_CORNER_POSITIONS[7], normal: [0.0, 1.0, 1.0] }, Vertex { position: CUBE_CORNER_POSITIONS[2], normal: [0.0, 1.0, 1.0] }, Vertex { position: CUBE_CORNER_POSITIONS[6], normal: [0.0, 1.0, 1.0] }, Vertex { position: CUBE_CORNER_POSITIONS[4], normal: [0.0, -1.0, 0.0] }, Vertex { position: CUBE_CORNER_POSITIONS[5], normal: [0.0, -1.0, 0.0] }, Vertex { position: CUBE_CORNER_POSITIONS[0], normal: [0.0, -1.0, 0.0] }, Vertex { position: CUBE_CORNER_POSITIONS[0], normal: [0.0, -1.0, 0.0] }, Vertex { position: CUBE_CORNER_POSITIONS[5], normal: [0.0, -1.0, 0.0] }, Vertex { position: CUBE_CORNER_POSITIONS[1], normal: [0.0, -1.0, 0.0] }, ];

#[rustfmt::skip]
// normals are all 1 because they don't matter with lines
// const CUBE_EDGE_VERTICES: [Vertex; 24] = [0, 4, 0, 1, 0, 3];
const CUBE_EDGE_VERTICES: [Vertex; 24] = [
    Vertex { position: CUBE_CORNER_POSITIONS[0], normal: [0.0, 0.0, -1.0] },
    Vertex { position: CUBE_CORNER_POSITIONS[1], normal: [0.0, 0.0, -1.0] },
    Vertex { position: CUBE_CORNER_POSITIONS[0], normal: [0.0, 0.0, -1.0] },
    Vertex { position: CUBE_CORNER_POSITIONS[4], normal: [0.0, 0.0, -1.0] },
    Vertex { position: CUBE_CORNER_POSITIONS[4], normal: [0.0, 0.0, -1.0] },
    Vertex { position: CUBE_CORNER_POSITIONS[5], normal: [0.0, 0.0, -1.0] },
    Vertex { position: CUBE_CORNER_POSITIONS[5], normal: [0.0, 0.0, -1.0] },
    Vertex { position: CUBE_CORNER_POSITIONS[1], normal: [0.0, 0.0, -1.0] },
    Vertex { position: CUBE_CORNER_POSITIONS[3], normal: [0.0, 0.0, -1.0] },
    Vertex { position: CUBE_CORNER_POSITIONS[7], normal: [0.0, 0.0, -1.0] },
    Vertex { position: CUBE_CORNER_POSITIONS[7], normal: [0.0, 0.0, -1.0] },
    Vertex { position: CUBE_CORNER_POSITIONS[6], normal: [0.0, 0.0, -1.0] },
    Vertex { position: CUBE_CORNER_POSITIONS[6], normal: [0.0, 0.0, -1.0] },
    Vertex { position: CUBE_CORNER_POSITIONS[2], normal: [0.0, 0.0, -1.0] },
    Vertex { position: CUBE_CORNER_POSITIONS[2], normal: [0.0, 0.0, -1.0] },
    Vertex { position: CUBE_CORNER_POSITIONS[3], normal: [0.0, 0.0, -1.0] },
    Vertex { position: CUBE_CORNER_POSITIONS[0], normal: [0.0, 0.0, -1.0] },
    Vertex { position: CUBE_CORNER_POSITIONS[3], normal: [0.0, 0.0, -1.0] },
    Vertex { position: CUBE_CORNER_POSITIONS[1], normal: [0.0, 0.0, -1.0] },
    Vertex { position: CUBE_CORNER_POSITIONS[2], normal: [0.0, 0.0, -1.0] },
    Vertex { position: CUBE_CORNER_POSITIONS[5], normal: [0.0, 0.0, -1.0] },
    Vertex { position: CUBE_CORNER_POSITIONS[6], normal: [0.0, 0.0, -1.0] },
    Vertex { position: CUBE_CORNER_POSITIONS[4], normal: [0.0, 0.0, -1.0] },
    Vertex { position: CUBE_CORNER_POSITIONS[7], normal: [0.0, 0.0, -1.0] },
];

pub fn load_obj(path: &Path) -> Result<Mesh, Error> {
    let file = BufReader::new(File::open(&path)?);

    let mut vertices = vec![];
    let mut normals = vec![];
    let mut faces = vec![];

    for line in file.lines() {
        let line = line.unwrap();
        // each line is either a vertex:
        // "v 0.72 -0.44 0.52"
        // a normal:
        // "vn 0.10 -0.94 0.31"
        // or a face:
        // "f 1//1 14//1 13//1"
        if line.starts_with("v ") {
            let pieces: Vec<_> = line.split_whitespace().collect();
            let x: f32 = pieces[1].parse().expect("Corrupt OBJ file");
            let y: f32 = pieces[2].parse().expect("Corrupt OBJ file");
            let z: f32 = pieces[3].parse().expect("Corrupt OBJ file");
            vertices.push([x, y, z]);
        } else if line.starts_with("vn ") {
            let pieces: Vec<_> = line.split_whitespace().collect();
            if let Ok(_) = pieces[1].parse::<f32>() {
            } else {
                dbg![&pieces];
            }
            let x: f32 = pieces[1].parse().unwrap_or(0.577);
            let y: f32 = pieces[2].parse().unwrap_or(0.577);
            let z: f32 = pieces[3].parse().unwrap_or(0.577);
            normals.push([x, y * 1.0, z]);
        } else if line.starts_with("f ") {
            let pieces: Vec<_> = line.split_whitespace().collect();
            let piece1 = pieces[1].split("/").collect::<Vec<_>>();
            let piece2 = pieces[2].split("/").collect::<Vec<_>>();
            let piece3 = pieces[3].split("/").collect::<Vec<_>>();
            let v1: u32 = piece1[0].parse().unwrap();
            let v2: u32 = piece2[0].parse().unwrap();
            let v3: u32 = piece3[0].parse().unwrap();
            let n1: u32 = piece1[2].parse().unwrap();
            let n2: u32 = piece2[2].parse().unwrap();
            let n3: u32 = piece3[2].parse().unwrap();

            faces.push((v1, v2, v3, n1, n2, n3));
        }
    }

    println!(
        "loaded obj: {} verts, {} normals, {} faces",
        vertices.len(),
        normals.len(),
        faces.len()
    );

    // TODO: switch to tobj because i don't want to write shit like this
    let mut vertices_with_normal_indices = vec![0; vertices.len()];
    for face in faces.iter() {
        let (v1, v2, v3, n1, n2, n3) = face;

        for (v_idx, n_idx) in [(v1, n1), (v2, n2), (v3, n3)].iter() {
            vertices_with_normal_indices[**v_idx as usize - 1] = **n_idx as usize - 1;
        }
    }

    let final_vertices: Vec<Vertex> = vertices
        .iter()
        .enumerate()
        .map(|(idx, v)| Vertex {
            position: *v,
            normal: normals[vertices_with_normal_indices[idx]],
        })
        .collect();

    let final_indices = faces
        .iter()
        .flat_map(|(v1, v2, v3, _, _, _)| vec![*v1 - 1, *v2 - 1, *v3 - 1])
        .collect();

    Ok(Mesh {
        vertices: Box::new(final_vertices),
        indices: final_indices,
    })
}

// TODO: get rid of center_position and radius because you can do the same with model matrices
pub fn create_vertices_for_cube(center_position: [f32; 3], radius: f32) -> Mesh {
    let vertices: Vec<Vertex> = CUBE_VERTICES
        .iter()
        .map(|vertex| Vertex {
            position: [
                vertex.position[0] * radius + center_position[0],
                vertex.position[1] * radius + center_position[1],
                vertex.position[2] * radius + center_position[2],
            ],
            normal: vertex.normal,
        })
        .collect();
    let indices: Vec<u32> = (0..36).collect();

    Mesh {
        vertices: Box::new(vertices),
        indices,
    }
}

pub fn create_vertices_for_cube_edges(center_position: [f32; 3], radius: f32) -> Mesh {
    let vertices: Vec<_> = CUBE_EDGE_VERTICES
        .iter()
        .map(|vertex| Vertex {
            position: [
                vertex.position[0] * radius + center_position[0],
                vertex.position[1] * radius + center_position[1],
                vertex.position[2] * radius + center_position[2],
            ],
            normal: vertex.normal,
        })
        .collect();

    let indices = (0..vertices.len() as u32).collect();

    Mesh {
        vertices: Box::new(vertices),
        indices,
    }
}

pub fn create_buffers_for_screen_square(
    device: Arc<Device>,
) -> (
    Arc<CpuAccessibleBuffer<[SimpleVertex]>>,
    Arc<CpuAccessibleBuffer<[u32]>>,
) {
    let simple_vbuf = {
        CpuAccessibleBuffer::from_iter(
            device.clone(),
            BufferUsage::all(),
            [
                SimpleVertex {
                    position: [-1.0, -1.0],
                },
                SimpleVertex {
                    position: [-1.0, 1.0],
                },
                SimpleVertex {
                    position: [1.0, -1.0],
                },
                SimpleVertex {
                    position: [1.0, 1.0],
                },
            ]
            .iter()
            .cloned(),
        )
        .unwrap()
    };

    let simple_ibuf = {
        CpuAccessibleBuffer::from_iter(
            device.clone(),
            BufferUsage::all(),
            [0, 1, 2, 3].iter().cloned(),
        )
        .unwrap()
    };

    (simple_vbuf, simple_ibuf)
}
