use crate::util::vulkano::vulkano_utils::SimpleVertex;

pub struct Triangle {
    pub vertices: [SimpleVertex; 3],
}

impl Triangle {
    pub fn new() -> Self {
        let vertices = [
            SimpleVertex {
                position: [-0.5, -0.5],
            },
            SimpleVertex {
                position: [0.5, -0.5],
            },
            SimpleVertex {
                position: [0.0, 0.5],
            },
        ];
        Triangle { vertices }
    }
}
