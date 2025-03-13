use ash;
use ash::vk;

use super::*;

/// COLUMN MAJOR NOT ROW MAJOR, THAT MEANS INNER ARRAY IS COLUMN NOT ROW
pub struct UniformMatrix([[f32; 4]; 4]);
impl UniformMatrix {
    pub const fn orthographic_projection(
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    ) -> Self {
        UniformMatrix([
            [2.0 / (right - left), 0.0, 0.0, 0.0],
            [0.0, 2.0 / (top - bottom), 0.0, 0.0],
            [0.0, 0.0, -2.0 / (near - far), 0.0],
            [
                -(right + left) / (right - left),
                -(top + bottom) / (top - bottom),
                -(far + near) / (far - near),
                1.0,
            ],
        ])
    }
}

pub struct UniformBuffer<const FRAMES: usize>(
    [(VulkanObject<vk::Buffer>, *mut std::ffi::c_void); FRAMES],
);
impl<const FRAMES: usize> UniformBuffer<FRAMES> {
    pub fn get_buffers(&self) -> Vec<vk::Buffer> {
        self.0.iter().map(|value| value.0 .0).collect::<Vec<_>>()
    }
    pub unsafe fn write<T>(&self, frame: usize, value: T) {
        (self.0[frame].1 as *mut T).write(value);
    }

    pub fn free_buffers(&self, core: &CoreRenderData) {
        unsafe {
            self.0.iter().for_each(|object| {
                core.device.unmap_memory(object.0 .1);
                core.device.destroy_buffer(object.0 .0, None);
                core.device.free_memory(object.0 .1, None);
            });
        }
    }
}

pub fn create_uniform_buffer<const FRAMES: usize>(core: &CoreRenderData) -> UniformBuffer<FRAMES> {
    let size = size_of::<UniformMatrix>() as u64;

    let device_memory_properties = unsafe {
        core.instance
            .get_physical_device_memory_properties(core.physical_device)
    };

    let buffers = [(); FRAMES].map(|_| {
        let buffer = resources::create_buffer(
            &core.device,
            &device_memory_properties,
            size as usize,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )
        .expect("failed to create uniform buffer");
        let mapped = unsafe {
            core.device
                .map_memory(buffer.1, 0, size as u64, vk::MemoryMapFlags::empty())
        }
        .unwrap();
        (buffer, mapped)
    });

    UniformBuffer(buffers)
}
