use ash;
use ash::vk;

use super::*;

impl Renderer {
    pub fn record_command_buffer(
        &self,
        command_buffer: &vk::CommandBuffer,
        image_index: u32,
        frame: usize,
        vertex_count: usize,
    ) {
        let begin_info =
            vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::empty());

        unsafe {
            self.core
                .device
                .begin_command_buffer(*command_buffer, &begin_info)
        }
        .expect("failed to begin command buffer");

        let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue::default(),
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
                    .extent(self.core.swapchain_info.extent),
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
        }

        unsafe {
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
                std::slice::from_ref(self.vertex_buffer.buffer()),
                &[(frame * vertices::VERTEX_BUFFER_DATA_SIZE) as u64],
            );
            self.core.device.cmd_bind_index_buffer(
                *command_buffer,
                *self.vertex_buffer.buffer(),
                (frame * vertices::VERTEX_BUFFER_DATA_SIZE
                    + vertices::VERTEX_DATA_SIZE * vertices::MAX_VERTICES) as u64,
                vk::IndexType::UINT16,
            );

            self.core
                .device
                .cmd_draw_indexed(*command_buffer, vertex_count as u32, 1, 0, 0, 0);

            self.core.device.cmd_end_render_pass(*command_buffer);

            self.core
                .device
                .end_command_buffer(*command_buffer)
                .expect("failed to record command buffer")
        }
    }
}
