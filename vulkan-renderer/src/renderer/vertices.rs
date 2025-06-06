use ash;
use ash::vk;

use super::*;

pub const MAX_VERTICES: usize = 400; // max of 100 quads
pub const INDICES_PER_QUAD: usize = 6;
pub const MAX_QUADS: usize = MAX_VERTICES / 4;
pub const MAX_INDICES: usize = INDICES_PER_QUAD * MAX_QUADS;

pub type Vertex = (f32, f32, f32);
pub type VertexData = (Vertex, (f32, f32), u32);
pub const VERTEX_DATA_SIZE: usize = size_of::<VertexData>();
pub const INDEX_SIZE: usize = size_of::<u16>();

pub const VERTEX_BUFFER_DATA_SIZE: usize =
    MAX_VERTICES * VERTEX_DATA_SIZE + MAX_INDICES * INDEX_SIZE;
pub const TOTAL_ALLOCATION_SIZE: usize = VERTEX_BUFFER_DATA_SIZE * MAX_FRAMES_IN_FLIGHT;

pub struct VertexBuffer<const FRAMES: usize> {
    vertex_buffers: [(VulkanObject<vk::Buffer>, *mut std::ffi::c_void); FRAMES],
    index_buffers: [(VulkanObject<vk::Buffer>, *mut std::ffi::c_void); FRAMES],
}

pub fn create_vertex_buffer<const FRAMES: usize>(core: &CoreRenderData) -> VertexBuffer<FRAMES> {
    let device_memory_properties = unsafe {
        core.instance
            .get_physical_device_memory_properties(core.physical_device)
    };
    let vertex_buffers = [(); FRAMES].map(|_| {
        let buffer = resources::create_buffer(
            &core.device,
            &device_memory_properties,
            VERTEX_DATA_SIZE * MAX_VERTICES,
            vk::BufferUsageFlags::TRANSFER_SRC
                | vk::BufferUsageFlags::TRANSFER_DST
                | vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )
        .expect("failed to create vertex buffer");

        let mapped = unsafe {
            core.device.map_memory(
                buffer.1,
                0,
                (VERTEX_DATA_SIZE * MAX_VERTICES) as u64,
                vk::MemoryMapFlags::empty(),
            )
        }
        .unwrap();

        (buffer, mapped)
    });

    let index_buffers = [(); FRAMES].map(|_| {
        let buffer = resources::create_buffer(
            &core.device,
            &device_memory_properties,
            INDEX_SIZE * MAX_INDICES,
            vk::BufferUsageFlags::TRANSFER_SRC
                | vk::BufferUsageFlags::TRANSFER_DST
                | vk::BufferUsageFlags::INDEX_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )
        .expect("failed to create index buffer");

        let mapped = unsafe {
            core.device.map_memory(
                buffer.1,
                0,
                (INDEX_SIZE * MAX_INDICES) as u64,
                vk::MemoryMapFlags::empty(),
            )
        }
        .unwrap();

        (buffer, mapped)
    });

    VertexBuffer {
        vertex_buffers,
        index_buffers,
    }
}

impl<const FRAMES: usize> VertexBuffer<FRAMES> {
    pub fn vertex_buffer(&self, frame: usize) -> &vk::Buffer {
        &self.vertex_buffers[frame].0 .0
    }
    pub fn index_buffer(&self, frame: usize) -> &vk::Buffer {
        &self.index_buffers[frame].0 .0
    }
    pub fn write_vertices(
        &self,
        vertices: &[VertexData],
        primative_vertices: &[(Vertex, [f32; 4])],
        frame: usize,
    ) {
        assert!(frame < MAX_FRAMES_IN_FLIGHT);
        //println!("writing {:?} to vertex buffer", vertices);

        for (i, vert) in vertices.iter().enumerate().take(MAX_VERTICES) {
            unsafe {
                (self.vertex_buffers[frame].1 as *mut VertexData)
                    .add(i)
                    .write(*vert);
            }
        }
        let primative_index = unsafe {
            (self.vertex_buffers[frame].1 as *mut (Vertex, [f32; 4])).add(vertices.len())
        };
        for (i, vert) in primative_vertices.iter().enumerate().take(MAX_VERTICES) {
            unsafe {
                primative_index.add(i).write(*vert);
            }
        }
    }
    pub fn write_indices(&self, indices: &[u16], frame: usize) {
        assert!(frame < MAX_FRAMES_IN_FLIGHT);
        //println!("writing {:?} to index buffer", indices);

        for (i, index) in indices.iter().enumerate().take(MAX_INDICES) {
            unsafe {
                (self.index_buffers[frame].1 as *mut u16)
                    .add(i)
                    .write(*index);
            }
        }
    }

    pub fn free_buffers(&self, core: &CoreRenderData) {
        unsafe {
            self.vertex_buffers
                .iter()
                .chain(self.index_buffers.iter())
                .for_each(|object| {
                    core.device.unmap_memory(object.0 .1);
                    core.device.destroy_buffer(object.0 .0, None);
                    core.device.free_memory(object.0 .1, None);
                });
        }
    }
}
