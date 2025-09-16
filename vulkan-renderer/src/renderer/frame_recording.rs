use ash;
use ash::vk;

use super::*;

impl Renderer {
    pub fn record_command_buffer(
        &self,
        command_buffer: &vk::CommandBuffer,
        image_index: u32,
        frame: usize,
        sprite_count: usize,
        rect_count: usize,
    ) {
        let begin_info =
            vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::empty());

        unsafe {
            self.core
                .device
                .begin_command_buffer(*command_buffer, &begin_info)
        }
        .expect("failed to begin command buffer");

        const EXTENT: vk::Extent2D = vk::Extent2D {
            width: RENDER_WIDTH,
            height: RENDER_HEIGHT,
        };

        let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.05, 0.05, 0.05, 1.0],
                },
            },
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue::default().depth(1.0).stencil(0),
            },
        ];
        let render_pass_begin_info = vk::RenderPassBeginInfo::default()
            .render_pass(self.render_pass)
            .framebuffer(self.framebuffers[image_index as usize])
            .render_area(
                vk::Rect2D::default()
                    .offset(vk::Offset2D::default())
                    .extent(EXTENT),
            )
            .clear_values(&clear_values);

        unsafe {
            self.core.device.cmd_begin_render_pass(
                *command_buffer,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );

            self.core.device.cmd_bind_pipeline(
                *command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline,
            );

            self.core.device.cmd_set_viewport(
                *command_buffer,
                0,
                std::slice::from_ref(
                    &vk::Viewport::default()
                        .x(0.0)
                        .y(0.0)
                        .width(RENDER_WIDTH as f32)
                        .height(RENDER_HEIGHT as f32)
                        .min_depth(0.0)
                        .max_depth(1.0),
                ),
            );

            self.core.device.cmd_set_scissor(
                *command_buffer,
                0,
                std::slice::from_ref(
                    &vk::Rect2D::default()
                        .offset(vk::Offset2D::default().x(0).y(0))
                        .extent(EXTENT),
                ),
            );

            self.core.device.cmd_bind_vertex_buffers(
                *command_buffer,
                0,
                std::slice::from_ref(self.vertex_buffers.vertex_buffer(frame)),
                &[0],
            );
            self.core.device.cmd_bind_index_buffer(
                *command_buffer,
                *self.vertex_buffers.index_buffer(frame),
                0,
                vk::IndexType::UINT16,
            );

            {
                let (width, height) = (RENDER_WIDTH as f32 / 2.0, RENDER_HEIGHT as f32 / 2.0);
                let ubo = uniforms::UniformMatrix::orthographic_projection(
                    -width, width, -height, height, 0.0, 1.0,
                );
                self.uniform_buffers.write(frame, ubo);

                self.core.device.cmd_bind_descriptor_sets(
                    *command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.pipeline_layout,
                    0,
                    &[
                        self.descriptor_sets[frame],
                        self.sampler_descriptor_sets[frame],
                    ],
                    &[],
                );
            }

            /*println!(
                "drawing {} vertices",
                vertices::INDICES_PER_QUAD * sprite_count
            );*/
            self.core.device.cmd_draw_indexed(
                *command_buffer,
                (vertices::INDICES_PER_QUAD * sprite_count) as u32,
                1,
                0,
                0,
                0,
            );

            self.core.device.cmd_end_render_pass(*command_buffer);

            self.core.device.cmd_pipeline_barrier(
                *command_buffer,
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                vk::PipelineStageFlags::ALL_COMMANDS,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[vk::ImageMemoryBarrier::default()
                    .src_access_mask(vk::AccessFlags::MEMORY_WRITE)
                    .dst_access_mask(vk::AccessFlags::MEMORY_WRITE)
                    .old_layout(vk::ImageLayout::UNDEFINED)
                    .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                    .image(self.core.present_images[image_index as usize].0)
                    .subresource_range(
                        vk::ImageSubresourceRange::default()
                            .aspect_mask(vk::ImageAspectFlags::COLOR)
                            .base_mip_level(0)
                            .level_count(1)
                            .base_array_layer(0)
                            .layer_count(1),
                    )],
            );

            self.core.device.cmd_blit_image(
                *command_buffer,
                self.core.color_image.0 .0,
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                self.core.present_images[image_index as usize].0,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[vk::ImageBlit::default()
                    .src_subresource(
                        vk::ImageSubresourceLayers::default()
                            .aspect_mask(vk::ImageAspectFlags::COLOR)
                            .mip_level(0)
                            .base_array_layer(0)
                            .layer_count(1),
                    )
                    .dst_subresource(
                        vk::ImageSubresourceLayers::default()
                            .aspect_mask(vk::ImageAspectFlags::COLOR)
                            .mip_level(0)
                            .base_array_layer(0)
                            .layer_count(1),
                    )
                    .src_offsets([
                        vk::Offset3D::default().x(0).y(0).z(0),
                        vk::Offset3D::default()
                            .x(RENDER_WIDTH as i32)
                            .y(RENDER_HEIGHT as i32)
                            .z(1),
                    ])
                    .dst_offsets([
                        vk::Offset3D::default().x(0).y(0).z(0),
                        vk::Offset3D::default()
                            .x(self.core.swapchain_info.extent.width as i32)
                            .y(self.core.swapchain_info.extent.height as i32)
                            .z(1),
                    ])],
                vk::Filter::NEAREST,
            );

            self.core.device.cmd_pipeline_barrier(
                *command_buffer,
                vk::PipelineStageFlags::ALL_COMMANDS,
                vk::PipelineStageFlags::ALL_COMMANDS,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[vk::ImageMemoryBarrier::default()
                    .src_access_mask(vk::AccessFlags::MEMORY_WRITE)
                    .dst_access_mask(vk::AccessFlags::MEMORY_WRITE | vk::AccessFlags::MEMORY_READ)
                    .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                    .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                    .image(self.core.present_images[image_index as usize].0)
                    .subresource_range(
                        vk::ImageSubresourceRange::default()
                            .aspect_mask(vk::ImageAspectFlags::COLOR)
                            .base_mip_level(0)
                            .level_count(1)
                            .base_array_layer(0)
                            .layer_count(1),
                    )],
            );

            let render_pass_begin_info = vk::RenderPassBeginInfo::default()
                .render_pass(self.depthless_render_pass)
                .framebuffer(self.depthless_framebuffers[image_index as usize])
                .render_area(
                    vk::Rect2D::default()
                        .offset(vk::Offset2D::default())
                        .extent(self.core.swapchain_info.extent),
                )
                .clear_values(&clear_values);

            self.core.device.cmd_begin_render_pass(
                *command_buffer,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );

            self.core.device.cmd_bind_pipeline(
                *command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.primative_pipeline,
            );

            self.core.device.cmd_set_viewport(
                *command_buffer,
                0,
                std::slice::from_ref(
                    &vk::Viewport::default()
                        .x(0.0)
                        .y(0.0)
                        .width(self.core.swapchain_info.extent.width as f32)
                        .height(self.core.swapchain_info.extent.height as f32)
                        .min_depth(0.0)
                        .max_depth(1.0),
                ),
            );

            self.core.device.cmd_set_scissor(
                *command_buffer,
                0,
                std::slice::from_ref(
                    &vk::Rect2D::default()
                        .offset(vk::Offset2D::default().x(0).y(0))
                        .extent(self.core.swapchain_info.extent),
                ),
            );

            self.core.device.cmd_bind_vertex_buffers(
                *command_buffer,
                0,
                std::slice::from_ref(self.vertex_buffers.vertex_buffer(frame)),
                &[0],
            );
            self.core.device.cmd_bind_index_buffer(
                *command_buffer,
                *self.vertex_buffers.index_buffer(frame),
                0,
                vk::IndexType::UINT16,
            );

            {
                let (width, height) = (RENDER_WIDTH as f32 / 2.0, RENDER_HEIGHT as f32 / 2.0);
                let ubo = uniforms::UniformMatrix::orthographic_projection(
                    -width, width, -height, height, 0.0, 1.0,
                );
                self.uniform_buffers.write(frame, ubo);

                self.core.device.cmd_bind_descriptor_sets(
                    *command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.primative_pipeline_layout,
                    0,
                    &[
                        self.descriptor_sets[frame],
                        self.sampler_descriptor_sets[frame],
                    ],
                    &[],
                );
            }

            /*println!(
                "drawing {} vertices",
                vertices::INDICES_PER_QUAD * sprite_count
            );*/
            self.core.device.cmd_draw_indexed(
                *command_buffer,
                (vertices::INDICES_PER_QUAD * rect_count) as u32,
                1,
                (vertices::INDICES_PER_QUAD * sprite_count) as u32,
                0,
                0,
            );

            self.core.device.cmd_end_render_pass(*command_buffer);

            self.core
                .device
                .end_command_buffer(*command_buffer)
                .expect("failed to record command buffer")
        }
    }
}
