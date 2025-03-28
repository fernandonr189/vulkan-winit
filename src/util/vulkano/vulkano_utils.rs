use std::sync::Arc;

type FenceFuture = FenceSignalFuture<
    PresentFuture<CommandBufferExecFuture<JoinFuture<Box<dyn GpuFuture>, SwapchainAcquireFuture>>>,
>;

use vulkano::{
    Validated, VulkanError, VulkanLibrary,
    buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage},
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferExecFuture, CommandBufferUsage,
        PrimaryAutoCommandBuffer, RenderPassBeginInfo, SubpassBeginInfo, SubpassContents,
        SubpassEndInfo, allocator::StandardCommandBufferAllocator,
    },
    descriptor_set::{
        DescriptorSet, WriteDescriptorSet, allocator::StandardDescriptorSetAllocator,
    },
    device::{
        Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo, QueueFlags,
        physical::{PhysicalDevice, PhysicalDeviceType},
    },
    format::{ClearValue, Format},
    image::{Image, ImageCreateInfo, ImageType, ImageUsage, SampleCount, view::ImageView},
    instance::{Instance, InstanceCreateFlags, InstanceCreateInfo},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    pipeline::{
        GraphicsPipeline, Pipeline, PipelineBindPoint, PipelineLayout,
        PipelineShaderStageCreateInfo,
        graphics::{
            GraphicsPipelineCreateInfo,
            color_blend::{ColorBlendAttachmentState, ColorBlendState},
            input_assembly::InputAssemblyState,
            multisample::MultisampleState,
            rasterization::RasterizationState,
            vertex_input::{Vertex, VertexDefinition, VertexInputState},
            viewport::{Viewport, ViewportState},
        },
        layout::PipelineDescriptorSetLayoutCreateInfo,
    },
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
    swapchain::{
        self, ColorSpace, PresentFuture, PresentMode, Surface, Swapchain, SwapchainAcquireFuture,
        SwapchainCreateInfo, SwapchainPresentInfo,
    },
    sync::{
        self, GpuFuture,
        future::{FenceSignalFuture, JoinFuture},
    },
};
use winit::window::Window;

use crate::util::{
    components::shape::Shape,
    shaders::shaders::{fragment_shader, vertex_shader},
};

