use std::sync::Arc;

use vulkano::{buffer::Subbuffer, descriptor_set::DescriptorSet};

use crate::util::vulkano::vulkano_utils::SimpleVertex;

use super::{rectangle::Rectangle, triangle::Triangle};

#[derive(Clone)]
pub enum Shape {
    Triangle(Triangle),
    Rectangle(Rectangle),
}

impl Shape {
    pub fn new_triangle(vertices: Vec<SimpleVertex>, color: [f32; 4]) -> Self {
        Shape::Triangle(Triangle::new(vertices, color))
    }
    pub fn new_rectangle(x: f32, y: f32, width: f32, height: f32, color: [f32; 4]) -> Self {
        Shape::Rectangle(Rectangle::new(x, y, width, height, color))
    }
    pub fn get_color(&self) -> [f32; 4] {
        match self {
            Shape::Triangle(triangle) => triangle.color,
            Shape::Rectangle(rectangle) => rectangle.color,
        }
    }
    pub fn update_descriptor_set(&mut self, descriptor_set: Arc<DescriptorSet>) {
        match self {
            Shape::Triangle(triangle) => triangle.descriptor_set = Some(descriptor_set),
            Shape::Rectangle(rectangle) => rectangle.descriptor_set = Some(descriptor_set),
        }
    }
    pub fn get_descriptor_set(&self) -> Option<Arc<DescriptorSet>> {
        match self {
            Shape::Triangle(triangle) => triangle.descriptor_set.clone(),
            Shape::Rectangle(rectangle) => rectangle.descriptor_set.clone(),
        }
    }
    pub fn get_vertex_buffer(&self) -> Option<Subbuffer<[SimpleVertex]>> {
        match self {
            Shape::Triangle(triangle) => triangle.vertex_buffer.clone(),
            Shape::Rectangle(rectangle) => rectangle.vertex_buffer.clone(),
        }
    }
    pub fn update_vertex_buffer(&mut self, vertex_buffer: Subbuffer<[SimpleVertex]>) {
        match self {
            Shape::Triangle(triangle) => triangle.vertex_buffer = Some(vertex_buffer),
            Shape::Rectangle(rectangle) => rectangle.vertex_buffer = Some(vertex_buffer),
        }
    }
    pub fn get_vertices(&self) -> Vec<SimpleVertex> {
        match self {
            Shape::Triangle(triangle) => triangle.vertices.clone(),
            Shape::Rectangle(rectangle) => rectangle.vertices.clone(),
        }
    }
}
