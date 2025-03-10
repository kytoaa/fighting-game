use ash::{vk, Device, Instance};

mod deinitialization;
mod frame_recording;
mod graphics_pipeline;
mod initialization;
mod render_pass;
mod shaders;

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

    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    in_flight_fences: Vec<vk::Fence>,

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

        let (pipeline, pipeline_layout) =
            graphics_pipeline::create_graphics_pipeline(&core, &render_pass);

        let (image_available_semaphores, render_finished_semaphores, in_flight_fences) = {
            let semaphore_create_info = vk::SemaphoreCreateInfo::default();
            let fence_create_info =
                vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);

            (
                (0..MAX_FRAMES_IN_FLIGHT)
                    .map(|_| {
                        unsafe { core.device.create_semaphore(&semaphore_create_info, None) }
                            .expect("failed to create semaphore")
                    })
                    .collect(),
                (0..MAX_FRAMES_IN_FLIGHT)
                    .map(|_| {
                        unsafe { core.device.create_semaphore(&semaphore_create_info, None) }
                            .expect("failed to create semaphore")
                    })
                    .collect(),
                (0..MAX_FRAMES_IN_FLIGHT)
                    .map(|_| {
                        unsafe { core.device.create_fence(&fence_create_info, None) }
                            .expect("failed to create fence")
                    })
                    .collect(),
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

            frame: 0,
        })
    }
}

impl Renderer {
    pub fn draw_sprite(&mut self, sprite: SpriteHandle, position: Vector2, flipped: bool) {
        todo!();
    }

    pub fn draw_frame(&mut self) {
        let frame = self.frame as usize % MAX_FRAMES_IN_FLIGHT;

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

            self.record_command_buffer(&self.core.command_buffers[frame], image_index);

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