pub struct Vulkan {
    swapchain: Arc<Swapchain>,
    render_pass: Arc<RenderPass>,
    viewport: Viewport,
    device: Arc<Device>,
    command_buffers: Vec<Arc<PrimaryAutoCommandBuffer>>,
    queue: Arc<Queue>,
    elements: Vec<Shape>,
    fences: Vec<Option<Arc<FenceFuture>>>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    previous_fence: u32,
    stages: [PipelineShaderStageCreateInfo; 2],
    vertex_input_state: VertexInputState,
    layout: Arc<PipelineLayout>,
    multisample_state: MultisampleState,
}
impl Vulkan {
    pub fn redraw(&mut self) -> bool {
        let swapchain = self.swapchain.clone();
        let mut recreate_swapchain = false;
        let (image_i, suboptimal, acquire_future) =
            match swapchain::acquire_next_image(swapchain.clone(), None).map_err(Validated::unwrap)
            {
                Ok(r) => r,
                Err(VulkanError::OutOfDate) => {
                    return true;
                }
                Err(e) => panic!("failed to acquire next image: {e}"),
            };

        if suboptimal {
            recreate_swapchain = true;
        }
        if let Some(image_fence) = &self.fences[image_i as usize] {
            image_fence.wait(None).unwrap();
        }

        let previous_future = match self.fences[self.previous_fence as usize].clone() {
            None => {
                let mut now = sync::now(self.device.clone());
                now.cleanup_finished();

                now.boxed()
            }
            Some(fence) => fence.boxed(),
        };
        let future = previous_future
            .join(acquire_future)
            .then_execute(
                self.queue.clone(),
                self.command_buffers[image_i as usize].clone(),
            )
            .unwrap()
            .then_swapchain_present(
                self.queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(swapchain.clone(), image_i),
            )
            .then_signal_fence_and_flush();

        self.fences[image_i as usize] = match future.map_err(Validated::unwrap) {
            Ok(value) => Some(Arc::new(value)),
            Err(VulkanError::OutOfDate) => {
                recreate_swapchain = true;
                None
            }
            Err(e) => {
                println!("failed to flush future: {e}");
                None
            }
        };
        self.previous_fence = image_i;
        return recreate_swapchain;
    }
    pub fn recreate_swapchain(&mut self, window: &Arc<Window>) {
        let new_dimensions = window.inner_size();

        let (new_swapchain, new_images) = self
            .swapchain
            .recreate(SwapchainCreateInfo {
                image_extent: new_dimensions.into(),
                ..self.swapchain.create_info()
            })
            .expect("failed to recreate swapchain");
        self.swapchain = new_swapchain;

        let multisampled_image = create_multisampled_image(
            &self.memory_allocator.clone(),
            window.inner_size().into(),
            self.swapchain.image_format(),
            self.multisample_state.rasterization_samples,
        );

        let new_framebuffers =
            get_framebuffers(&new_images, &multisampled_image, &self.render_pass.clone());

        self.viewport.extent = new_dimensions.into();
        let new_pipeline = get_pipeline(
            &self.device.clone(),
            &self.render_pass.clone(),
            self.viewport.clone(),
            self.layout.clone(),
            self.stages.clone(),
            &self.vertex_input_state,
            self.multisample_state.clone(),
        );

        self.command_buffers = get_command_buffers(
            &self.command_buffer_allocator,
            &self.queue,
            &new_pipeline,
            &new_framebuffers,
            self.elements.clone(),
            &self.memory_allocator,
        );
    }
    pub fn initialize(
        window: &Arc<Window>,
        mut elements: Vec<Shape>,
        allow_tearing: bool,
        samples: SampleCount,
    ) -> Self {
        let instance = create_instance(window).expect("Failed to create Vulkan instance");
        let surface = Surface::from_window(instance.clone(), window.clone())
            .expect("Failed to create Vulkan surface");
        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };

        let (physical_device, queue_family_index) =
            select_physical_device(&instance, &surface, &device_extensions);

        let (device, mut queues) = Device::new(
            physical_device.clone(),
            DeviceCreateInfo {
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],
                enabled_extensions: device_extensions, // new
                ..Default::default()
            },
        )
        .expect("failed to create device");

        let queue = queues.next().unwrap();

        let (swapchain, images) =
            create_swapchain(&physical_device, &surface, &window, &device, allow_tearing);

        let render_pass = get_render_pass(device.clone(), swapchain.clone(), samples);

        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
        let multisampled_image = create_multisampled_image(
            &memory_allocator.clone(),
            window.inner_size().into(),
            swapchain.image_format(),
            samples,
        );
        let framebuffers = get_framebuffers(&images, &multisampled_image, &render_pass.clone());

        let multisample_state = MultisampleState {
            rasterization_samples: samples,
            ..MultisampleState::default()
        };

        let vs = vertex_shader::load(device.clone()).expect("failed to create shader module");
        let fs = fragment_shader::load(device.clone()).expect("failed to create shader module");

        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: window.inner_size().into(),
            depth_range: 0.0..=1.0,
        };

        let vs = vs.entry_point("main").unwrap();
        let fs = fs.entry_point("main").unwrap();

        let vertex_input_state = SimpleVertex::per_vertex().definition(&vs).unwrap();

        let stages = [
            PipelineShaderStageCreateInfo::new(vs),
            PipelineShaderStageCreateInfo::new(fs),
        ];

        let layout = get_layout(&device, stages.clone());

        let pipeline = get_pipeline(
            &device.clone(),
            &render_pass.clone(),
            viewport.clone(),
            layout.clone(),
            stages.clone(),
            &vertex_input_state,
            multisample_state.clone(),
        );

        for element in elements.iter_mut() {
            let color_buffer = Buffer::from_data(
                memory_allocator.clone(),
                BufferCreateInfo {
                    usage: BufferUsage::UNIFORM_BUFFER,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                        | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
                ColorUniform {
                    input_color: element.get_color(),
                },
            )
            .unwrap();
            let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
                device.clone(),
                Default::default(),
            ));
            let pipeline_layout = pipeline.layout();

            let descriptor_set_layouts = pipeline_layout.set_layouts();
            let descriptor_set_layout_index = 0;
            let descriptor_set_layout = descriptor_set_layouts
                .get(descriptor_set_layout_index)
                .unwrap();
            let descriptor_set = DescriptorSet::new(
                descriptor_set_allocator,
                descriptor_set_layout.clone(),
                [WriteDescriptorSet::buffer(0, color_buffer)],
                [],
            )
            .unwrap();
            element.update_descriptor_set(descriptor_set);
        }

        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            Default::default(),
        ));

        let command_buffers = get_command_buffers(
            &command_buffer_allocator,
            &queue,
            &pipeline,
            &framebuffers,
            elements.clone(),
            &memory_allocator,
        );
        let frames_in_flight = images.len();
        Vulkan {
            swapchain,
            render_pass,
            viewport,
            device,
            command_buffers,
            queue,
            elements,
            fences: vec![None; frames_in_flight],
            previous_fence: 0,
            memory_allocator,
            command_buffer_allocator,
            stages,
            vertex_input_state,
            layout,
            multisample_state,
        }
    }
}

