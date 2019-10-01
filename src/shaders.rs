use vulkano::device::Device;
use vulkano::pipeline::shader::GraphicsEntryPoint;
use vulkano::pipeline::shader::ShaderModule;

use shade_runner::{
    load, parse, Entry, FragInput, FragLayout, FragOutput, VertInput, VertLayout, VertOutput,
};

use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Clone)]
pub struct Shader {
    pub path: PathBuf,
    pub module: Arc<ShaderModule>,
    pub entry: Entry,
}

#[derive(Clone)]
pub struct ShaderSystem {
    pub vs: Shader,
    pub fs: Shader,
}

impl ShaderSystem {
    pub fn load_from_file(device: Arc<Device>, vs_path: &Path, fs_path: &Path) -> Self {
        let shaders = load(vs_path, fs_path).expect("Couldn't load shaders");
        let entry = parse(&shaders).expect("Couldn't parse shaders");

        let vs_module =
            unsafe { ShaderModule::from_words(device.clone(), &shaders.vertex) }.unwrap();

        let fs_module =
            unsafe { ShaderModule::from_words(device.clone(), &shaders.fragment) }.unwrap();

        let vs = Shader {
            path: vs_path.to_path_buf(),
            module: vs_module,
            entry: entry.clone(),
        };

        let fs = Shader {
            path: fs_path.to_path_buf(),
            module: fs_module,
            entry: entry.clone(),
        };

        Self { vs, fs }
    }

    pub fn get_entry_points(&self) -> (VertEntry, FragEntry) {
        let vs_entry = self.vs.entry.clone();
        let fs_entry = self.fs.entry.clone();

        let vert_main = unsafe {
            self.vs.module.graphics_entry_point(
                std::ffi::CStr::from_bytes_with_nul_unchecked(b"main\0"),
                vs_entry.vert_input,
                vs_entry.vert_output,
                vs_entry.vert_layout,
                vulkano::pipeline::shader::GraphicsShaderType::Vertex,
            )
        };

        let frag_main = unsafe {
            self.fs.module.graphics_entry_point(
                std::ffi::CStr::from_bytes_with_nul_unchecked(b"main\0"),
                fs_entry.frag_input,
                fs_entry.frag_output,
                fs_entry.frag_layout,
                vulkano::pipeline::shader::GraphicsShaderType::Fragment,
            )
        };

        (vert_main, frag_main)
    }
}

pub fn relative_path(local_path: &str) -> PathBuf {
    [env!("CARGO_MANIFEST_DIR"), local_path].iter().collect()
}

type VertEntry<'a> = GraphicsEntryPoint<'a, (), VertInput, VertOutput, VertLayout>;
type FragEntry<'a> = GraphicsEntryPoint<'a, (), FragInput, FragOutput, FragLayout>;
