use vulkano::device::{Device, Queue};

use std::sync::Arc;
use std::collections::HashMap;

use crate::input::FrameInfo;
use crate::producer::ProducerCollection;
use crate::system::System;
use crate::template_systems;
use crate::world::*;
use crate::window::Window;

pub struct App<'a> {
    device: Arc<Device>,
    queue: Arc<Queue>,
    pub done: bool,
    world: World,
    window: Window,
    system: System<'a>,
    producers: ProducerCollection,
}

impl<'a> App<'a> {
    pub fn new() -> Self {
        // create the system
        let mut window = Window::new();
        let queue = window.get_queue();
        let device = queue.device().clone();

        let (system, producers) = template_systems::forward_with_depth(queue.clone());
        // TODO: which render pass does this refer to?
        let render_pass = system.get_passes()[0].get_render_pass().clone();
        window.set_render_pass(render_pass.clone());

        let world = World::new(render_pass, device.clone());

        Self {
            device,
            queue,
            done: false,
            world,
            window,
            system,
            producers,
        }
    }

    pub fn get_world_com(&self) -> WorldCommunicator {
        self.world.get_communicator()
    }

    // TODO: create a separate module for managing systems. hopefully it would
    // let you do stuff like compose whether multisampling was enabled and AO
    // and all that

    // another idea: "optimizations" for different passes, kinda like vulkano's
    // command buffers and images. For example, if there are never any
    // additional resources, the check can be skipped.
    pub fn set_system(&mut self, system: System<'a>) {
        // for now it assumes a single pass is used in the system
        // TODO: make it so it can figure out which pass belongs to the world
        // and which belongs to the window
        let render_pass = system.get_passes()[0].get_render_pass();
        self.window.set_render_pass(render_pass.clone());
        self.world.set_render_pass(render_pass.clone());
        self.system = system;
    }

    pub fn set_producers(&mut self, new_producers: ProducerCollection) {
        self.producers = new_producers;
    }

    pub fn draw_frame(&mut self) {
        self.draw_objects();

        self.handle_input();
        self.world.update();
    }

    fn handle_input(&mut self) {
        self.done = self.window.update();

        self.producers
            .update(self.window.get_frame_info());
    }

    pub fn get_frame_info(&mut self) -> FrameInfo {
        self.window.get_frame_info()
    }

    pub fn print_fps(&self) {
        let fps = self.window.get_fps();
        println!("FPS: {}", fps);
        self.system.print_stats();
    }

    pub fn get_device(&self) -> Arc<Device> {
        self.device.clone()
    }

    pub fn get_queue(&self) -> Arc<Queue> {
        self.queue.clone()
    }

    fn draw_objects(&mut self) {
        let world_renderable_objects = self.world.get_objects();
        let mut all_renderable_objects = HashMap::new();
        all_renderable_objects.insert("geometry", world_renderable_objects);
        let swapchain_image = self.window.next_image();
        let swapchain_fut = self.window.get_future();
        let shared_resources = self.producers.get_shared_resources(self.device.clone());

        // draw_frame returns a future representing the completion of rendering
        let frame_fut = self.system.draw_frame(
            swapchain_image.dimensions(),
            all_renderable_objects,
            shared_resources,
            swapchain_image,
            swapchain_fut,
        );

        self.window.present_future(frame_fut);
    }
}

impl<'a> Default for App<'a> {
    fn default() -> Self {
        Self::new()
    }
}
