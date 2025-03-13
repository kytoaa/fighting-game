use ash::{vk, Device, Instance};

mod deinitialization;
mod initialization;

mod frame_recording;
mod graphics_pipeline;
mod render_pass;

mod commands;
mod resources;

mod descriptors;
mod shaders;
mod textures;
mod uniforms;
mod vertices;

use super::Vector2;
use asset_manager::SpriteHandle;

const MAX_FRAMES_IN_FLIGHT: usize = 2;

struct VulkanImage(vk::Image, vk::ImageView);
struct VulkanObject<T>(T, vk::DeviceMemory);

struct Queues {
    graphics: vk::Queue,
    transfer: vk::Queue,
}

struct SwapchainInfo {
    format: vk::Format,
    extent: vk::Extent2D,
}

struct CoreRenderData {
    instance: Instance,
    device: Device,
    physical_device: vk::PhysicalDevice,

    surface_instance: ash::khr::surface::Instance,
    surface: vk::SurfaceKHR,
    queues: Queues,

    swapchain_device: ash::khr::swapchain::Device,
    swapchain: vk::SwapchainKHR,
    swapchain_info: SwapchainInfo,
    present_images: Vec<VulkanImage>,

    command_pool: vk::CommandPool,
    transfer_pool: vk::CommandPool,

    command_buffers: Vec<vk::CommandBuffer>,

    depth_image: VulkanObject<VulkanImage>,

    debug_callback: vk::DebugUtilsMessengerEXT,
    debug_utils_instance: ash::ext::debug_utils::Instance,
}

pub struct Renderer {
    core: CoreRenderData,

    render_pass: vk::RenderPass,
    framebuffers: Vec<vk::Framebuffer>,
    pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,

    image_available_semaphores: [vk::Semaphore; MAX_FRAMES_IN_FLIGHT],
    render_finished_semaphores: [vk::Semaphore; MAX_FRAMES_IN_FLIGHT],
    in_flight_fences: [vk::Fence; MAX_FRAMES_IN_FLIGHT],

    sampler: vk::Sampler,

    vertex_buffers: vertices::VertexBuffer<MAX_FRAMES_IN_FLIGHT>,
    uniform_buffers: uniforms::UniformBuffer<MAX_FRAMES_IN_FLIGHT>,

    descriptor_pool: vk::DescriptorPool,
    descriptor_set_layout: vk::DescriptorSetLayout,
    descriptor_sets: [vk::DescriptorSet; MAX_FRAMES_IN_FLIGHT],

    images: std::collections::HashMap<SpriteHandle, VulkanObject<VulkanImage>>,

    frame: u32,
}

impl Renderer {
    pub fn init(
        display_handle: &dyn winit::raw_window_handle::HasDisplayHandle,
        window_handle: &dyn winit::raw_window_handle::HasWindowHandle,
        window_size: (u32, u32),
    ) -> Result<Renderer, Box<dyn std::error::Error>> {
        let core = CoreRenderData::init(display_handle, window_handle, window_size);

        let render_pass = render_pass::create_render_pass(&core);

        let framebuffers = initialization::create_framebuffers(&core, &render_pass);

        let vertex_buffers = vertices::create_vertex_buffer(&core);
        let uniform_buffers = uniforms::create_uniform_buffer(&core);

        let sampler = textures::create_image_sampler(&core);

        let descriptor_pool = descriptors::create_descriptor_pool(&core);
        let descriptor_set_layout = descriptors::create_descriptor_set_layout(&core);
        let descriptor_sets = descriptors::create_descriptor_sets(
            &core,
            &descriptor_pool,
            &descriptor_set_layout,
            &uniform_buffers.get_buffers(),
            &sampler,
        );

        let (pipeline, pipeline_layout) = graphics_pipeline::create_graphics_pipeline(
            &core,
            &render_pass,
            std::slice::from_ref(&descriptor_set_layout),
        );

        let (image_available_semaphores, render_finished_semaphores, in_flight_fences) = {
            let semaphore_create_info = vk::SemaphoreCreateInfo::default();
            let fence_create_info =
                vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);

            (
                [(); MAX_FRAMES_IN_FLIGHT].map(|_| {
                    unsafe { core.device.create_semaphore(&semaphore_create_info, None) }
                        .expect("failed to create semaphore")
                }),
                [(); MAX_FRAMES_IN_FLIGHT].map(|_| {
                    unsafe { core.device.create_semaphore(&semaphore_create_info, None) }
                        .expect("failed to create semaphore")
                }),
                [(); MAX_FRAMES_IN_FLIGHT].map(|_| {
                    unsafe { core.device.create_fence(&fence_create_info, None) }
                        .expect("failed to create fence")
                }),
            )
        };

        Ok(Renderer {
            core,
            render_pass,
            pipeline,
            pipeline_layout,

            framebuffers,

            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,

            sampler,

            vertex_buffers,
            uniform_buffers,

            descriptor_pool,
            descriptor_set_layout,
            descriptor_sets,

            images: Default::default(),

            frame: 0,
        })
    }
}

