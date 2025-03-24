use std::sync::Arc;

use vulkano::{buffer::Subbuffer, descriptor_set::DescriptorSet};

use crate::util::vulkano::vulkano_utils::SimpleVertex;

#[derive(Clone, Debug)]
pub struct Triangle {
    pub vertices: Vec<SimpleVertex>,
    pub descriptor_set: Option<Arc<DescriptorSet>>,
    pub vertex_buffer: Option<Subbuffer<[SimpleVertex]>>,
    pub color: [f32; 4],
}
