use ash::{vk, Device, Entry, Instance};

mod initialization;

use super::Vector2;
use asset_manager::{ShaderHandle, SpriteHandle};

struct VulkanImage(vk::Image, vk::ImageView);
struct VulkanObject<T>(T, vk::DeviceMemory);

struct Queues {
    graphics: vk::Queue,
    transfer: vk::Queue,
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
    present_images: Vec<VulkanImage>,

    command_pool: vk::CommandPool,
    transfer_pool: vk::CommandPool,

    depth_image: VulkanObject<VulkanImage>,

    debug_callback: vk::DebugUtilsMessengerEXT,
    debug_utils_instance: ash::ext::debug_utils::Instance,
}

pub struct Renderer {
    core: CoreRenderData,
}

impl Renderer {
    pub fn init(
        display_handle: &dyn winit::raw_window_handle::HasDisplayHandle,
        window_handle: &dyn winit::raw_window_handle::HasWindowHandle,
        window_size: (u32, u32),
    ) -> Result<Renderer, Box<dyn std::error::Error>> {
        let core = CoreRenderData::init(display_handle, window_handle, window_size);

        Ok(Renderer { core })
    }
}

impl Renderer {
    pub fn draw_sprite(&mut self, sprite: SpriteHandle, position: Vector2, flipped: bool) {
        todo!();
    }
}
