use ash;
use ash::vk;

use super::*;

pub const MAX_IMAGE_SAMPLERS: usize = 64;

pub fn create_descriptor_pool(core: &CoreRenderData) -> vk::DescriptorPool {
    let pool_sizes = [
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(MAX_FRAMES_IN_FLIGHT as u32),
        /*vk::DescriptorPoolSize::default()
        .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count(MAX_FRAMES_IN_FLIGHT as u32),*/
    ];

    let pool_create_info = vk::DescriptorPoolCreateInfo::default()
        .pool_sizes(&pool_sizes)
        .max_sets(MAX_FRAMES_IN_FLIGHT as u32);

    unsafe { core.device.create_descriptor_pool(&pool_create_info, None) }
        .expect("failed to create descriptor pool")
}

pub fn create_descriptor_set_layout(core: &CoreRenderData) -> vk::DescriptorSetLayout {
    let bindings = [vk::DescriptorSetLayoutBinding::default()
        .binding(0)
        .descriptor_count(1)
        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
        .stage_flags(vk::ShaderStageFlags::VERTEX)];

    let layout_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);

    unsafe { core.device.create_descriptor_set_layout(&layout_info, None) }
        .expect("failed to create descriptor set layout")
}

pub fn create_descriptor_sets(
    core: &CoreRenderData,
    pool: &vk::DescriptorPool,
    layout: &vk::DescriptorSetLayout,
    uniforms: &[vk::Buffer],
) -> [vk::DescriptorSet; MAX_FRAMES_IN_FLIGHT] {
    let layouts = [(); MAX_FRAMES_IN_FLIGHT].map(|_| layout.clone());

    let alloc_info = vk::DescriptorSetAllocateInfo::default()
        .descriptor_pool(*pool)
        .set_layouts(&layouts);

    let descriptor_sets = {
        let mut descriptor_sets = unsafe { core.device.allocate_descriptor_sets(&alloc_info) }
            .expect("failed to allocate for descriptor sets")
            .into_iter();
        [(); MAX_FRAMES_IN_FLIGHT].map(|_| descriptor_sets.next().unwrap())
    };

    for i in 0..MAX_FRAMES_IN_FLIGHT {
        let buffer_info = vk::DescriptorBufferInfo::default()
            .buffer(uniforms[i])
            .offset(0)
            .range(size_of::<uniforms::UniformMatrix>() as u64);

        /*let image_info = vk::DescriptorImageInfo::default()
        .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
        .image_view(vk::ImageView::null())
        .sampler(*sampler);*/

        let descriptor_writes = [
            vk::WriteDescriptorSet::default()
                .dst_set(descriptor_sets[i])
                .dst_binding(0)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .buffer_info(std::slice::from_ref(&buffer_info)),
            /*vk::WriteDescriptorSet::default()
            .dst_set(descriptor_sets[i])
            .dst_binding(1)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .image_info(std::slice::from_ref(&image_info)),*/
        ];

        unsafe { core.device.update_descriptor_sets(&descriptor_writes, &[]) };
    }

    descriptor_sets
}

pub fn create_sampler_array_descriptor_sets(
    core: &CoreRenderData,
) -> (
    vk::DescriptorPool,
    vk::DescriptorSetLayout,
    [vk::DescriptorSet; MAX_FRAMES_IN_FLIGHT],
) {
    let pool_sizes = [vk::DescriptorPoolSize::default()
        .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count((MAX_FRAMES_IN_FLIGHT * MAX_IMAGE_SAMPLERS) as u32)];

    let pool_create_info = vk::DescriptorPoolCreateInfo::default()
        .pool_sizes(&pool_sizes)
        .max_sets(MAX_FRAMES_IN_FLIGHT as u32);

    let pool = unsafe { core.device.create_descriptor_pool(&pool_create_info, None) }
        .expect("failed to create descriptor pool");

    let layout_binding = [vk::DescriptorSetLayoutBinding::default()
        .binding(0)
        .descriptor_count(MAX_IMAGE_SAMPLERS as u32)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .stage_flags(vk::ShaderStageFlags::FRAGMENT)];

    let layout_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&layout_binding);

    let layout = unsafe { core.device.create_descriptor_set_layout(&layout_info, None) }
        .expect("failed to create descriptor set layout");

    let alloc_info = vk::DescriptorSetAllocateInfo::default()
        .descriptor_pool(pool)
        .set_layouts(std::slice::from_ref(&layout));

    (
        pool,
        layout,
        [(); MAX_FRAMES_IN_FLIGHT].map(|_| {
            unsafe { core.device.allocate_descriptor_sets(&alloc_info) }
                .expect("failed to allocate for descriptor sets")[0]
        }),
    )
}
impl Renderer {
    pub fn insert_image_views(&mut self, image_views: &[vk::ImageView], frame: usize) {
        if image_views.len() == 0 {
            return;
        }
        if image_views.len() as usize == MAX_IMAGE_SAMPLERS {
            panic!("too many images");
        }
        let image_infos: Vec<_> = image_views
            .iter()
            .map(|image_view| {
                vk::DescriptorImageInfo::default()
                    .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                    .image_view(*image_view)
                    .sampler(self.sampler)
            })
            .collect();

        let writes = [vk::WriteDescriptorSet::default()
            .dst_set(self.sampler_descriptor_sets[frame])
            .dst_binding(0)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(image_infos.len() as u32)
            .image_info(&image_infos)];

        unsafe { self.core.device.update_descriptor_sets(&writes, &[]) };
    }
}
