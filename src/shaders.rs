extern crate shade_runner;
extern crate vulkano;

use vulkano::pipeline::shader::ShaderModule;
use vulkano::device::Device;
use shade_runner::{Entry, load, parse};

use std::path::Path;
use std::sync::Arc;

#[derive(Clone)]
pub struct Shader {
    pub module: Arc<ShaderModule>,
    pub entry: Entry,
}

impl Shader {
    pub fn load_from_file(device: Arc<Device>, vs_path: &Path, fs_path: &Path) -> (Self, Self) {
        let shaders = load(vs_path, fs_path).expect("Couldn't load shaders");
        let entry = parse(&shaders).expect("Couldn't parse shaders");

        let vs_module = unsafe {
            ShaderModule::from_words(device.clone(), &shaders.vertex)
        }
        .unwrap();

        let fs_module = unsafe {
            ShaderModule::from_words(device.clone(), &shaders.fragment)
        }
        .unwrap();

        let vs = Shader {
            module: vs_module,
            entry: entry.clone(),
        };

        let fs = Shader {
            module: fs_module,
            entry: entry.clone(),
        };

        (vs, fs)
    }
}
