use std::sync::Arc;

use vulkano::{buffer::Subbuffer, descriptor_set::DescriptorSet};

use crate::util::vulkano::vulkano_utils::SimpleVertex;

#[derive(Clone, Debug)]
pub struct Triangle {
    pub vertices: Vec<SimpleVertex>,
    pub color: [f32; 4],
    pub descriptor_set: Option<Arc<DescriptorSet>>,
    pub vertex_buffer: Option<Subbuffer<[SimpleVertex]>>,
}

impl Triangle {
    pub fn new(vertices: Vec<SimpleVertex>, color: [f32; 4]) -> Self {
        Triangle {
            vertices,
            color,
            // Descriptor set and vertex buffer generated automatically in vulkan initialization
            descriptor_set: None,
            vertex_buffer: None,
        }
    }
}
