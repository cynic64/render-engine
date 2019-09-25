// TODO: maybe define vertex here instead of in system?
use crate::system::Vertex;

use std::fs::File;
use std::io::{BufRead, BufReader, Error};
use std::path::Path;

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

#[rustfmt::skip]
const SPHERE_VERTICES: [Vertex; 240] = [ Vertex { position: [0.000000, -1.000000, 0.000000], normal: [0.1024, -0.9435, 0.3151]}, Vertex { position: [0.425323, -0.850654, 0.309011], normal: [0.1024, -0.9435, 0.3151]}, Vertex { position: [-0.162456, -0.850654, 0.499995], normal: [0.1024, -0.9435, 0.3151]}, Vertex { position: [0.723607, -0.447220, 0.525725], normal: [0.7002, -0.6617, 0.2680]}, Vertex { position: [0.425323, -0.850654, 0.309011], normal: [0.7002, -0.6617, 0.2680]}, Vertex { position: [0.850648, -0.525736, 0.000000], normal: [0.7002, -0.6617, 0.2680]}, Vertex { position: [0.000000, -1.000000, 0.000000], normal: [-0.2680, -0.9435, 0.1947]}, Vertex { position: [-0.162456, -0.850654, 0.499995], normal: [-0.2680, -0.9435, 0.1947]}, Vertex { position: [-0.525730, -0.850652, 0.000000], normal: [-0.2680, -0.9435, 0.1947]}, Vertex { position: [0.000000, -1.000000, 0.000000], normal: [-0.2680, -0.9435, -0.1947]}, Vertex { position: [-0.525730, -0.850652, 0.000000], normal: [-0.2680, -0.9435, -0.1947]}, Vertex { position: [-0.162456, -0.850654, -0.499995], normal: [-0.2680, -0.9435, -0.1947]}, Vertex { position: [0.000000, -1.000000, 0.000000], normal: [0.1024, -0.9435, -0.3151]}, Vertex { position: [-0.162456, -0.850654, -0.499995], normal: [0.1024, -0.9435, -0.3151]}, Vertex { position: [0.425323, -0.850654, -0.309011], normal: [0.1024, -0.9435, -0.3151]}, Vertex { position: [0.723607, -0.447220, 0.525725], normal: [0.9050, -0.3304, 0.2680]}, Vertex { position: [0.850648, -0.525736, 0.000000], normal: [0.9050, -0.3304, 0.2680]}, Vertex { position: [0.951058, 0.000000, 0.309013], normal: [0.9050, -0.3304, 0.2680]}, Vertex { position: [-0.276388, -0.447220, 0.850649], normal: [0.0247, -0.3304, 0.9435]}, Vertex { position: [0.262869, -0.525738, 0.809012], normal: [0.0247, -0.3304, 0.9435]}, Vertex { position: [0.000000, 0.000000, 1.000000], normal: [0.0247, -0.3304, 0.9435]}, Vertex { position: [-0.894426, -0.447216, 0.000000], normal: [-0.8897, -0.3304, 0.3151]}, Vertex { position: [-0.688189, -0.525736, 0.499997], normal: [-0.8897, -0.3304, 0.3151]}, Vertex { position: [-0.951058, 0.000000, 0.309013], normal: [-0.8897, -0.3304, 0.3151]}, Vertex { position: [-0.276388, -0.447220, -0.850649], normal: [-0.5746, -0.3304, -0.7488]}, Vertex { position: [-0.688189, -0.525736, -0.499997], normal: [-0.5746, -0.3304, -0.7488]}, Vertex { position: [-0.587786, 0.000000, -0.809017], normal: [-0.5746, -0.3304, -0.7488]}, Vertex { position: [0.723607, -0.447220, -0.525725], normal: [0.5346, -0.3304, -0.7779]}, Vertex { position: [0.262869, -0.525738, -0.809012], normal: [0.5346, -0.3304, -0.7779]}, Vertex { position: [0.587786, 0.000000, -0.809017], normal: [0.5346, -0.3304, -0.7779]}, Vertex { position: [0.723607, -0.447220, 0.525725], normal: [0.8026, -0.1256, 0.5831]}, Vertex { position: [0.951058, 0.000000, 0.309013], normal: [0.8026, -0.1256, 0.5831]}, Vertex { position: [0.587786, 0.000000, 0.809017], normal: [0.8026, -0.1256, 0.5831]}, Vertex { position: [-0.276388, -0.447220, 0.850649], normal: [-0.3066, -0.1256, 0.9435]}, Vertex { position: [0.000000, 0.000000, 1.000000], normal: [-0.3066, -0.1256, 0.9435]}, Vertex { position: [-0.587786, 0.000000, 0.809017], normal: [-0.3066, -0.1256, 0.9435]}, Vertex { position: [-0.894426, -0.447216, 0.000000], normal: [-0.9921, -0.1256, 0.0000]}, Vertex { position: [-0.951058, 0.000000, 0.309013], normal: [-0.9921, -0.1256, 0.0000]}, Vertex { position: [-0.951058, 0.000000, -0.309013], normal: [-0.9921, -0.1256, 0.0000]}, Vertex { position: [-0.276388, -0.447220, -0.850649], normal: [-0.3066, -0.1256, -0.9435]}, Vertex { position: [-0.587786, 0.000000, -0.809017], normal: [-0.3066, -0.1256, -0.9435]}, Vertex { position: [0.000000, 0.000000, -1.000000], normal: [-0.3066, -0.1256, -0.9435]}, Vertex { position: [0.723607, -0.447220, -0.525725], normal: [0.8026, -0.1256, -0.5831]}, Vertex { position: [0.587786, 0.000000, -0.809017], normal: [0.8026, -0.1256, -0.5831]}, Vertex { position: [0.951058, 0.000000, -0.309013], normal: [0.8026, -0.1256, -0.5831]}, Vertex { position: [0.276388, 0.447220, 0.850649], normal: [0.4089, 0.6617, 0.6284]}, Vertex { position: [0.688189, 0.525736, 0.499997], normal: [0.4089, 0.6617, 0.6284]}, Vertex { position: [0.162456, 0.850654, 0.499995], normal: [0.4089, 0.6617, 0.6284]}, Vertex { position: [-0.723607, 0.447220, 0.525725], normal: [-0.4713, 0.6617, 0.5831]}, Vertex { position: [-0.262869, 0.525738, 0.809012], normal: [-0.4713, 0.6617, 0.5831]}, Vertex { position: [-0.425323, 0.850654, 0.309011], normal: [-0.4713, 0.6617, 0.5831]}, Vertex { position: [-0.723607, 0.447220, -0.525725], normal: [-0.7002, 0.6617, -0.2680]}, Vertex { position: [-0.850648, 0.525736, 0.000000], normal: [-0.7002, 0.6617, -0.2680]}, Vertex { position: [-0.425323, 0.850654, -0.309011], normal: [-0.7002, 0.6617, -0.2680]}, Vertex { position: [0.276388, 0.447220, -0.850649], normal: [0.0385, 0.6617, -0.7488]}, Vertex { position: [-0.262869, 0.525738, -0.809012], normal: [0.0385, 0.6617, -0.7488]}, Vertex { position: [0.162456, 0.850654, -0.499995], normal: [0.0385, 0.6617, -0.7488]}, Vertex { position: [0.894426, 0.447216, 0.000000], normal: [0.7240, 0.6617, -0.1947]}, Vertex { position: [0.688189, 0.525736, -0.499997], normal: [0.7240, 0.6617, -0.1947]}, Vertex { position: [0.525730, 0.850652, 0.000000], normal: [0.7240, 0.6617, -0.1947]}, Vertex { position: [0.525730, 0.850652, 0.000000], normal: [0.2680, 0.9435, -0.1947]}, Vertex { position: [0.162456, 0.850654, -0.499995], normal: [0.2680, 0.9435, -0.1947]}, Vertex { position: [0.000000, 1.000000, 0.000000], normal: [0.2680, 0.9435, -0.1947]}, Vertex { position: [0.525730, 0.850652, 0.000000], normal: [0.4911, 0.7947, -0.3568]}, Vertex { position: [0.688189, 0.525736, -0.499997], normal: [0.4911, 0.7947, -0.3568]}, Vertex { position: [0.162456, 0.850654, -0.499995], normal: [0.4911, 0.7947, -0.3568]}, Vertex { position: [0.688189, 0.525736, -0.499997], normal: [0.4089, 0.6617, -0.6284]}, Vertex { position: [0.276388, 0.447220, -0.850649], normal: [0.4089, 0.6617, -0.6284]}, Vertex { position: [0.162456, 0.850654, -0.499995], normal: [0.4089, 0.6617, -0.6284]}, Vertex { position: [0.162456, 0.850654, -0.499995], normal: [-0.1024, 0.9435, -0.3151]}, Vertex { position: [-0.425323, 0.850654, -0.309011], normal: [-0.1024, 0.9435, -0.3151]}, Vertex { position: [0.000000, 1.000000, 0.000000], normal: [-0.1024, 0.9435, -0.3151]}, Vertex { position: [0.162456, 0.850654, -0.499995], normal: [-0.1876, 0.7947, -0.5773]}, Vertex { position: [-0.262869, 0.525738, -0.809012], normal: [-0.1876, 0.7947, -0.5773]}, Vertex { position: [-0.425323, 0.850654, -0.309011], normal: [-0.1876, 0.7947, -0.5773]}, Vertex { position: [-0.262869, 0.525738, -0.809012], normal: [-0.4713, 0.6617, -0.5831]}, Vertex { position: [-0.723607, 0.447220, -0.525725], normal: [-0.4713, 0.6617, -0.5831]}, Vertex { position: [-0.425323, 0.850654, -0.309011], normal: [-0.4713, 0.6617, -0.5831]}, Vertex { position: [-0.425323, 0.850654, -0.309011], normal: [-0.3313, 0.9435, 0.0000]}, Vertex { position: [-0.425323, 0.850654, 0.309011], normal: [-0.3313, 0.9435, 0.0000]}, Vertex { position: [0.000000, 1.000000, 0.000000], normal: [-0.3313, 0.9435, 0.0000]}, Vertex { position: [-0.425323, 0.850654, -0.309011], normal: [-0.6071, 0.7947, 0.0000]}, Vertex { position: [-0.850648, 0.525736, 0.000000], normal: [-0.6071, 0.7947, 0.0000]}, Vertex { position: [-0.425323, 0.850654, 0.309011], normal: [-0.6071, 0.7947, 0.0000]}, Vertex { position: [-0.850648, 0.525736, 0.000000], normal: [-0.7002, 0.6617, 0.2680]}, Vertex { position: [-0.723607, 0.447220, 0.525725], normal: [-0.7002, 0.6617, 0.2680]}, Vertex { position: [-0.425323, 0.850654, 0.309011], normal: [-0.7002, 0.6617, 0.2680]}, Vertex { position: [-0.425323, 0.850654, 0.309011], normal: [-0.1024, 0.9435, 0.3151]}, Vertex { position: [0.162456, 0.850654, 0.499995], normal: [-0.1024, 0.9435, 0.3151]}, Vertex { position: [0.000000, 1.000000, 0.000000], normal: [-0.1024, 0.9435, 0.3151]}, Vertex { position: [-0.425323, 0.850654, 0.309011], normal: [-0.1876, 0.7947, 0.5773]}, Vertex { position: [-0.262869, 0.525738, 0.809012], normal: [-0.1876, 0.7947, 0.5773]}, Vertex { position: [0.162456, 0.850654, 0.499995], normal: [-0.1876, 0.7947, 0.5773]}, Vertex { position: [-0.262869, 0.525738, 0.809012], normal: [0.0385, 0.6617, 0.7488]}, Vertex { position: [0.276388, 0.447220, 0.850649], normal: [0.0385, 0.6617, 0.7488]}, Vertex { position: [0.162456, 0.850654, 0.499995], normal: [0.0385, 0.6617, 0.7488]}, Vertex { position: [0.162456, 0.850654, 0.499995], normal: [0.2680, 0.9435, 0.1947]}, Vertex { position: [0.525730, 0.850652, 0.000000], normal: [0.2680, 0.9435, 0.1947]}, Vertex { position: [0.000000, 1.000000, 0.000000], normal: [0.2680, 0.9435, 0.1947]}, Vertex { position: [0.162456, 0.850654, 0.499995], normal: [0.4911, 0.7947, 0.3568]}, Vertex { position: [0.688189, 0.525736, 0.499997], normal: [0.4911, 0.7947, 0.3568]}, Vertex { position: [0.525730, 0.850652, 0.000000], normal: [0.4911, 0.7947, 0.3568]}, Vertex { position: [0.688189, 0.525736, 0.499997], normal: [0.7240, 0.6617, 0.1947]}, Vertex { position: [0.894426, 0.447216, 0.000000], normal: [0.7240, 0.6617, 0.1947]}, Vertex { position: [0.525730, 0.850652, 0.000000], normal: [0.7240, 0.6617, 0.1947]}, Vertex { position: [0.951058, 0.000000, -0.309013], normal: [0.8897, 0.3304, -0.3151]}, Vertex { position: [0.688189, 0.525736, -0.499997], normal: [0.8897, 0.3304, -0.3151]}, Vertex { position: [0.894426, 0.447216, 0.000000], normal: [0.8897, 0.3304, -0.3151]}, Vertex { position: [0.951058, 0.000000, -0.309013], normal: [0.7947, 0.1876, -0.5773]}, Vertex { position: [0.587786, 0.000000, -0.809017], normal: [0.7947, 0.1876, -0.5773]}, Vertex { position: [0.688189, 0.525736, -0.499997], normal: [0.7947, 0.1876, -0.5773]}, Vertex { position: [0.587786, 0.000000, -0.809017], normal: [0.5746, 0.3304, -0.7488]}, Vertex { position: [0.276388, 0.447220, -0.850649], normal: [0.5746, 0.3304, -0.7488]}, Vertex { position: [0.688189, 0.525736, -0.499997], normal: [0.5746, 0.3304, -0.7488]}, Vertex { position: [0.000000, 0.000000, -1.000000], normal: [-0.0247, 0.3304, -0.9435]}, Vertex { position: [-0.262869, 0.525738, -0.809012], normal: [-0.0247, 0.3304, -0.9435]}, Vertex { position: [0.276388, 0.447220, -0.850649], normal: [-0.0247, 0.3304, -0.9435]}, Vertex { position: [0.000000, 0.000000, -1.000000], normal: [-0.3035, 0.1876, -0.9342]}, Vertex { position: [-0.587786, 0.000000, -0.809017], normal: [-0.3035, 0.1876, -0.9342]}, Vertex { position: [-0.262869, 0.525738, -0.809012], normal: [-0.3035, 0.1876, -0.9342]}, Vertex { position: [-0.587786, 0.000000, -0.809017], normal: [-0.5346, 0.3304, -0.7779]}, Vertex { position: [-0.723607, 0.447220, -0.525725], normal: [-0.5346, 0.3304, -0.7779]}, Vertex { position: [-0.262869, 0.525738, -0.809012], normal: [-0.5346, 0.3304, -0.7779]}, Vertex { position: [-0.951058, 0.000000, -0.309013], normal: [-0.9050, 0.3304, -0.2680]}, Vertex { position: [-0.850648, 0.525736, 0.000000], normal: [-0.9050, 0.3304, -0.2680]}, Vertex { position: [-0.723607, 0.447220, -0.525725], normal: [-0.9050, 0.3304, -0.2680]}, Vertex { position: [-0.951058, 0.000000, -0.309013], normal: [-0.9822, 0.1876, 0.0000]}, Vertex { position: [-0.951058, 0.000000, 0.309013], normal: [-0.9822, 0.1876, 0.0000]}, Vertex { position: [-0.850648, 0.525736, 0.000000], normal: [-0.9822, 0.1876, 0.0000]}, Vertex { position: [-0.951058, 0.000000, 0.309013], normal: [-0.9050, 0.3304, 0.2680]}, Vertex { position: [-0.723607, 0.447220, 0.525725], normal: [-0.9050, 0.3304, 0.2680]}, Vertex { position: [-0.850648, 0.525736, 0.000000], normal: [-0.9050, 0.3304, 0.2680]}, Vertex { position: [-0.587786, 0.000000, 0.809017], normal: [-0.5346, 0.3304, 0.7779]}, Vertex { position: [-0.262869, 0.525738, 0.809012], normal: [-0.5346, 0.3304, 0.7779]}, Vertex { position: [-0.723607, 0.447220, 0.525725], normal: [-0.5346, 0.3304, 0.7779]}, Vertex { position: [-0.587786, 0.000000, 0.809017], normal: [-0.3035, 0.1876, 0.9342]}, Vertex { position: [0.000000, 0.000000, 1.000000], normal: [-0.3035, 0.1876, 0.9342]}, Vertex { position: [-0.262869, 0.525738, 0.809012], normal: [-0.3035, 0.1876, 0.9342]}, Vertex { position: [0.000000, 0.000000, 1.000000], normal: [-0.0247, 0.3304, 0.9435]}, Vertex { position: [0.276388, 0.447220, 0.850649], normal: [-0.0247, 0.3304, 0.9435]}, Vertex { position: [-0.262869, 0.525738, 0.809012], normal: [-0.0247, 0.3304, 0.9435]}, Vertex { position: [0.587786, 0.000000, 0.809017], normal: [0.5746, 0.3304, 0.7488]}, Vertex { position: [0.688189, 0.525736, 0.499997], normal: [0.5746, 0.3304, 0.7488]}, Vertex { position: [0.276388, 0.447220, 0.850649], normal: [0.5746, 0.3304, 0.7488]}, Vertex { position: [0.587786, 0.000000, 0.809017], normal: [0.7947, 0.1876, 0.5773]}, Vertex { position: [0.951058, 0.000000, 0.309013], normal: [0.7947, 0.1876, 0.5773]}, Vertex { position: [0.688189, 0.525736, 0.499997], normal: [0.7947, 0.1876, 0.5773]}, Vertex { position: [0.951058, 0.000000, 0.309013], normal: [0.8897, 0.3304, 0.3151]}, Vertex { position: [0.894426, 0.447216, 0.000000], normal: [0.8897, 0.3304, 0.3151]}, Vertex { position: [0.688189, 0.525736, 0.499997], normal: [0.8897, 0.3304, 0.3151]}, Vertex { position: [0.587786, 0.000000, -0.809017], normal: [0.3066, 0.1256, -0.9435]}, Vertex { position: [0.000000, 0.000000, -1.000000], normal: [0.3066, 0.1256, -0.9435]}, Vertex { position: [0.276388, 0.447220, -0.850649], normal: [0.3066, 0.1256, -0.9435]}, Vertex { position: [0.587786, 0.000000, -0.809017], normal: [0.3035, -0.1876, -0.9342]}, Vertex { position: [0.262869, -0.525738, -0.809012], normal: [0.3035, -0.1876, -0.9342]}, Vertex { position: [0.000000, 0.000000, -1.000000], normal: [0.3035, -0.1876, -0.9342]}, Vertex { position: [0.262869, -0.525738, -0.809012], normal: [0.0247, -0.3304, -0.9435]}, Vertex { position: [-0.276388, -0.447220, -0.850649], normal: [0.0247, -0.3304, -0.9435]}, Vertex { position: [0.000000, 0.000000, -1.000000], normal: [0.0247, -0.3304, -0.9435]}, Vertex { position: [-0.587786, 0.000000, -0.809017], normal: [-0.8026, 0.1256, -0.5831]}, Vertex { position: [-0.951058, 0.000000, -0.309013], normal: [-0.8026, 0.1256, -0.5831]}, Vertex { position: [-0.723607, 0.447220, -0.525725], normal: [-0.8026, 0.1256, -0.5831]}, Vertex { position: [-0.587786, 0.000000, -0.809017], normal: [-0.7947, -0.1876, -0.5773]}, Vertex { position: [-0.688189, -0.525736, -0.499997], normal: [-0.7947, -0.1876, -0.5773]}, Vertex { position: [-0.951058, 0.000000, -0.309013], normal: [-0.7947, -0.1876, -0.5773]}, Vertex { position: [-0.688189, -0.525736, -0.499997], normal: [-0.8897, -0.3304, -0.3151]}, Vertex { position: [-0.894426, -0.447216, 0.000000], normal: [-0.8897, -0.3304, -0.3151]}, Vertex { position: [-0.951058, 0.000000, -0.309013], normal: [-0.8897, -0.3304, -0.3151]}, Vertex { position: [-0.951058, 0.000000, 0.309013], normal: [-0.8026, 0.1256, 0.5831]}, Vertex { position: [-0.587786, 0.000000, 0.809017], normal: [-0.8026, 0.1256, 0.5831]}, Vertex { position: [-0.723607, 0.447220, 0.525725], normal: [-0.8026, 0.1256, 0.5831]}, Vertex { position: [-0.951058, 0.000000, 0.309013], normal: [-0.7947, -0.1876, 0.5773]}, Vertex { position: [-0.688189, -0.525736, 0.499997], normal: [-0.7947, -0.1876, 0.5773]}, Vertex { position: [-0.587786, 0.000000, 0.809017], normal: [-0.7947, -0.1876, 0.5773]}, Vertex { position: [-0.688189, -0.525736, 0.499997], normal: [-0.5746, -0.3304, 0.7488]}, Vertex { position: [-0.276388, -0.447220, 0.850649], normal: [-0.5746, -0.3304, 0.7488]}, Vertex { position: [-0.587786, 0.000000, 0.809017], normal: [-0.5746, -0.3304, 0.7488]}, Vertex { position: [0.000000, 0.000000, 1.000000], normal: [0.3066, 0.1256, 0.9435]}, Vertex { position: [0.587786, 0.000000, 0.809017], normal: [0.3066, 0.1256, 0.9435]}, Vertex { position: [0.276388, 0.447220, 0.850649], normal: [0.3066, 0.1256, 0.9435]}, Vertex { position: [0.000000, 0.000000, 1.000000], normal: [0.3035, -0.1876, 0.9342]}, Vertex { position: [0.262869, -0.525738, 0.809012], normal: [0.3035, -0.1876, 0.9342]}, Vertex { position: [0.587786, 0.000000, 0.809017], normal: [0.3035, -0.1876, 0.9342]}, Vertex { position: [0.262869, -0.525738, 0.809012], normal: [0.5346, -0.3304, 0.7779]}, Vertex { position: [0.723607, -0.447220, 0.525725], normal: [0.5346, -0.3304, 0.7779]}, Vertex { position: [0.587786, 0.000000, 0.809017], normal: [0.5346, -0.3304, 0.7779]}, Vertex { position: [0.951058, 0.000000, 0.309013], normal: [0.9921, 0.1256, 0.0000]}, Vertex { position: [0.951058, 0.000000, -0.309013], normal: [0.9921, 0.1256, 0.0000]}, Vertex { position: [0.894426, 0.447216, 0.000000], normal: [0.9921, 0.1256, 0.0000]}, Vertex { position: [0.951058, 0.000000, 0.309013], normal: [0.9822, -0.1876, 0.0000]}, Vertex { position: [0.850648, -0.525736, 0.000000], normal: [0.9822, -0.1876, 0.0000]}, Vertex { position: [0.951058, 0.000000, -0.309013], normal: [0.9822, -0.1876, 0.0000]}, Vertex { position: [0.850648, -0.525736, 0.000000], normal: [0.9050, -0.3304, -0.2680]}, Vertex { position: [0.723607, -0.447220, -0.525725], normal: [0.9050, -0.3304, -0.2680]}, Vertex { position: [0.951058, 0.000000, -0.309013], normal: [0.9050, -0.3304, -0.2680]}, Vertex { position: [0.425323, -0.850654, -0.309011], normal: [0.4713, -0.6617, -0.5831]}, Vertex { position: [0.262869, -0.525738, -0.809012], normal: [0.4713, -0.6617, -0.5831]}, Vertex { position: [0.723607, -0.447220, -0.525725], normal: [0.4713, -0.6617, -0.5831]}, Vertex { position: [0.425323, -0.850654, -0.309011], normal: [0.1876, -0.7947, -0.5773]}, Vertex { position: [-0.162456, -0.850654, -0.499995], normal: [0.1876, -0.7947, -0.5773]}, Vertex { position: [0.262869, -0.525738, -0.809012], normal: [0.1876, -0.7947, -0.5773]}, Vertex { position: [-0.162456, -0.850654, -0.499995], normal: [-0.0385, -0.6617, -0.7488]}, Vertex { position: [-0.276388, -0.447220, -0.850649], normal: [-0.0385, -0.6617, -0.7488]}, Vertex { position: [0.262869, -0.525738, -0.809012], normal: [-0.0385, -0.6617, -0.7488]}, Vertex { position: [-0.162456, -0.850654, -0.499995], normal: [-0.4089, -0.6617, -0.6284]}, Vertex { position: [-0.688189, -0.525736, -0.499997], normal: [-0.4089, -0.6617, -0.6284]}, Vertex { position: [-0.276388, -0.447220, -0.850649], normal: [-0.4089, -0.6617, -0.6284]}, Vertex { position: [-0.162456, -0.850654, -0.499995], normal: [-0.4911, -0.7947, -0.3568]}, Vertex { position: [-0.525730, -0.850652, 0.000000], normal: [-0.4911, -0.7947, -0.3568]}, Vertex { position: [-0.688189, -0.525736, -0.499997], normal: [-0.4911, -0.7947, -0.3568]}, Vertex { position: [-0.525730, -0.850652, 0.000000], normal: [-0.7240, -0.6617, -0.1947]}, Vertex { position: [-0.894426, -0.447216, 0.000000], normal: [-0.7240, -0.6617, -0.1947]}, Vertex { position: [-0.688189, -0.525736, -0.499997], normal: [-0.7240, -0.6617, -0.1947]}, Vertex { position: [-0.525730, -0.850652, 0.000000], normal: [-0.7240, -0.6617, 0.1947]}, Vertex { position: [-0.688189, -0.525736, 0.499997], normal: [-0.7240, -0.6617, 0.1947]}, Vertex { position: [-0.894426, -0.447216, 0.000000], normal: [-0.7240, -0.6617, 0.1947]}, Vertex { position: [-0.525730, -0.850652, 0.000000], normal: [-0.4911, -0.7947, 0.3568]}, Vertex { position: [-0.162456, -0.850654, 0.499995], normal: [-0.4911, -0.7947, 0.3568]}, Vertex { position: [-0.688189, -0.525736, 0.499997], normal: [-0.4911, -0.7947, 0.3568]}, Vertex { position: [-0.162456, -0.850654, 0.499995], normal: [-0.4089, -0.6617, 0.6284]}, Vertex { position: [-0.276388, -0.447220, 0.850649], normal: [-0.4089, -0.6617, 0.6284]}, Vertex { position: [-0.688189, -0.525736, 0.499997], normal: [-0.4089, -0.6617, 0.6284]}, Vertex { position: [0.850648, -0.525736, 0.000000], normal: [0.7002, -0.6617, -0.2680]}, Vertex { position: [0.425323, -0.850654, -0.309011], normal: [0.7002, -0.6617, -0.2680]}, Vertex { position: [0.723607, -0.447220, -0.525725], normal: [0.7002, -0.6617, -0.2680]}, Vertex { position: [0.850648, -0.525736, 0.000000], normal: [0.6071, -0.7947, 0.0000]}, Vertex { position: [0.425323, -0.850654, 0.309011], normal: [0.6071, -0.7947, 0.0000]}, Vertex { position: [0.425323, -0.850654, -0.309011], normal: [0.6071, -0.7947, 0.0000]}, Vertex { position: [0.425323, -0.850654, 0.309011], normal: [0.3313, -0.9435, 0.0000]}, Vertex { position: [0.000000, -1.000000, 0.000000], normal: [0.3313, -0.9435, 0.0000]}, Vertex { position: [0.425323, -0.850654, -0.309011], normal: [0.3313, -0.9435, 0.0000]}, Vertex { position: [-0.162456, -0.850654, 0.499995], normal: [-0.0385, -0.6617, 0.7488]}, Vertex { position: [0.262869, -0.525738, 0.809012], normal: [-0.0385, -0.6617, 0.7488]}, Vertex { position: [-0.276388, -0.447220, 0.850649], normal: [-0.0385, -0.6617, 0.7488]}, Vertex { position: [-0.162456, -0.850654, 0.499995], normal: [0.1876, -0.7947, 0.5773]}, Vertex { position: [0.425323, -0.850654, 0.309011], normal: [0.1876, -0.7947, 0.5773]}, Vertex { position: [0.262869, -0.525738, 0.809012], normal: [0.1876, -0.7947, 0.5773]}, Vertex { position: [0.425323, -0.850654, 0.309011], normal: [0.4713, -0.6617, 0.5831]}, Vertex { position: [0.723607, -0.447220, 0.525725], normal: [0.4713, -0.6617, 0.5831]}, Vertex { position: [0.262869, -0.525738, 0.809012], normal: [0.4713, -0.6617, 0.5831]}, ];

