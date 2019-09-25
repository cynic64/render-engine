use vulkano::device::Queue;

use std::sync::Arc;

use crate::camera::OrbitCamera;
use crate::producer::ProducerCollection;
use crate::render_passes;
use crate::system::{ComplexPass, System};

// TODO: shaders should be chosen here, because then everything is stored in one
// place and pieces becoming invalid is less likely

// TODO: for now all of these include cameras. That shouldn't be the case.
pub fn forward<'a>(queue: Arc<Queue>) -> (System<'a>, ProducerCollection) {
    let device = queue.device().clone();

    // create system
    let pass1 = ComplexPass {
        images_needed: vec![],
        images_created: vec!["color"],
        resources_needed: vec!["view_proj"],
        render_pass: render_passes::basic(device.clone()),
    };

    let output_tag = "color";

    let system = System::new(queue, vec![Box::new(pass1)], output_tag);

    // create producers
    let camera = OrbitCamera::default();
    // camera implements ResourceProducer
    let producer = Box::new(camera);
    let producer_collection = ProducerCollection::new(vec![producer]);

    (system, producer_collection)
}

pub fn forward_with_depth<'a>(queue: Arc<Queue>) -> (System<'a>, ProducerCollection) {
    let device = queue.device().clone();

    // create system
    let pass1 = ComplexPass {
        images_needed: vec![],
        images_created: vec!["color", "depth"],
        resources_needed: vec!["view_proj"],
        render_pass: render_passes::with_depth(device.clone()),
    };

    let output_tag = "color";

    let system = System::new(queue, vec![Box::new(pass1)], output_tag);

    // create producers
    // TODO: hopefully the duplication of all this stuff will be improved by a
    // more flexible system-templating system
    let camera = OrbitCamera::default();
    // camera implements ResourceProducer
    let producer = Box::new(camera);
    let producer_collection = ProducerCollection::new(vec![producer]);

    (system, producer_collection)
}

pub fn forward_msaa_depth<'a>(queue: Arc<Queue>) -> (System<'a>, ProducerCollection) {
    let device = queue.device().clone();

    // create system
    let pass1 = ComplexPass {
        images_needed: vec![],
        images_created: vec![
            "resolve_color",
            "multisampled_color",
            "multisampled_depth",
            "resolve_depth",
        ],
        resources_needed: vec!["view_proj"],
        render_pass: render_passes::multisampled_with_depth(device.clone(), 4),
    };

    let output_tag = "resolve_color";

    let system = System::new(queue, vec![Box::new(pass1)], output_tag);

    // create producers
    let camera = OrbitCamera::default();
    // camera implements ResourceProducer
    let producer = Box::new(camera);
    let producer_collection = ProducerCollection::new(vec![producer]);

    (system, producer_collection)
}