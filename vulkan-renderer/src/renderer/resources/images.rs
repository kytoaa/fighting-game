use super::*;

pub fn create_image(
    device: &Device,
    memory_props: &vk::PhysicalDeviceMemoryProperties,
    size: (usize, usize),
    format: vk::Format,
    tiling: vk::ImageTiling,
    usage: vk::ImageUsageFlags,
    properties: vk::MemoryPropertyFlags,
    aspect_flags: vk::ImageAspectFlags,
) -> Result<VulkanObject<VulkanImage>, &'static str> {
    let image_create_info = vk::ImageCreateInfo::default()
        .image_type(vk::ImageType::TYPE_2D)
        .extent(
            vk::Extent3D::default()
                .width(size.0 as u32)
                .height(size.1 as u32)
                .depth(1),
        )
        .mip_levels(1)
        .array_layers(1)
        .format(format)
        .tiling(tiling)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .samples(vk::SampleCountFlags::TYPE_1)
        .flags(vk::ImageCreateFlags::empty());

    let image = unsafe { device.create_image(&image_create_info, None) }
        .map_err(|_| "error creating image")?;

    let memory_requirements = unsafe { device.get_image_memory_requirements(image) };

    let alloc_info = vk::MemoryAllocateInfo::default()
        .allocation_size(memory_requirements.size)
        .memory_type_index(
            initialization::find_memorytype_index(&memory_requirements, memory_props, properties)
                .ok_or("failed to find suitable memory type")?,
        );

    let memory = unsafe { device.allocate_memory(&alloc_info, None) }
        .map_err(|_| "failed to allocate memory for image")?;

    unsafe { device.bind_image_memory(image, memory, 0) }
        .map_err(|_| "error binding image memory")?;

    let image_view = create_image_view(device, &image, format, aspect_flags)?;

    Ok(VulkanObject(VulkanImage(image, image_view), memory))
}

pub fn create_image_view(
    device: &Device,
    image: &vk::Image,
    format: vk::Format,
    aspect_flags: vk::ImageAspectFlags,
) -> Result<vk::ImageView, &'static str> {
    let image_view_create_info = vk::ImageViewCreateInfo::default()
        .image(*image)
        .view_type(vk::ImageViewType::TYPE_2D)
        .format(format)
        .subresource_range(
            vk::ImageSubresourceRange::default()
                .aspect_mask(aspect_flags)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1),
        );

    unsafe { device.create_image_view(&image_view_create_info, None) }
        .map_err(|_| "error creating image view")
}
