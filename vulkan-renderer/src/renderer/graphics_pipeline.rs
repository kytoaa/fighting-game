use ash::vk;
use ash::Device;

use super::*;

pub fn create_graphics_pipeline(
    core: &CoreRenderData,
    render_pass: &vk::RenderPass,
    descriptor_set_layouts: &[vk::DescriptorSetLayout],
) -> (vk::Pipeline, vk::PipelineLayout) {
    let vert_shader = shaders::HELLO_TRIANGLE_VERT_SHADER;
    let frag_shader = shaders::HELLO_TRIANGLE_FRAG_SHADER;

    let vert_shader_module = create_shader_module(&core.device, vert_shader);
    let frag_shader_module = create_shader_module(&core.device, frag_shader);

    let vert_shader_stage_info = vk::PipelineShaderStageCreateInfo::default()
        .stage(vk::ShaderStageFlags::VERTEX)
        .module(vert_shader_module)
        .name(c"main");

    let frag_shader_stage_info = vk::PipelineShaderStageCreateInfo::default()
        .stage(vk::ShaderStageFlags::FRAGMENT)
        .module(frag_shader_module)
        .name(c"main");

    let shader_stages = [vert_shader_stage_info, frag_shader_stage_info];

    let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
    let dynamic_state_stage_info =
        vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);

    let vertex_input_binding_description = vk::VertexInputBindingDescription::default()
        .binding(0)
        .stride(vertices::VERTEX_DATA_SIZE as u32)
        .input_rate(vk::VertexInputRate::VERTEX);
    let vertex_input_attribute_descriptions = [
        vk::VertexInputAttributeDescription::default()
            .binding(0)
            .location(0)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(std::mem::offset_of!(vertices::VertexData, 0) as u32),
        vk::VertexInputAttributeDescription::default()
            .binding(0)
            .location(1)
            .format(vk::Format::R32G32_SFLOAT)
            .offset(std::mem::offset_of!(vertices::VertexData, 1) as u32),
        vk::VertexInputAttributeDescription::default()
            .binding(0)
            .location(2)
            .format(vk::Format::R32_UINT)
            .offset(std::mem::offset_of!(vertices::VertexData, 2) as u32),
    ];
    let vertex_input_info_stage = vk::PipelineVertexInputStateCreateInfo::default()
        .vertex_binding_descriptions(std::slice::from_ref(&vertex_input_binding_description))
        .vertex_attribute_descriptions(&vertex_input_attribute_descriptions);

    let input_assembly_info_stage = vk::PipelineInputAssemblyStateCreateInfo::default()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
        .primitive_restart_enable(false);

    let viewport_state = vk::PipelineViewportStateCreateInfo::default()
        .viewport_count(1)
        .scissor_count(1);

    let rasterizer_state = vk::PipelineRasterizationStateCreateInfo::default()
        .depth_clamp_enable(false)
        .rasterizer_discard_enable(false)
        .polygon_mode(vk::PolygonMode::FILL)
        .line_width(1.0)
        .cull_mode(vk::CullModeFlags::NONE)
        .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
        .depth_bias_enable(false)
        .depth_bias_constant_factor(0.0)
        .depth_bias_clamp(0.0)
        .depth_bias_slope_factor(0.0);

    let multisampling_state = vk::PipelineMultisampleStateCreateInfo::default()
        .sample_shading_enable(false)
        .rasterization_samples(vk::SampleCountFlags::TYPE_1);

    let colorblend_attachment_state = vk::PipelineColorBlendAttachmentState::default()
        .color_write_mask(
            vk::ColorComponentFlags::R
                | vk::ColorComponentFlags::G
                | vk::ColorComponentFlags::B
                | vk::ColorComponentFlags::A,
        )
        .blend_enable(false);
    let colorblend_state = vk::PipelineColorBlendStateCreateInfo::default()
        .logic_op_enable(false)
        .attachments(std::slice::from_ref(&colorblend_attachment_state));

    let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo::default()
        .depth_test_enable(true)
        .depth_write_enable(true)
        .depth_compare_op(vk::CompareOp::LESS)
        .depth_bounds_test_enable(false)
        .stencil_test_enable(false);

    let pipeline_layout_create_info =
        vk::PipelineLayoutCreateInfo::default().set_layouts(descriptor_set_layouts);

    let pipeline_layout = unsafe {
        core.device
            .create_pipeline_layout(&pipeline_layout_create_info, None)
    }
    .expect("failed to create graphics pipeline");

    let pipeline_create_info = vk::GraphicsPipelineCreateInfo::default()
        .stages(&shader_stages)
        .vertex_input_state(&vertex_input_info_stage)
        .input_assembly_state(&input_assembly_info_stage)
        .viewport_state(&viewport_state)
        .rasterization_state(&rasterizer_state)
        .multisample_state(&multisampling_state)
        .color_blend_state(&colorblend_state)
        .dynamic_state(&dynamic_state_stage_info)
        .depth_stencil_state(&depth_stencil_state)
        .layout(pipeline_layout)
        .render_pass(*render_pass)
        .subpass(0)
        .base_pipeline_handle(vk::Pipeline::null())
        .base_pipeline_index(-1);

    let pipeline = unsafe {
        core.device.create_graphics_pipelines(
            vk::PipelineCache::null(),
            &[pipeline_create_info],
            None,
        )
    }
    .expect("failed to create graphics pipeline")[0];

    unsafe {
        core.device.destroy_shader_module(vert_shader_module, None);
        core.device.destroy_shader_module(frag_shader_module, None);
    }

    (pipeline, pipeline_layout)
}

fn create_shader_module(device: &Device, code: &[u32]) -> vk::ShaderModule {
    let shader_module_create_info = vk::ShaderModuleCreateInfo::default().code(code);

    unsafe { device.create_shader_module(&shader_module_create_info, None) }.unwrap()
}
