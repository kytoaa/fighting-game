use super::*;

impl Drop for CoreRenderData {
    fn drop(&mut self) {
        unsafe {
            self.device.device_wait_idle().unwrap();

            self.device.destroy_image_view(self.depth_image.0 .1, None);
            self.device.destroy_image(self.depth_image.0 .0, None);
            self.device.free_memory(self.depth_image.1, None);

            self.device.destroy_image_view(self.color_image.0 .1, None);
            self.device.destroy_image(self.color_image.0 .0, None);
            self.device.free_memory(self.color_image.1, None);

            self.present_images.iter().for_each(|image| {
                // NOTE: image views get destroyed but not the images
                self.device.destroy_image_view(image.1, None);
            });
            self.swapchain_device
                .destroy_swapchain(self.swapchain, None);

            self.device
                .free_command_buffers(self.command_pool, &self.command_buffers);

            self.device.destroy_command_pool(self.command_pool, None);
            self.device.destroy_command_pool(self.transfer_pool, None);

            self.surface_instance.destroy_surface(self.surface, None);

            self.device.destroy_device(None);

            self.debug_utils_instance
                .destroy_debug_utils_messenger(self.debug_callback, None);

            self.instance.destroy_instance(None);
        }
    }
}
impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            self.core.device.device_wait_idle().unwrap();

            self.core.device.destroy_sampler(self.sampler, None);
            self.images.values().for_each(|image| {
                self.core.device.destroy_image_view(image.0 .1, None);
                self.core.device.destroy_image(image.0 .0, None);
                self.core.device.free_memory(image.1, None);
            });

            self.vertex_buffers.free_buffers(&self.core);
            self.uniform_buffers.free_buffers(&self.core);

            self.core
                .device
                .destroy_descriptor_set_layout(self.descriptor_set_layout, None);
            self.core
                .device
                .destroy_descriptor_pool(self.descriptor_pool, None);
            self.core
                .device
                .destroy_descriptor_set_layout(self.sampler_descriptor_set_layout, None);
            self.core
                .device
                .destroy_descriptor_pool(self.sampler_descriptor_pool, None);

            self.image_available_semaphores
                .iter()
                .for_each(|semaphore| self.core.device.destroy_semaphore(*semaphore, None));
            self.render_finished_semaphores
                .iter()
                .for_each(|semaphore| self.core.device.destroy_semaphore(*semaphore, None));
            self.in_flight_fences
                .iter()
                .for_each(|fence| self.core.device.destroy_fence(*fence, None));

            self.framebuffers
                .iter()
                .chain(self.depthless_framebuffers.iter())
                .for_each(|framebuffer| self.core.device.destroy_framebuffer(*framebuffer, None));

            self.core.device.destroy_render_pass(self.render_pass, None);
            self.core
                .device
                .destroy_render_pass(self.depthless_render_pass, None);
            self.core
                .device
                .destroy_pipeline_layout(self.pipeline_layout, None);
            self.core.device.destroy_pipeline(self.pipeline, None);
            self.core
                .device
                .destroy_pipeline_layout(self.primative_pipeline_layout, None);
            self.core
                .device
                .destroy_pipeline(self.primative_pipeline, None);
        }
    }
}
