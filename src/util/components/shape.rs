use std::sync::Arc;

use vulkano::{buffer::Subbuffer, descriptor_set::DescriptorSet};

use crate::util::vulkano::vulkano_utils::SimpleVertex;

use super::triangle::Triangle;

#[derive(Clone)]
pub enum Shape {
    Triangle(Triangle),
}

impl Shape {
    pub fn new_triangle(vertices: Vec<SimpleVertex>, color: [f32; 4]) -> Self {
        Shape::Triangle(Triangle::new(vertices, color))
    }
    pub fn get_color(&self) -> [f32; 4] {
        match self {
            Shape::Triangle(triangle) => triangle.color,
        }
    }
    pub fn update_descriptor_set(&mut self, descriptor_set: Arc<DescriptorSet>) {
        match self {
            Shape::Triangle(triangle) => triangle.descriptor_set = Some(descriptor_set),
        }
    }
    pub fn get_descriptor_set(&self) -> Option<Arc<DescriptorSet>> {
        match self {
            Shape::Triangle(triangle) => triangle.descriptor_set.clone(),
        }
    }
    pub fn get_vertex_buffer(&self) -> Option<Subbuffer<[SimpleVertex]>> {
        match self {
            Shape::Triangle(triangle) => triangle.vertex_buffer.clone(),
        }
    }
    pub fn update_vertex_buffer(&mut self, vertex_buffer: Subbuffer<[SimpleVertex]>) {
        match self {
            Shape::Triangle(triangle) => triangle.vertex_buffer = Some(vertex_buffer),
        }
    }
    pub fn get_vertices(&self) -> Vec<SimpleVertex> {
        match self {
            Shape::Triangle(triangle) => triangle.vertices.clone(),
        }
    }
}