fn get_command_buffers(
    command_buffer_allocator: &Arc<StandardCommandBufferAllocator>,
    queue: &Arc<Queue>,
    pipeline: &Arc<GraphicsPipeline>,
    framebuffers: &Vec<Arc<Framebuffer>>,
    mut elements: Vec<Shape>,
    memory_allocator: &Arc<StandardMemoryAllocator>,
) -> Vec<Arc<PrimaryAutoCommandBuffer>> {
    framebuffers
        .iter()
        .map(|framebuffer| {
            let clear_values_count = framebuffer.attachments().len();
            let clear_values: Vec<Option<ClearValue>> =
                vec![Some([0.1, 0.1, 0.1, 1.0].into()); clear_values_count];
            let mut builder = AutoCommandBufferBuilder::primary(
                command_buffer_allocator.clone(),
                queue.queue_family_index(),
                CommandBufferUsage::MultipleSubmit,
            )
            .unwrap();

            unsafe {
                builder
                    .begin_render_pass(
                        RenderPassBeginInfo {
                            clear_values,
                            ..RenderPassBeginInfo::framebuffer(framebuffer.clone())
                        },
                        SubpassBeginInfo {
                            contents: SubpassContents::Inline,
                            ..Default::default()
                        },
                    )
                    .unwrap()
                    .bind_pipeline_graphics(pipeline.clone())
                    .unwrap();
                for element in elements.iter_mut() {
                    match element.get_vertex_buffer() {
                        Some(_) => {}
                        None => {
                            let vertex_buffer = Buffer::from_iter(
                                memory_allocator.clone(),
                                BufferCreateInfo {
                                    usage: BufferUsage::VERTEX_BUFFER,
                                    ..Default::default()
                                },
                                AllocationCreateInfo {
                                    memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                                        | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                                    ..Default::default()
                                },
                                element.get_vertices().clone(),
                            )
                            .unwrap();
                            element.update_vertex_buffer(vertex_buffer);
                        }
                    }

                    builder
                        .bind_descriptor_sets(
                            PipelineBindPoint::Graphics,
                            pipeline.layout().clone(),
                            0,
                            element.get_descriptor_set().clone().unwrap(),
                        )
                        .unwrap()
                        .bind_vertex_buffers(0, element.get_vertex_buffer().clone().unwrap())
                        .unwrap()
                        .draw(
                            element.get_vertex_buffer().clone().unwrap().len() as u32,
                            1,
                            0,
                            0,
                        )
                        .unwrap();
                }
                builder.end_render_pass(SubpassEndInfo::default()).unwrap();
            }

            builder.build().unwrap()
        })
        .collect()
}

