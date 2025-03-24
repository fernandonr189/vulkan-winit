use std::sync::Arc;

use vulkano::descriptor_set::DescriptorSet;

use crate::util::vulkano::vulkano_utils::SimpleVertex;

#[derive(Clone, Debug)]
pub struct Triangle {
    pub vertices: Vec<SimpleVertex>,
    pub descriptor_set: Option<Arc<DescriptorSet>>,
    pub color: [f32; 4],
}
