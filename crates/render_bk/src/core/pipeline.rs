use std::sync::Arc;

use ash::{
    vk::{self, PipelineLayout, PipelineRasterizationStateCreateInfo},
    Device,
};

use super::{
    shaders::{ShaderModule, Vertex},
    vk::Vulkan,
};

#[allow(unused)]
pub struct Pipeline {
    device: Arc<Device>,
    layout: PipelineLayout,
    pipeline: vk::Pipeline,
}

impl Pipeline {
    pub fn new(vk: &Vulkan) -> Self {
        let vert = &read_file("./crates/engine/shaders/spv/default.vert.spv");
        let frag = &read_file("./crates/engine/shaders/spv/default.frag.spv");

        let vert_module = ShaderModule::new(vk.get_device(), vert);
        let frag_module = ShaderModule::new(vk.get_device(), frag);

        let main_function_name = std::ffi::CString::new("main").unwrap();

        let vert_shader_info = vk::PipelineShaderStageCreateInfo::builder()
            .name(&main_function_name)
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(*vert_module);

        let frag_shader_info = vk::PipelineShaderStageCreateInfo::builder()
            .name(&main_function_name)
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(*frag_module);

        let shader_states = [*vert_shader_info, *frag_shader_info];

        let dynamic_states = vec![vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_states_info =
            vk::PipelineDynamicStateCreateInfo::builder().dynamic_states(&dynamic_states);

        let binding_descriptions = Vertex::get_binding_description();
        let attribute_descriptions = Vertex::get_attribute_description();

        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(&binding_descriptions)
            .vertex_attribute_descriptions(&attribute_descriptions);

        let input_assembly_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        let viewports = [*vk::Viewport::builder()
            .x(0.)
            .y(0.)
            .width(vk.get_swapchain().swapchain_extent.width as f32)
            .height(vk.get_swapchain().swapchain_extent.height as f32)
            .min_depth(0.)
            .max_depth(1.)];

        let scissors = [*vk::Rect2D::builder()
            .offset(vk::Offset2D { x: 0, y: 0 })
            .extent(vk.get_swapchain().swapchain_extent)];

        let viewport_state_create_info = vk::PipelineViewportStateCreateInfo::builder()
            .viewports(&viewports)
            .scissors(&scissors);

        let rasterizer = PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .depth_bias_enable(false)
            .line_width(1.);

        let multisampling = vk::PipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);

        let color_blend_attachments = [*vk::PipelineColorBlendAttachmentState::builder()
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .blend_enable(false)];

        let color_blending = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .attachments(&color_blend_attachments);

        let binding = [*vk.get_descriptor_set_layout()];
        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::builder().set_layouts(&binding);

        let pipeline_layout = unsafe {
            vk.get_device()
                .create_pipeline_layout(&pipeline_layout_info, None)
                .expect("Failed to create pipeline layout!")
        };

        let pipeline_info = [*vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_states)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly_info)
            .viewport_state(&viewport_state_create_info)
            .rasterization_state(&rasterizer)
            .multisample_state(&multisampling)
            .color_blend_state(&color_blending)
            .layout(pipeline_layout)
            .render_pass(*vk.get_render_pass())
            .base_pipeline_handle(vk::Pipeline::null())
            .base_pipeline_index(-1)
            .dynamic_state(&dynamic_states_info)];

        let graphics_pipeline = unsafe {
            vk.get_device()
                .create_graphics_pipelines(vk::PipelineCache::null(), &pipeline_info, None)
                .expect("Failed to create graphics pipeline!")
        };

        Self {
            device: vk.get_device(),
            layout: pipeline_layout,
            pipeline: graphics_pipeline[0],
        }
    }

    pub fn get_pipeline(&self) -> vk::Pipeline {
        self.pipeline
    }

    pub fn get_layout(&self) -> PipelineLayout {
        self.layout
    }
}

// Creates this function for use later, but for now shader are read at compile time
#[allow(dead_code)]
fn read_file(path: &str) -> Vec<u8> {
    use std::{fs::File, io::Read, path::PathBuf, str::FromStr};

    let path = PathBuf::from_str(path).expect("Could not create path to shader");

    File::open(path)
        .expect("Could not read file")
        .bytes()
        .filter_map(|byte| byte.ok())
        .collect()
}