fn get_layout(
    device: &Arc<Device>,
    stages: [PipelineShaderStageCreateInfo; 2],
) -> Arc<PipelineLayout> {
    PipelineLayout::new(
        device.clone(),
        PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
            .into_pipeline_layout_create_info(device.clone())
            .unwrap(),
    )
    .unwrap()
}

fn get_pipeline(
    device: &Arc<Device>,
    render_pass: &Arc<RenderPass>,
    viewport: Viewport,
    layout: Arc<PipelineLayout>,
    stages: [PipelineShaderStageCreateInfo; 2],
    vertex_input_state: &VertexInputState,
    multisample_state: MultisampleState,
) -> Arc<GraphicsPipeline> {
    let subpass = Subpass::from(render_pass.clone(), 0).unwrap();

    GraphicsPipeline::new(
        device.clone(),
        None,
        GraphicsPipelineCreateInfo {
            stages: stages.into_iter().collect(),
            vertex_input_state: Some(vertex_input_state.clone()),
            input_assembly_state: Some(InputAssemblyState::default()),
            viewport_state: Some(ViewportState {
                viewports: [viewport].into_iter().collect(),
                ..Default::default()
            }),
            rasterization_state: Some(RasterizationState::default()),
            multisample_state: Some(multisample_state),
            color_blend_state: Some(ColorBlendState::with_attachment_states(
                subpass.num_color_attachments(),
                ColorBlendAttachmentState::default(),
            )),
            subpass: Some(subpass.into()),
            ..GraphicsPipelineCreateInfo::layout(layout)
        },
    )
    .unwrap()
}
fn get_framebuffers(
    images: &[Arc<Image>],
    multisampled_image: &Arc<ImageView>,
    render_pass: &Arc<RenderPass>,
) -> Vec<Arc<Framebuffer>> {
    images
        .iter()
        .map(|image| {
            let view = ImageView::new_default(image.clone()).unwrap();
            Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![multisampled_image.clone(), view],
                    ..Default::default()
                },
            )
            .unwrap()
        })
        .collect::<Vec<_>>()
}

fn get_render_pass(
    device: Arc<Device>,
    swapchain: Arc<Swapchain>,
    samples: SampleCount,
) -> Arc<RenderPass> {
    vulkano::single_pass_renderpass!(
        device,
        attachments: {
            multisample: {
                format: swapchain.image_format(),
                samples: samples,
                load_op: Clear,
                store_op: DontCare,
            },
            color: {
                format: swapchain.image_format(),
                samples: SampleCount::Sample1,
                load_op: Clear,
                store_op: Store,
            },
        },
        pass: {
            color: [multisample],
            color_resolve: [color],
            depth_stencil: {},
        },
    )
    .unwrap()
}
fn select_physical_device(
    instance: &Arc<Instance>,
    surface: &Arc<Surface>,
    device_extensions: &DeviceExtensions,
) -> (Arc<PhysicalDevice>, u32) {
    instance
        .enumerate_physical_devices()
        .expect("could not enumerate devices")
        .filter(|p| p.supported_extensions().contains(&device_extensions))
        .filter_map(|p| {
            p.queue_family_properties()
                .iter()
                .enumerate()
                .position(|(i, q)| {
                    q.queue_flags.contains(QueueFlags::GRAPHICS)
                        && p.surface_support(i as u32, &surface).unwrap_or(false)
                })
                .map(|q| (p, q as u32))
        })
        .min_by_key(|(p, _)| match p.properties().device_type {
            PhysicalDeviceType::DiscreteGpu => 0,
            PhysicalDeviceType::IntegratedGpu => 1,
            PhysicalDeviceType::VirtualGpu => 2,
            PhysicalDeviceType::Cpu => 3,
            _ => 4,
        })
        .expect("no device available")
}

