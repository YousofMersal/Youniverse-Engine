use std::mem::size_of;

use ash::vk::{
    Buffer, BufferCopy, BufferCreateInfo, BufferUsageFlags, CommandBuffer,
    CommandBufferAllocateInfo, CommandBufferBeginInfo, CommandBufferLevel, CommandBufferUsageFlags,
    CommandPool, DeviceMemory, DeviceSize, Fence, MemoryAllocateInfo, MemoryMapFlags,
    MemoryPropertyFlags, MemoryRequirements, PhysicalDeviceMemoryProperties, Queue, SharingMode,
    SubmitInfo,
};
use glam::Mat4;

use super::{
    shaders::{Vertex, VERTS},
    sync::MAX_FRAMES_IN_FLIGHT,
    vk::Vulkan,
};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct UniformBufferObject {
    model: Mat4,
    view: Mat4,
    proj: Mat4,
}

pub struct UniformBufferMem {
    pub buffers: Vec<Buffer>,
    pub mems: Vec<DeviceMemory>,
}

pub struct BufferMem {
    pub buffer: Buffer,
    pub memory: DeviceMemory,
}

impl UniformBufferMem {
    pub fn new(vk: &Vulkan) -> Self {
        let buffer_size = size_of::<UniformBufferObject>();

        let mut buffers = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        let mut buffers_mem = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);

        (0..MAX_FRAMES_IN_FLIGHT).for_each(|_| {
            let (buffer, buffer_mem, _) = create_buffer(
                vk,
                buffer_size as u64,
                BufferUsageFlags::UNIFORM_BUFFER,
                &(MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT),
            );

            buffers.push(buffer);
            buffers_mem.push(buffer_mem);
        });

        Self {
            buffers,
            mems: buffers_mem,
        }
    }
}

impl BufferMem {
    pub fn new(vk: &Vulkan) -> Self {
        let buffer_size = (size_of::<Vertex>() * VERTS.len()) as u64;

        let (stagin_buffer, stagin_memory, _staging_size) = create_buffer(
            vk,
            buffer_size,
            BufferUsageFlags::TRANSFER_SRC,
            &(MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_CACHED),
        );

        unsafe {
            let data_ptr = vk
                .get_device()
                .map_memory(stagin_memory, 0, buffer_size, MemoryMapFlags::empty())
                .expect("Failed to map memory") as *mut Vertex;

            data_ptr.copy_from_nonoverlapping(VERTS.as_ptr(), VERTS.len());

            vk.get_device().unmap_memory(stagin_memory);
        }

        let (buffer, memory, _) = create_buffer(
            vk,
            buffer_size,
            BufferUsageFlags::TRANSFER_DST | BufferUsageFlags::VERTEX_BUFFER,
            &MemoryPropertyFlags::DEVICE_LOCAL,
        );

        copy_buffer(
            vk,
            &vk.get_queues().graphics_queue,
            &stagin_buffer,
            &buffer,
            &buffer_size,
            &vk.get_command_pool(),
        );

        unsafe {
            vk.get_device().destroy_buffer(stagin_buffer, None);
            vk.get_device().free_memory(stagin_memory, None);
        };

        Self { buffer, memory }
    }
}

pub fn create_buffer(
    vk: &Vulkan,
    size: DeviceSize,
    usage: BufferUsageFlags,
    i_mem_props: &MemoryPropertyFlags,
) -> (Buffer, DeviceMemory, u64) {
    let buffer_info = BufferCreateInfo::builder()
        .size(size)
        .usage(usage)
        .sharing_mode(SharingMode::EXCLUSIVE);

    let buffer = unsafe {
        vk.get_device()
            .create_buffer(&buffer_info, None)
            .expect("Could not create Buffer")
    };

    let mem_reqs = unsafe { vk.get_device().get_buffer_memory_requirements(buffer) };

    let mem_props = unsafe {
        vk.get_instance()
            .unwrap()
            .get_physical_device_memory_properties(*vk.get_physical_device())
    };

    let mem = {
        let mem_type = find_memory_type(mem_reqs, i_mem_props, &mem_props);

        let alloc_info = MemoryAllocateInfo::builder()
            .allocation_size(mem_reqs.size)
            .memory_type_index(mem_type);

        unsafe {
            vk.get_device()
                .allocate_memory(&alloc_info, None)
                .expect("Could not allocate memory")
        }
    };

    unsafe {
        vk.get_device()
            .bind_buffer_memory(buffer, mem, 0)
            .expect("Could not bind buffer memory");
    };

    (buffer, mem, mem_reqs.size)
}

pub fn find_memory_type(
    memory_req: MemoryRequirements,
    required_props: &MemoryPropertyFlags,
    mem_properties: &PhysicalDeviceMemoryProperties,
) -> u32 {
    let Some(i) = mem_properties.memory_types.iter().enumerate().find(|(i, mem_type)| {
        memory_req.memory_type_bits & (1 << i) > 0
            && mem_type.property_flags.contains(*required_props)
    }) else {
        panic!("Could not find a suitable memory type");
    };

    i.0 as u32
}

pub fn copy_buffer(
    vk: &Vulkan,
    queue: &Queue,
    src: &Buffer,
    dst: &Buffer,
    size: &DeviceSize,
    command_pool: &CommandPool,
) {
    unsafe {
        let command_buffer = begin_single_time_command(vk);

        let copy_regions = [*BufferCopy::builder().size(*size)];

        vk.get_device()
            .cmd_copy_buffer(command_buffer, *src, *dst, &copy_regions);

        end_single_time_command(
            vk.get_device().as_ref(),
            command_pool,
            command_buffer,
            queue,
        )
    }
}

pub fn begin_single_time_command(vk: &Vulkan) -> ash::vk::CommandBuffer {
    let alloc_info = CommandBufferAllocateInfo::builder()
        .level(CommandBufferLevel::PRIMARY)
        .command_pool(*vk.get_command_pool())
        .command_buffer_count(1);

    let command_buffers = unsafe {
        vk.get_device()
            .allocate_command_buffers(&alloc_info)
            .expect("Failed to allocate command buffer")
    };

    let begin_info =
        CommandBufferBeginInfo::builder().flags(CommandBufferUsageFlags::ONE_TIME_SUBMIT);

    unsafe {
        vk.get_device()
            .begin_command_buffer(command_buffers[0], &begin_info)
            .expect("Could not being command buffers");
    };

    command_buffers[0]
}

pub fn end_single_time_command(
    device: &ash::Device,
    command_pool: &CommandPool,
    command_buffer: CommandBuffer,
    queue: &Queue,
) {
    unsafe {
        device
            .end_command_buffer(command_buffer)
            .expect("Could not end command buffer");

        let submit_info = [*SubmitInfo::builder().command_buffers(&[command_buffer])];

        // TODO: This function segfaults in release mode
        device
            .queue_submit(*queue, &submit_info, Fence::null())
            .expect("Could not submit command buffer queue");

        device
            .queue_wait_idle(*queue)
            .expect("Could not wait for the graphics queue to idle");

        device.free_command_buffers(*command_pool, &[command_buffer]);
    };
}
