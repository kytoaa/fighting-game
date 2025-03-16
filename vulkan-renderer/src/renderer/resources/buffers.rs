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
