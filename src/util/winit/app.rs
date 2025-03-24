use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes, WindowId},
};

use crate::util::{
    components::triangle::Triangle,
    vulkano::vulkano_utils::{SimpleVertex, Vulkan},
};

#[derive(Default)]
pub struct App {
    window: Option<Arc<Window>>,
    vulkan: Option<Vulkan>,
    size: [u32; 2],
    resized: bool,
    recreate_swapchain: bool,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        match self.vulkan {
            Some(_) => {}
            None => {
                println!("Initializing Vulkan");
                self.window = Some(Arc::new(
                    event_loop
                        .create_window(WindowAttributes::default())
                        .unwrap(),
                ));
                let window = self.window.clone().unwrap();
                self.vulkan = Some(Vulkan::initialize(
                    &window,
                    vec![
                        Triangle::new(
                            vec![
                                SimpleVertex {
                                    position: [-1.0, -1.0],
                                },
                                SimpleVertex {
                                    position: [0.0, 0.0],
                                },
                                SimpleVertex {
                                    position: [-1.0, 0.0],
                                },
                            ],
                            [1.0, 0.0, 0.0, 1.0],
                        ),
                        Triangle::new(
                            vec![
                                SimpleVertex {
                                    position: [1.0, 1.0],
                                },
                                SimpleVertex {
                                    position: [0.0, 0.0],
                                },
                                SimpleVertex {
                                    position: [1.0, 0.0],
                                },
                            ],
                            [0.0, 1.0, 0.0, 1.0],
                        ),
                    ],
                ));
                println!("Vulkan initialized");
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::Resized(size) => {
                println!("Resized to {}x{}", size.width, size.height);
                self.size = [size.width, size.height];
                self.resized = true;
            }
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                println!("Redraw requested");
                if self.resized || self.recreate_swapchain {
                    self.resized = false;
                    match self.vulkan.as_mut() {
                        Some(vulkan) => {
                            let window = self.window.clone().unwrap();
                            vulkan.recreate_swapchain(&window);
                        }
                        None => {}
                    }
                }

                self.recreate_swapchain = self.vulkan.as_mut().unwrap().redraw();
            }
            _ => {}
        }
    }
}
