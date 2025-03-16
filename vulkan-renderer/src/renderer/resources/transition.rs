use super::*;

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