pub fn load_obj(path: &Path) -> Result<Vec<Vertex>, Error> {
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
            vertices.push([x, y * -1.0, z]);
        } else if line.starts_with("vn ") {
            let pieces: Vec<_> = line.split_whitespace().collect();
            let x: f32 = pieces[1].parse().expect("Corrupt OBJ file");
            let y: f32 = pieces[2].parse().expect("Corrupt OBJ file");
            let z: f32 = pieces[3].parse().expect("Corrupt OBJ file");
            normals.push([x, y * 1.0, z]);
        } else if line.starts_with("f ") {
            let pieces: Vec<_> = line.split_whitespace().collect();
            let piece1 = pieces[1].split("/").collect::<Vec<_>>();
            let piece2 = pieces[2].split("/").collect::<Vec<_>>();
            let piece3 = pieces[3].split("/").collect::<Vec<_>>();
            let v1: usize = piece1[0].parse().unwrap();
            let v2: usize = piece2[0].parse().unwrap();
            let v3: usize = piece3[0].parse().unwrap();
            let n1: usize = piece1[2].parse().unwrap();
            let n2: usize = piece2[2].parse().unwrap();
            let n3: usize = piece3[2].parse().unwrap();

            faces.push((v1, v2, v3, n1, n2, n3));
        }
    }

    println!("loaded obj: {} verts, {} normals, {} faces", vertices.len(), normals.len(), faces.len());

    Ok(faces
        .iter()
        .flat_map(|(v1, v2, v3, n1, n2, n3)| {
            vec![
                Vertex {
                    position: vertices[*v1 - 1],
                    normal: normals[*n1 - 1],
                },
                Vertex {
                    position: vertices[*v2 - 1],
                    normal: normals[*n2 - 1],
                },
                Vertex {
                    position: vertices[*v3 - 1],
                    normal: normals[*n3 - 1],
                },
            ]
        })
        .collect())
}

