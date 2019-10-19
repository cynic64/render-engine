use vulkano::device::Device;
use vulkano::framebuffer::{RenderPassAbstract, Subpass};
use vulkano::pipeline::input_assembly::PrimitiveTopology;
use vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineAbstract};

use std::path::PathBuf;
use std::sync::Arc;

use crate::input::get_elapsed;
use crate::mesh::{SimpleVertex, Vertex};
use crate::shaders::ShaderSystem;

// pipeline caches are specific to a single render pass.
pub struct PipelineCache {
    // TODO: switch to a hashmap
    c_pipes: Vec<CachedPipeline>,
    device: Arc<Device>,
    render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    stats: CacheStats,
}

struct CachedPipeline {
    spec: PipelineSpec,
    pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
}

#[derive(Default)]
struct CacheStats {
    hits: u32,
    misses: u32,
    gen_times: Vec<f32>,
}

#[derive(PartialEq, Clone, Debug)]
pub struct PipelineSpec {
    pub vs_path: PathBuf,
    pub fs_path: PathBuf,
    pub fill_type: PrimitiveTopology,
    pub depth: bool,
}

impl PipelineCache {
    pub fn new(
        device: Arc<Device>,
        render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    ) -> Self {
        Self {
            c_pipes: vec![],
            device,
            render_pass,
            stats: CacheStats::default(),
        }
    }

    pub fn get(&mut self, spec: &PipelineSpec) -> Arc<dyn GraphicsPipelineAbstract + Send + Sync> {
        let mut pipeline = None;

        // first search through cached pipelines to see if we have one with matching spec
        for c_pipe in self.c_pipes.iter() {
            if c_pipe.spec == *spec {
                pipeline = Some(c_pipe.pipeline.clone());
                self.stats.hits += 1;
            }
        }

        match pipeline {
            Some(pipeline) => pipeline,
            None => {
                self.stats.misses += 1;
                let start_time = std::time::Instant::now();

                let shader_sys =
                    ShaderSystem::load_from_file(self.device.clone(), &spec.vs_path, &spec.fs_path);

                let (vs_main, fs_main) = shader_sys.get_entry_points();

                // TODO: right now whether a depth pass is included or not
                // determines the vertex type. This is dumb. Fix it with dynamic
                // traits and all that crap I hate.
                let pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync> = if spec.depth {
                    Arc::new(
                        GraphicsPipeline::start()
                            .vertex_input_single_buffer::<Vertex>()
                            .vertex_shader(vs_main, ())
                            .primitive_topology(spec.fill_type)
                            .viewports_dynamic_scissors_irrelevant(1)
                            .fragment_shader(fs_main, ())
                            .render_pass(Subpass::from(self.render_pass.clone(), 0).unwrap())
                            .depth_stencil_simple_depth()
                            .build(self.device.clone())
                            .unwrap(),
                    )
                } else {
                    Arc::new(
                        GraphicsPipeline::start()
                            .vertex_input_single_buffer::<SimpleVertex>()
                            .vertex_shader(vs_main, ())
                            .primitive_topology(spec.fill_type)
                            .viewports_dynamic_scissors_irrelevant(1)
                            .fragment_shader(fs_main, ())
                            .render_pass(Subpass::from(self.render_pass.clone(), 0).unwrap())
                            .build(self.device.clone())
                            .unwrap(),
                    )
                };

                let c_pipe = CachedPipeline {
                    spec: spec.clone(),
                    pipeline: pipeline.clone(),
                };

                self.c_pipes.push(c_pipe);

                self.stats.gen_times.push(get_elapsed(start_time));

                pipeline
            }
        }
    }

    pub fn print_stats(&self) {
        let avg: f32 =
            self.stats.gen_times.iter().sum::<f32>() / (self.stats.gen_times.len() as f32);
        let percent =
            (self.stats.hits as f32) / ((self.stats.hits + self.stats.misses) as f32) * 100.0;
        println!(
            "Hits: {}, misses: {}, {}%, avg. time taken to gen pipeline: {}",
            self.stats.hits, self.stats.misses, percent, avg
        );
    }
}