fn create_instance(window: &Arc<Window>) -> Result<Arc<Instance>, Validated<VulkanError>> {
    let library = VulkanLibrary::new().expect("no local Vulkan library/DLL");
    let required_extensions = Surface::required_extensions(&(*window)).unwrap();
    let instance = Instance::new(
        library,
        InstanceCreateInfo {
            flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
            enabled_extensions: required_extensions,
            ..Default::default()
        },
    );
    instance
}

fn choose_memory_efficient_format(
    available_formats: &Vec<(Format, ColorSpace)>,
) -> vulkano::format::Format {
    use vulkano::format::Format;
    let preferred_formats = [
        Format::R5G6B5_UNORM_PACK16,   // 16 bits per pixel
        Format::A1R5G5B5_UNORM_PACK16, // 16 bits per pixel
        Format::B4G4R4A4_UNORM_PACK16, // 16 bits per pixel
        Format::R8G8B8_UNORM,          // 24 bits per pixel
        Format::B8G8R8_UNORM,          // 24 bits per pixel
        Format::R8G8B8A8_UNORM,        // 32 bits per pixel
        Format::B8G8R8A8_UNORM,        // 32 bits per pixel
        Format::R8G8B8A8_SRGB,         // 32 bits per pixel
        Format::B8G8R8A8_SRGB,         // 32 bits per pixel
    ];

    for preferred_format in preferred_formats.iter() {
        if available_formats
            .iter()
            .any(|(format, _)| format == preferred_format)
        {
            return *preferred_format;
        }
    }
    available_formats[0].0
}

fn create_multisampled_image(
    allocator: &Arc<StandardMemoryAllocator>,
    extent: [u32; 2],
    format: Format,
    samples: SampleCount,
) -> Arc<ImageView> {
    let image = Image::new(
        allocator.clone(),
        ImageCreateInfo {
            image_type: ImageType::Dim2d,
            format,
            extent: [extent[0], extent[1], 1],
            usage: ImageUsage::TRANSIENT_ATTACHMENT | ImageUsage::COLOR_ATTACHMENT,
            samples,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
            ..Default::default()
        },
    )
    .unwrap();

    ImageView::new_default(image).unwrap()
}

fn create_swapchain(
    physical_device: &Arc<PhysicalDevice>,
    surface: &Arc<Surface>,
    window: &Arc<Window>,
    device: &Arc<Device>,
    allow_tearing: bool,
) -> (Arc<Swapchain>, Vec<Arc<Image>>) {
    let caps = physical_device
        .surface_capabilities(&surface, Default::default())
        .expect("failed to get surface capabilities");

    let dimensions = window.inner_size();
    let composite_alpha = caps.supported_composite_alpha.into_iter().next().unwrap();
    let image_formats = physical_device
        .surface_formats(&surface, Default::default())
        .unwrap();
    let image_format = choose_memory_efficient_format(&image_formats);

    Swapchain::new(
        device.clone(),
        surface.clone(),
        SwapchainCreateInfo {
            min_image_count: caps.min_image_count,
            image_format,
            image_extent: dimensions.into(),
            image_usage: ImageUsage::COLOR_ATTACHMENT | ImageUsage::TRANSFER_DST,
            composite_alpha,
            present_mode: if allow_tearing {
                PresentMode::Immediate
            } else {
                PresentMode::Fifo
            },
            ..Default::default()
        },
    )
    .unwrap()
}

#[derive(BufferContents, Vertex, Clone, Debug)]
#[repr(C)]
pub struct SimpleVertex {
    #[format(R32G32_SFLOAT)]
    pub position: [f32; 2],
}

#[repr(C)]
#[derive(Default, BufferContents)]
struct ColorUniform {
    input_color: [f32; 4],
}
