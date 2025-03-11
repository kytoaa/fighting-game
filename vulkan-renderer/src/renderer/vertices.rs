use ash;
use ash::vk;

use super::*;

// interleaved vertex and uv, indices

pub const MAX_VERTICES: usize = 400; // max of 100 quads
pub const INDICES_PER_QUAD: usize = 6;
pub const MAX_QUADS: usize = MAX_VERTICES / 4;
pub const MAX_INDICES: usize = INDICES_PER_QUAD * MAX_QUADS;

pub type Vertex = (f32, f32, f32);
pub type VertexData = (Vertex, (f32, f32));
pub const VERTEX_DATA_SIZE: usize = size_of::<VertexData>();
pub const INDEX_SIZE: usize = size_of::<u16>();

pub const VERTEX_BUFFER_DATA_SIZE: usize =
    MAX_VERTICES * VERTEX_DATA_SIZE + MAX_INDICES * INDEX_SIZE;
pub const TOTAL_ALLOCATION_SIZE: usize = VERTEX_BUFFER_DATA_SIZE * MAX_FRAMES_IN_FLIGHT;

pub struct VertexBuffer {
    full_buffer: vk::Buffer,
    memory: vk::DeviceMemory,

    mapped_access: *mut std::ffi::c_void,
}

pub fn create_vertex_buffer(core: &CoreRenderData) -> VertexBuffer {
    let buffer_info = vk::BufferCreateInfo::default()
        .size(TOTAL_ALLOCATION_SIZE as u64)
        .usage(
            vk::BufferUsageFlags::TRANSFER_SRC
                | vk::BufferUsageFlags::TRANSFER_DST
                | vk::BufferUsageFlags::VERTEX_BUFFER
                | vk::BufferUsageFlags::INDEX_BUFFER,
        )
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    let buffer = unsafe { core.device.create_buffer(&buffer_info, None) }.unwrap();

    let memory_requirements = unsafe { core.device.get_buffer_memory_requirements(buffer) };

    let device_memory_properties = unsafe {
        core.instance
            .get_physical_device_memory_properties(core.physical_device)
    };

    let allocation_info = vk::MemoryAllocateInfo::default()
        .allocation_size(memory_requirements.size)
        .memory_type_index(
            initialization::find_memorytype_index(
                &memory_requirements,
                &device_memory_properties,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )
            .unwrap(),
        );

    let memory = unsafe { core.device.allocate_memory(&allocation_info, None) }
        .expect("failed to allocate for vertex and index buffers");

    unsafe { core.device.bind_buffer_memory(buffer, memory, 0) }.unwrap();

    let mapped_memory = unsafe {
        core.device.map_memory(
            memory,
            0,
            TOTAL_ALLOCATION_SIZE as u64,
            vk::MemoryMapFlags::empty(),
        )
    }
    .unwrap();

    VertexBuffer {
        full_buffer: buffer,
        memory,

        mapped_access: mapped_memory,
    }
}

impl VertexBuffer {
    pub fn buffer(&self) -> &vk::Buffer {
        &self.full_buffer
    }
    pub fn write_vertices(&self, vertices: &[VertexData], frame: usize) {
        assert!(frame < MAX_FRAMES_IN_FLIGHT);
        let offset = frame * VERTEX_BUFFER_DATA_SIZE;

        for (i, vert) in vertices.iter().enumerate().take(MAX_VERTICES) {
            unsafe {
                ((self.mapped_access as *mut u8).add(offset) as *mut VertexData)
                    .add(i * VERTEX_DATA_SIZE)
                    .write_unaligned(*vert);
            }
        }
    }
    pub fn write_indices(&self, indices: &[u16], frame: usize) {
        assert!(frame < MAX_FRAMES_IN_FLIGHT);
        let offset = frame * VERTEX_BUFFER_DATA_SIZE + VERTEX_DATA_SIZE * MAX_VERTICES;

        for (i, index) in indices.iter().enumerate().take(MAX_INDICES) {
            unsafe {
                ((self.mapped_access as *mut u8).add(offset) as *mut u16)
                    .add(i * INDEX_SIZE)
                    .write_unaligned(*index);
            }
        }
    }

    pub fn free_buffer(&self, core: &CoreRenderData) {
        unsafe {
            core.device.unmap_memory(self.memory);
            core.device.destroy_buffer(self.full_buffer, None);
            core.device.free_memory(self.memory, None);
        }
    }
}
