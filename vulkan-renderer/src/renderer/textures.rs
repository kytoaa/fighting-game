use ash;
use ash::vk;

use super::*;

pub fn create_image_sampler(core: &CoreRenderData) -> vk::Sampler {
    let sampler_create_info = vk::SamplerCreateInfo::default()
        .mag_filter(vk::Filter::NEAREST)
        .min_filter(vk::Filter::NEAREST)
        .address_mode_u(vk::SamplerAddressMode::REPEAT)
        .address_mode_v(vk::SamplerAddressMode::REPEAT)
        .address_mode_w(vk::SamplerAddressMode::REPEAT)
        .anisotropy_enable(false)
        .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
        .unnormalized_coordinates(false)
        .compare_enable(false)
        .mipmap_mode(vk::SamplerMipmapMode::NEAREST)
        .mip_lod_bias(0.0)
        .min_lod(0.0)
        .max_lod(0.0);

    unsafe { core.device.create_sampler(&sampler_create_info, None) }
        .expect("failed to create sampler")
}

pub fn create_image(
    core: &CoreRenderData,
    image_data: ((usize, usize), &[u8]),
) -> VulkanObject<VulkanImage> {
    let device_memory_properties = unsafe {
        core.instance
            .get_physical_device_memory_properties(core.physical_device)
    };

    // NOTE: width * height * color depth
    let image_size = image_data.0 .0 * image_data.0 .1 * 4;

    let staging_buffer = resources::create_buffer(
        &core.device,
        &device_memory_properties,
        image_size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    )
    .expect("failed to create image staging buffer");

    unsafe {
        let mapped = core
            .device
            .map_memory(
                staging_buffer.1,
                0,
                image_size as u64,
                vk::MemoryMapFlags::empty(),
            )
            .unwrap() as *mut u8;

        std::ptr::copy_nonoverlapping(image_data.1.as_ptr(), mapped, image_size);
        core.device.unmap_memory(staging_buffer.1); // not necessary
    }

    let image = resources::create_image(
        &core.device,
        &device_memory_properties,
        image_data.0,
        vk::Format::R8G8B8_SRGB,
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
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
    )
    .expect("failed to transition image layout");

    resources::copy_buffer_to_image(
        &core.device,
        &core.transfer_pool,
        &core.queues.transfer,
        &image.0 .0,
        &staging_buffer.0,
        image_data.0,
    )
    .expect("failed to copy buffer to image");

    resources::transition_image_layout(
        &core.device,
        &core.transfer_pool,
        &core.queues.transfer,
        &image.0 .0,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
    )
    .expect("failed to transition image layout");

    unsafe {
        core.device.destroy_buffer(staging_buffer.0, None);
        core.device.free_memory(staging_buffer.1, None);
    }

    image
}
