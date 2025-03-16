use ash;
use ash::vk;

use super::*;

pub const RESOLUTION: (usize, usize) = (320, 180);

pub fn create_low_res_framebuffers(
    core: &CoreRenderData,
    render_pass: &vk::RenderPass,
) -> Vec<(vk::Framebuffer, VulkanObject<VulkanImage>)> {
    Vec::from_iter((0..MAX_FRAMES_IN_FLIGHT).map(|_| {
        let device_memory_properties = unsafe {
            core.instance
                .get_physical_device_memory_properties(core.physical_device)
        };

        let image = resources::create_image(
            &core.device,
            &device_memory_properties,
            RESOLUTION,
            core.swapchain_info.format,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            vk::ImageAspectFlags::COLOR,
        )
        .expect("failed to create image");

        resources::transition_image_layout(
            &core.device,
            &core.transfer_pool,
            &core.queues.transfer,
            &image.0 .0,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::PRESENT_SRC_KHR,
        )
        .expect("failed to transition image layout");

        let attachments = [image.0 .1, core.depth_image.0 .1];

        let framebuffer_create_info = vk::FramebufferCreateInfo::default()
            .render_pass(*render_pass)
            .attachments(&attachments)
            .width(RESOLUTION.0 as u32)
            .height(RESOLUTION.1 as u32)
            .layers(1);

        let framebuffer = unsafe {
            core.device
                .create_framebuffer(&framebuffer_create_info, None)
        }
        .expect("failed to create framebuffer");

        (framebuffer, image)
    }))
}
