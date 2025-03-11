use ash;
use ash::vk;

use super::*;

const MAX_VERTICES: usize = 400; // max of 100 quads
const INDICES_PER_QUAD: usize = 6;
const MAX_QUADS: usize = MAX_VERTICES / 4;
const MAX_INDICES: usize = INDICES_PER_QUAD * MAX_QUADS;

type Vertex = (f32, f32, f32);
type VertexData = (Vertex, (f32, f32));
const VERTEX_DATA_SIZE: usize = size_of::<VertexData>();
const INDEX_SIZE: usize = size_of::<u16>();

pub fn create_vertex_buffer(core: &CoreRenderData) -> VertexBuffer {
    let total_allocation_size =
        (MAX_VERTICES * VERTEX_DATA_SIZE + MAX_INDICES * INDEX_SIZE) * MAX_FRAMES_IN_FLIGHT;

    let buffer_info = vk::BufferCreateInfo::default()
        .size(total_allocation_size as u64)
        .usage(vk::BufferUsageFlags::TRANSFER_SRC | vk::BufferUsageFlags::TRANSFER_DST)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    let buffer = unsafe { core.device.create_buffer(&buffer_info, None) }.unwrap();

    let vertex_mem_requirements = unsafe { core.device.get_buffer_memory_requirements(buffer) };

    let device_memory_properties = unsafe {
        core.instance
            .get_physical_device_memory_properties(core.physical_device)
    };

    let allocation_info = vk::MemoryAllocateInfo::default()
        .allocation_size(total_allocation_size as u64)
        .memory_type_index(
            initialization::find_memorytype_index(
                &vertex_mem_requirements,
                &device_memory_properties,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )
            .unwrap(),
        );

    let memory = unsafe { core.device.allocate_memory(&allocation_info, None) }
        .expect("failed to allocate for vertex and index buffers");

    todo!();
}
