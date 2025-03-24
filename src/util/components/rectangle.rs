use std::sync::Arc;

use vulkano::{buffer::Subbuffer, descriptor_set::DescriptorSet};

use crate::util::vulkano::vulkano_utils::SimpleVertex;

#[derive(Clone, Debug)]
pub struct Rectangle {
    pub vertices: Vec<SimpleVertex>,
    pub color: [f32; 4],
    pub descriptor_set: Option<Arc<DescriptorSet>>,
    pub vertex_buffer: Option<Subbuffer<[SimpleVertex]>>,
}

impl Rectangle {
    pub fn new(x: f32, y: f32, width: f32, height: f32, color: [f32; 4]) -> Self {
        let vertices = vec![
            // First triangle (top-left, bottom-left, top-right)
            SimpleVertex { position: [x, y] },
            SimpleVertex {
                position: [x, y + height],
            },
            SimpleVertex {
                position: [x + width, y],
            },
            // Second triangle (bottom-left, bottom-right, top-right)
            SimpleVertex {
                position: [x, y + height],
            },
            SimpleVertex {
                position: [x + width, y + height],
            },
            SimpleVertex {
                position: [x + width, y],
            },
        ];
        Rectangle {
            vertices,
            color,
            // Descriptor set and vertex buffer generated automatically in vulkan initialization
            descriptor_set: None,
            vertex_buffer: None,
        }
    }
}
