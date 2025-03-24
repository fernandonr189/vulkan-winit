pub mod vertex_shader {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "
            #version 460

            layout(location = 0) in vec2 position;

            void main() {
                gl_Position = vec4(position, 0.0, 1.0);
            }
        ",
    }
}

pub mod fragment_shader {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "
            #version 460

            layout(location = 0) out vec4 f_color;

            layout(set = 0, binding = 0) uniform ColorUniform {
                vec4 input_color;
            };

            void main() {
                f_color = input_color;
            }
        ",
    }
}