impl Renderer {
    pub fn draw_frame(
        &mut self,
        assets: &asset_manager::AssetManager,
        sprites: Vec<(SpriteHandle, Vector2, bool, f32)>,
    ) {
        let frame = self.frame as usize % MAX_FRAMES_IN_FLIGHT;

        let images: Vec<_> = sprites
            .iter()
            .map(|s| s.0)
            .map(|handle| match self.images.get(&handle) {
                Some(image) => image.0 .1,
                None => {
                    let image = assets.get_sprite(handle);
                    let data = image.data().data();
                    let image = textures::create_image(&self.core, (image.size(), data));
                    self.images.insert(handle, image);
                    self.images.get(&handle).unwrap().0 .1
                }
            })
            .collect();

        self.populate_vertex_index_buffers(
            sprites.iter().map(|(handle, position, flipped, depth)| {
                (
                    assets.get_sprite(*handle).size(),
                    *position,
                    *flipped,
                    *depth,
                )
            }),
            frame,
        );

        unsafe {
            self.core
                .device
                .wait_for_fences(
                    std::slice::from_ref(&self.in_flight_fences[frame]),
                    true,
                    u64::MAX,
                )
                .unwrap();

            let (image_index, _) = self
                .core
                .swapchain_device
                .acquire_next_image(
                    self.core.swapchain,
                    u64::MAX,
                    self.image_available_semaphores[frame],
                    vk::Fence::null(),
                )
                .unwrap();

            self.core
                .device
                .reset_fences(std::slice::from_ref(&self.in_flight_fences[frame]))
                .unwrap();

            self.core
                .device
                .reset_command_buffer(
                    self.core.command_buffers[frame],
                    vk::CommandBufferResetFlags::empty(),
                )
                .unwrap();

            self.record_command_buffer(
                &self.core.command_buffers[frame],
                image_index,
                frame,
                &images,
            );

            let signal_semaphores = [self.render_finished_semaphores[frame]];
            let wait_semaphores = [self.image_available_semaphores[frame]];

            let submit_info = vk::SubmitInfo::default()
                .wait_semaphores(&wait_semaphores)
                .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
                .command_buffers(std::slice::from_ref(&self.core.command_buffers[frame]))
                .signal_semaphores(&signal_semaphores);

            self.core
                .device
                .queue_submit(
                    self.core.queues.graphics,
                    &[submit_info],
                    self.in_flight_fences[frame],
                )
                .expect("failed to submit draw command buffer");

            let present_info = vk::PresentInfoKHR::default()
                .wait_semaphores(&signal_semaphores)
                .swapchains(std::slice::from_ref(&self.core.swapchain))
                .image_indices(std::slice::from_ref(&image_index));

            self.core
                .swapchain_device
                .queue_present(self.core.queues.graphics, &present_info)
                .expect("failed to aquire swapchain image");
        }

        //println!("{}, {}", self.frame, frame);
        self.frame += 1;
    }

    fn populate_vertex_index_buffers(
        &mut self,
        sprites: impl Iterator<Item = ((usize, usize), Vector2, bool, f32)>,
        frame: usize,
    ) {
        let quads: Vec<_> = sprites
            .map(|((width, height), position, flipped, depth)| {
                [
                    (
                        (
                            position.x - (width as f32 / 2.0),
                            position.y - (height as f32 / 2.0),
                            depth,
                        ),
                        (0.0, 0.0),
                    ),
                    (
                        (
                            position.x - (width as f32 / 2.0),
                            position.y + (height as f32 / 2.0),
                            depth,
                        ),
                        (0.0, 1.0),
                    ),
                    (
                        (
                            position.x + (width as f32 / 2.0),
                            position.y + (height as f32 / 2.0),
                            depth,
                        ),
                        (1.0, 1.0),
                    ),
                    (
                        (
                            position.x + (width as f32 / 2.0),
                            position.y - (height as f32 / 2.0),
                            depth,
                        ),
                        (1.0, 0.0),
                    ),
                ]
                .into_iter()
                .map(move |(vertex, uv)| {
                    if flipped {
                        (
                            vertex,
                            (
                                uv.0,
                                match uv.1 {
                                    0.0 => 1.0,
                                    1.0 => 0.0,
                                    _ => unreachable!(),
                                },
                            ),
                        )
                    } else {
                        (vertex, uv)
                    }
                })
            })
            .flatten()
            .collect();

        let indices: Vec<_> = (0..(quads.len() / 4))
            .map(|i| {
                [0, 1, 3, 3, 1, 2]
                    .into_iter()
                    .map(move |v| v + i * vertices::INDICES_PER_QUAD)
                    .map(|v| v as u16)
            })
            .flatten()
            .collect();

        self.vertex_buffers.write_vertices(&quads, frame);
        self.vertex_buffers.write_indices(&indices, frame);
    }
}

fn find_render_pass_depth_format(
    instance: &ash::Instance,
    physical_device: &vk::PhysicalDevice,
) -> vk::Format {
    let formats = [
        vk::Format::D32_SFLOAT,
        vk::Format::D32_SFLOAT_S8_UINT,
        vk::Format::D24_UNORM_S8_UINT,
    ];
    let tiling = vk::ImageTiling::OPTIMAL;
    let features = vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT;

    for format in formats {
        let properties =
            unsafe { instance.get_physical_device_format_properties(*physical_device, format) };

        if tiling == vk::ImageTiling::LINEAR
            && (properties.linear_tiling_features & features) == features
        {
            return format;
        } else if tiling == vk::ImageTiling::OPTIMAL
            && (properties.optimal_tiling_features & features) == features
        {
            return format;
        }
    }
    panic!("could not find supported depth format");
}
