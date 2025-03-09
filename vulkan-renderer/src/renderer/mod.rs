use ash::{vk, Device, Entry, Instance};

mod graphics_pipeline;
mod initialization;
mod render_pass;
mod shaders;

use super::Vector2;
use asset_manager::SpriteHandle;

struct VulkanImage(vk::Image, vk::ImageView);
struct VulkanObject<T>(T, vk::DeviceMemory);

struct Queues {
    graphics: vk::Queue,
    transfer: vk::Queue,
}

struct SwapchainInfo {
    format: vk::Format,
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

    depth_image: VulkanObject<VulkanImage>,

    debug_callback: vk::DebugUtilsMessengerEXT,
    debug_utils_instance: ash::ext::debug_utils::Instance,
}

pub struct Renderer {
    core: CoreRenderData,

    render_pass: vk::RenderPass,
}

impl Renderer {
    pub fn init(
        display_handle: &dyn winit::raw_window_handle::HasDisplayHandle,
        window_handle: &dyn winit::raw_window_handle::HasWindowHandle,
        window_size: (u32, u32),
    ) -> Result<Renderer, Box<dyn std::error::Error>> {
        let core = CoreRenderData::init(display_handle, window_handle, window_size);

        let render_pass = render_pass::create_render_pass(&core);

        Ok(Renderer { core, render_pass })
    }
}

impl Renderer {
    pub fn draw_sprite(&mut self, sprite: SpriteHandle, position: Vector2, flipped: bool) {
        todo!();
    }
}
