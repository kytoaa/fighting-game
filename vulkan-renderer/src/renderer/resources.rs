use ash::vk;
use ash::Device;

use super::*;

pub fn create_buffer(
    device: &Device,
    memory_props: &vk::PhysicalDeviceMemoryProperties,
    size: usize,
    usage: vk::BufferUsageFlags,
    properties: vk::MemoryPropertyFlags,
) -> Result<VulkanObject<vk::Buffer>, &'static str> {
    let buffer_create_info = vk::BufferCreateInfo::default()
        .size(size as u64)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    let buffer = unsafe { device.create_buffer(&buffer_create_info, None) }
        .map_err(|_| "failed to create buffer")?;

    let memory_requirements = unsafe { device.get_buffer_memory_requirements(buffer) };

    let alloc_info = vk::MemoryAllocateInfo::default()
        .allocation_size(memory_requirements.size)
        .memory_type_index(
            initialization::find_memorytype_index(&memory_requirements, memory_props, properties)
                .ok_or("failed to find suitable memory type")?,
        );

    let memory = unsafe { device.allocate_memory(&alloc_info, None) }
        .map_err(|_| "failed to allocate memory for buffer")?;

    unsafe { device.bind_buffer_memory(buffer, memory, 0) }
        .map_err(|_| "error binding buffer memory")?;

    Ok(VulkanObject(buffer, memory))
}

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

pub fn transition_image_layout(
    device: &Device,
    command_pool: &vk::CommandPool,
    queue: &vk::Queue,
    image: &vk::Image,
    old_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout,
) -> Result<(), &'static str> {
    commands::execute_command_batch(device, command_pool, queue, |command_buffer| {
        let barrier = vk::ImageMemoryBarrier::default()
            .old_layout(old_layout)
            .new_layout(new_layout)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(*image)
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1),
            );

        let (barrier, source_stage, destination_stage) = match (old_layout, new_layout) {
            (vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL) => (
                barrier
                    .src_access_mask(vk::AccessFlags::empty())
                    .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE),
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::TRANSFER,
            ),
            (vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL) => (
                barrier
                    .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                    .dst_access_mask(vk::AccessFlags::SHADER_READ),
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
            ),
            _ => return Err("unsupported layout transition"),
        };

        unsafe {
            device.cmd_pipeline_barrier(
                command_buffer,
                source_stage,
                destination_stage,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                std::slice::from_ref(&barrier),
            )
        };

        Ok(())
    })
}

pub fn copy_buffer_to_image(
    device: &Device,
    command_pool: &vk::CommandPool,
    queue: &vk::Queue,
    image: &vk::Image,
    buffer: &vk::Buffer,
    size: (usize, usize),
) -> Result<(), &'static str> {
    commands::execute_command_batch(device, command_pool, queue, |command_buffer| {
        let region = vk::BufferImageCopy::default()
            .buffer_offset(0)
            .buffer_row_length(0)
            .buffer_image_height(0)
            .image_subresource(
                vk::ImageSubresourceLayers::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .mip_level(0)
                    .base_array_layer(0)
                    .layer_count(1),
            )
            .image_offset(vk::Offset3D::default().x(0).y(0).z(0))
            .image_extent(
                vk::Extent3D::default()
                    .width(size.0 as u32)
                    .height(size.1 as u32)
                    .depth(1),
            );

        unsafe {
            device.cmd_copy_buffer_to_image(
                command_buffer,
                *buffer,
                *image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                std::slice::from_ref(&region),
            )
        };

        Ok(())
    })
}
