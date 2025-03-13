use ash;
use ash::vk;
use ash::Device;

pub fn execute_command_batch(
    device: &Device,
    command_pool: &vk::CommandPool,
    queue: &vk::Queue,
    f: impl Fn(vk::CommandBuffer) -> Result<(), &'static str>,
) -> Result<(), &'static str> {
    let alloc_info = vk::CommandBufferAllocateInfo::default()
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_pool(*command_pool)
        .command_buffer_count(1);

    let command_buffer = unsafe { device.allocate_command_buffers(&alloc_info) }
        .map_err(|_| "failed to allocate command buffer")?[0];

    let begin_info =
        vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

    unsafe { device.begin_command_buffer(command_buffer, &begin_info) }
        .map_err(|_| "failed to begin command buffer")?;

    f(command_buffer)?;

    unsafe { device.end_command_buffer(command_buffer) }
        .map_err(|_| "failed to end command buffer")?;

    let submit_info =
        vk::SubmitInfo::default().command_buffers(std::slice::from_ref(&command_buffer));

    unsafe {
        device.queue_submit(
            *queue,
            std::slice::from_ref(&submit_info),
            vk::Fence::null(),
        )
    }
    .map_err(|_| "failed to submit queue")?;

    unsafe { device.queue_wait_idle(*queue) }.map_err(|_| "error waiting for queue to idle")?;

    unsafe { device.free_command_buffers(*command_pool, std::slice::from_ref(&command_buffer)) };

    Ok(())
}
