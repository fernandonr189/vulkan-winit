use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes, WindowId},
};

use super::vulkano::vulkano_utils::Vulkan;

#[derive(Default)]
pub struct App {
    window: Option<Window>,
    vulkan: Option<Vulkan>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        match self.vulkan {
            Some(_) => {}
            None => {
                println!("Initializing Vulkan");
                let window = Arc::new(
                    event_loop
                        .create_window(WindowAttributes::default())
                        .unwrap(),
                );
                self.vulkan = Some(Vulkan::initialize(&window));
                println!("Vulkan initialized");
            }
        }
        self.window = Some(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::Resized(size) => {
                println!("Resized to {}x{}", size.width, size.height);
            }
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                //println!("Redrawing");

                self.window.as_ref().unwrap().request_redraw();
            }
            _ => {
                println!("Unhandled event");
            }
        }
    }
}