pub fn create_vertices_for_cube(
    center_position: [f32; 3],
    radius: f32,
) -> Vec<Vertex> {
    CUBE_VERTICES
        .iter()
        .map(|vertex| Vertex {
            position: [
                vertex.position[0] * radius + center_position[0],
                vertex.position[1] * radius + center_position[1],
                vertex.position[2] * radius + center_position[2],
            ],
            normal: vertex.normal,
        })
        .collect()
}

pub fn create_vertices_for_cube_edges(
    center_position: [f32; 3],
    radius: f32,
) -> Vec<Vertex> {
    CUBE_EDGE_VERTICES
        .iter()
        .map(|vertex| Vertex {
            position: [
                vertex.position[0] * radius + center_position[0],
                vertex.position[1] * radius + center_position[1],
                vertex.position[2] * radius + center_position[2],
            ],
            normal: vertex.normal,
        })
        .collect()
}

pub fn create_vertices_for_sphere(
    center_position: [f32; 3],
    radius: f32,
    color: [f32; 3],
) -> Vec<Vertex> {
    SPHERE_VERTICES
        .iter()
        .map(|vertex| Vertex {
            position: [
                vertex.position[0] * radius + center_position[0],
                vertex.position[1] * radius + center_position[1],
                vertex.position[2] * radius + center_position[2],
            ],
            normal: vertex.normal,
        })
        .collect()
}
