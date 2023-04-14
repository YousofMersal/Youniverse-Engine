use ash::{
    vk::{
        Format, ShaderModuleCreateInfo, VertexInputAttributeDescription,
        VertexInputBindingDescription, VertexInputRate,
    },
    Device,
};
use glam::{Vec2, Vec3};
use memoffset::offset_of;
use std::{mem::size_of, ops::Deref, sync::Arc};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub pos: Vec2,
    pub color: Vec3,
}

pub struct ShaderModule {
    module: ash::vk::ShaderModule,
    device: Arc<Device>,
}

pub const VERTS: [Vertex; 4] = [
    Vertex {
        pos: glam::Vec2::new(-0.5, -0.5),
        color: Vec3::new(1., 0., 0.),
    },
    Vertex {
        pos: glam::Vec2::new(0.5, -0.5),
        color: Vec3::new(0., 1., 0.),
    },
    Vertex {
        pos: glam::Vec2::new(0.5, 0.5),
        color: Vec3::new(0., 0., 1.),
    },
    Vertex {
        pos: glam::Vec2::new(-0.5, 0.5),
        color: Vec3::new(1., 1., 1.),
    },
];

pub const INDICES: [u32; 6] = [0, 1, 2, 2, 3, 0];

impl ShaderModule {
    pub fn new(device: Arc<Device>, code: &[u8]) -> Self {
        let module = Self::create_shader_module(device.clone(), code);

        Self { module, device }
    }

    pub fn create_shader_module(device: Arc<Device>, code: &[u8]) -> ash::vk::ShaderModule {
        let code = Vec::<u8>::from(code);

        let (prefix, code, suffix) = unsafe { code.align_to::<u32>() };
        if !prefix.is_empty() || !suffix.is_empty() {
            panic!("Shader code is not aligned to u32");
        }

        let module_info = ShaderModuleCreateInfo::builder().code(code);

        unsafe {
            device
                .create_shader_module(&module_info, None)
                .expect("Failed to create shader module!")
        }
    }
}

impl Deref for ShaderModule {
    type Target = ash::vk::ShaderModule;

    fn deref(&self) -> &Self::Target {
        &self.module
    }
}

impl Drop for ShaderModule {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_shader_module(self.module, None);
        }
    }
}

impl Vertex {
    pub fn get_binding_description() -> [VertexInputBindingDescription; 1] {
        [*VertexInputBindingDescription::builder()
            .binding(0)
            .stride(size_of::<Vertex>() as _)
            .input_rate(VertexInputRate::VERTEX)]
    }

    pub fn get_attribute_description() -> [VertexInputAttributeDescription; 2] {
        let position_desc = VertexInputAttributeDescription::builder()
            .binding(0)
            .location(0)
            .format(Format::R32G32_SFLOAT)
            .offset(offset_of!(Self, pos) as u32)
            .build();
        let color_desc = VertexInputAttributeDescription::builder()
            .binding(0)
            .location(1)
            .format(Format::R32G32B32_SFLOAT)
            .offset(offset_of!(Self, color) as u32)
            .build(); // float is 4 bytes then multiply by how many butes in position_desc

        [position_desc, color_desc]
    }
}
