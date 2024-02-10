use std::{
    collections::HashSet,
    ffi::{c_char, c_void, CString},
    mem::size_of,
    sync::{Arc, Mutex},
    time::Instant,
};

use ash::{
    extensions::{ext::DebugUtils, khr::Surface},
    vk::{
        self, make_api_version, CommandBufferAllocateInfo, CommandBufferLevel,
        CommandPoolCreateFlags, ComponentMapping, CompositeAlphaFlagsKHR, DebugUtilsMessengerEXT,
        DescriptorBufferInfo, DescriptorPoolCreateInfo, DescriptorPoolSize,
        DescriptorSetAllocateInfo, DescriptorSetLayoutBinding, DescriptorType, DeviceCreateInfo,
        DeviceQueueCreateInfo, Extent2D, Framebuffer, ImageAspectFlags, ImageUsageFlags, ImageView,
        ImageViewType, MemoryMapFlags, PhysicalDevice, Queue, RenderPass, ShaderStageFlags,
        SurfaceKHR, SwapchainCreateInfoKHR, WriteDescriptorSet,
    },
    Device, Entry, Instance,
};
use ash_window::*;
use raw_window_handle::HasDisplayHandle;
use winit::event_loop::EventLoop;

use crate::core::util::vk_to_str;

use super::{
    buffer::{IndexBuffer, VertexBuffer},
    sync::{InFlightFrames, MAX_FRAMES_IN_FLIGHT},
};

use super::{
    buffer::{BufferMem, UniformBufferMem, UniformBufferObject},
    pipeline::Pipeline,
    util::populate_debug_messenger_create_info,
    window::Window,
};

pub struct Vulkan {
    instance: Option<Arc<Instance>>,
    entry: Arc<Entry>,
    use_validation_layers: bool,
    debug_message: Option<DebugUtilsMessenger>,
    physical_device: Option<Arc<PhysicalDevice>>,
    queues: Option<Arc<Queues>>,
    device: Option<Arc<Device>>,
    indicies: Option<Arc<QueueFamilyIndices>>,
    swapchain: Option<Arc<SwapChain>>,
    images: Option<Vec<Arc<ImageView>>>,
    render_pass: Option<Arc<RenderPass>>,
    descriptor_set_layout: Option<Arc<vk::DescriptorSetLayout>>,
    pipeline: Option<Arc<Pipeline>>,
    framebuffers: Option<Vec<Arc<vk::Framebuffer>>>,
    command_pool: Option<Arc<vk::CommandPool>>,
    vertex_buffer: Option<Arc<BufferMem>>,
    index_buffer: Option<Arc<BufferMem>>,
    uniform_buffers: Option<Arc<UniformBufferMem>>,
    descriptor_pool: Option<Arc<vk::DescriptorPool>>,
    descriptor_sets: Option<Vec<vk::DescriptorSet>>,
    command_buffers: Option<Vec<vk::CommandBuffer>>,
    sync: Option<Arc<Mutex<InFlightFrames>>>,
    surface: Option<Arc<SurfaceInfo>>,
}

#[derive(Clone)]
pub struct SurfaceInfo {
    pub surface: SurfaceKHR,
    pub surface_loader: Surface,
}

impl SurfaceInfo {
    pub fn new(surface: SurfaceKHR, surface_loader: Surface) -> Self {
        Self {
            surface,
            surface_loader,
        }
    }
}

pub struct Queues {
    pub graphics_queue: Arc<Queue>,
    pub present_queue: Arc<Queue>,
}

pub struct SwapChain {
    pub swapchain_loader: ash::extensions::khr::Swapchain,
    pub swapchain: vk::SwapchainKHR,
    pub swapchain_images: Vec<vk::Image>,
    pub swapchain_format: vk::Format,
    pub swapchain_extent: vk::Extent2D,
}

pub struct QueueFamilyIndices {
    pub graphics_family: Option<u32>,
    pub present_family: Option<u32>,
}

impl Default for QueueFamilyIndices {
    fn default() -> Self {
        Self::new()
    }
}

struct SwapChainSupportDetail {
    capabilities: vk::SurfaceCapabilitiesKHR,
    formats: Vec<vk::SurfaceFormatKHR>,
    present_modes: Vec<vk::PresentModeKHR>,
}

impl QueueFamilyIndices {
    pub fn new() -> QueueFamilyIndices {
        QueueFamilyIndices {
            graphics_family: None,
            present_family: None,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.graphics_family.is_some() && self.present_family.is_some()
    }
}

pub struct DebugUtilsMessenger {
    messenger: DebugUtilsMessengerEXT,
    loader: DebugUtils,
}

const VALIDATION_LAYERS: [&str; 2] = [
    "VK_LAYER_KHRONOS_validation",
    "VK_LAYER_LUNARG_monitor", // "VK_LAYER_KHRONOS_synchronization2",
];

impl Vulkan {
    pub fn new() -> Self {
        let entry = unsafe { Entry::load().expect("Could not load vulkan library") };
        let entry = Arc::new(entry);

        Self {
            entry,
            use_validation_layers: cfg!(debug_assertions),
            instance: None,
            debug_message: None,
            physical_device: None,
            queues: None,
            device: None,
            swapchain: None,
            images: None,
            render_pass: None,
            indicies: None,
            descriptor_set_layout: None,
            pipeline: None,
            framebuffers: None,
            command_pool: None,
            vertex_buffer: None,
            index_buffer: None,
            uniform_buffers: None,
            descriptor_pool: None,
            descriptor_sets: None,
            command_buffers: None,
            surface: None,
            sync: None,
        }
    }

    pub fn get_instance(&self) -> Arc<Instance> {
        self.instance.clone().expect("Could not get instance")
    }

    pub fn get_device(&self) -> Arc<Device> {
        self.device.clone().expect("Could not get device")
    }

    pub fn select_physical_device(&mut self) {
        let int = self.get_instance();

        let physical_devices = unsafe {
            let dev = int
                .enumerate_physical_devices()
                .expect("Could not enumerate physical devices");

            println!("Found {} devices with vulkan support", dev.len());
            dev
        };

        let dev = Some(Arc::new(
            *physical_devices
                .iter()
                .filter(|device| self.is_device_suitable(device))
                .map(|device| (device, self.rate_device_suitability(&int, device)))
                .max_by(|(_, x), (_, y)| x.cmp(y))
                .expect("Could not find any suitable GPU!")
                .0,
        ));
        self.physical_device = dev;
    }

    fn rate_device_suitability(&self, instance: &Instance, device: &PhysicalDevice) -> i32 {
        let device_props = unsafe { instance.get_physical_device_properties(*device) };
        let device_features = unsafe { instance.get_physical_device_features(*device) };

        let mut score = 0;

        // big plus that it's a GPU
        if device_props.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
            score += 1000;
        }

        // small plus that it's a integrated
        if device_props.device_type == vk::PhysicalDeviceType::INTEGRATED_GPU {
            score += 100;
        }

        // add add max image dimensions to score
        score += device_props.limits.max_image_dimension2_d as i32;

        // need geometry shader
        if device_features.geometry_shader == 0 {
            return 0;
        }

        score
    }

    fn is_device_suitable(&self, device: &PhysicalDevice) -> bool {
        let instance = self.get_instance();
        let device_props = unsafe { instance.get_physical_device_properties(*device) };
        let device_features = unsafe { instance.get_physical_device_features(*device) };
        let queue_families = self.find_queue_families(&self.instance.clone().unwrap(), device);

        let is_extensions_supported = self.check_device_extension_support(device);
        let swapchain_support = if is_extensions_supported {
            let swapchain_support = self.query_spawnchain_support(device);
            !swapchain_support.formats.is_empty() && !swapchain_support.present_modes.is_empty()
        } else {
            false
        };

        match device_props.device_type {
            vk::PhysicalDeviceType::DISCRETE_GPU | vk::PhysicalDeviceType::INTEGRATED_GPU => {
                println!("Found a GPU");

                device_features.geometry_shader > 0
                    && queue_families.is_complete()
                    && is_extensions_supported
                    && swapchain_support
            }
            _ => false,
        }
    }

    pub fn make_queues(&mut self) {
        unsafe {
            let graphics = self
                .device
                .clone()
                .expect("No device found")
                .get_device_queue(self.indicies.clone().unwrap().graphics_family.unwrap(), 0);

            let present = self
                .device
                .clone()
                .expect("No device found")
                .get_device_queue(self.indicies.clone().unwrap().present_family.unwrap(), 0);

            let queues = Arc::new(Queues {
                graphics_queue: Arc::new(graphics),
                present_queue: Arc::new(present),
            });

            self.queues = Some(queues);
        }
    }

    fn check_device_extension_support(&self, device: &PhysicalDevice) -> bool {
        let extensions = unsafe {
            self.instance
                .clone()
                .unwrap()
                .enumerate_device_extension_properties(*device)
        }
        .unwrap();

        let req_extension: Vec<&str> = self.get_swap_required_extensions();

        let res = req_extension.iter().all(|extension| {
            extensions
                .iter()
                .map(|elem| vk_to_str(&elem.extension_name))
                .any(|x| x == *extension)
        });

        res
    }

    fn find_queue_families(
        &self,
        instance: &Instance,
        device: &PhysicalDevice,
    ) -> QueueFamilyIndices {
        let queue_familys =
            unsafe { instance.get_physical_device_queue_family_properties(*device) };

        let mut res = QueueFamilyIndices::new();

        for (i, family) in queue_familys.iter().enumerate() {
            if family.queue_count > 0 && family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                res.graphics_family = Some(i as u32);
            }

            let is_present_support = unsafe {
                self.surface
                    .clone()
                    .unwrap()
                    .surface_loader
                    .get_physical_device_surface_support(
                        *device,
                        i as u32,
                        self.surface.clone().unwrap().surface,
                    )
                    .unwrap()
            };

            if family.queue_count > 0 && is_present_support {
                res.present_family = Some(i as u32);
            }

            if res.is_complete() {
                break;
            }
        }

        res
    }

    pub fn create_logical_device(&mut self) {
        let indicies = self.find_queue_families(
            &self.instance.clone().unwrap(),
            &self.physical_device.clone().unwrap(),
        );

        let mut unique_queue_families = HashSet::new();
        unique_queue_families.insert(indicies.graphics_family.unwrap());
        unique_queue_families.insert(indicies.present_family.unwrap());

        let queue_priority = [1.0_f32];

        let queue_infos: Vec<DeviceQueueCreateInfo> = unique_queue_families
            .iter()
            .map(|family| {
                *DeviceQueueCreateInfo::builder()
                    .queue_priorities(&queue_priority)
                    .queue_family_index(*family)
            })
            .collect();

        let physical_device_features = vk::PhysicalDeviceFeatures::builder();

        let enabled_extension_names = vec![ash::extensions::khr::Swapchain::name().as_ptr()];

        let layers: Vec<CString> = if self.is_using_validation_layers() {
            println!("Available validation layers:");

            VALIDATION_LAYERS
                .iter()
                .map(|layer| {
                    println!("{}", &layer);
                    CString::new(*layer).unwrap()
                })
                .collect()
        } else {
            vec![]
        };

        let layers: Vec<*const i8> = layers.iter().map(|s| s.as_ptr()).collect();

        let device_create_info = DeviceCreateInfo::builder()
            .queue_create_infos(&queue_infos)
            .enabled_layer_names(&layers)
            .enabled_extension_names(&enabled_extension_names)
            .enabled_features(&physical_device_features);

        let device = Arc::new(unsafe {
            self.instance
                .clone()
                .unwrap()
                .create_device(
                    *self.physical_device.clone().unwrap(),
                    &device_create_info,
                    None,
                )
                .expect("Could not create logical device")
        });

        self.indicies = Some(Arc::new(indicies));
        self.device = Some(device);
    }

    pub fn create_swapchain(&mut self, window: Arc<Mutex<Window>>) {
        let swapchain_support = self.query_spawchain_support();

        let surface_format = self.choose_swap_surface_format(&swapchain_support.formats);

        let present_mode = self.choose_swap_present_mode(&swapchain_support.present_modes);
        let extent = self.choose_swap_extent(&swapchain_support.capabilities, window);

        let mut image_count = swapchain_support.capabilities.min_image_count + 1;
        if swapchain_support.capabilities.max_image_count != 0
            && image_count > swapchain_support.capabilities.max_image_count
        {
            image_count = swapchain_support.capabilities.max_image_count
        };

        let mut queue_familie_indices = vec![];
        let Some(indicies) = self.indicies.clone() else {
            panic!("No queues found");
        };

        let image_sharing_mode = if indicies.graphics_family != indicies.present_family {
            queue_familie_indices.push(indicies.graphics_family.unwrap());
            queue_familie_indices.push(indicies.present_family.unwrap());
            vk::SharingMode::CONCURRENT
        } else {
            vk::SharingMode::EXCLUSIVE
        };

        let swap_chain_info = SwapchainCreateInfoKHR::builder()
            .surface(self.surface.clone().unwrap().surface)
            .min_image_count(image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_usage(ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(image_sharing_mode)
            .queue_family_indices(&queue_familie_indices)
            .pre_transform(swapchain_support.capabilities.current_transform)
            .composite_alpha(CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true)
            .image_array_layers(1)
            .old_swapchain(vk::SwapchainKHR::null());

        let swapchain_loader =
            ash::extensions::khr::Swapchain::new(&self.get_instance(), &self.get_device());

        let swapchain = unsafe {
            swapchain_loader
                .create_swapchain(&swap_chain_info, None)
                .expect("Failed to create swapchain")
        };

        let swapchain_images = unsafe {
            swapchain_loader
                .get_swapchain_images(swapchain)
                .expect("Failed to get swapchain images!")
        };

        self.swapchain = Some(Arc::new(SwapChain {
            swapchain_loader,
            swapchain,
            swapchain_images,
            swapchain_format: surface_format.format,
            swapchain_extent: extent,
        }));
    }

    pub fn create_instance(&mut self, e_loop: &EventLoop<()>) {
        if self.is_using_validation_layers() && !self.check_validation_layers_support() {
            panic!("Validation layers requested, but not available!");
        }

        let app_name = CString::new("TempestForge").unwrap();
        let engine_name = CString::new("TempestForge Engine").unwrap();
        let version = make_api_version(
            0,
            env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap(),
            env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap(),
            env!("CARGO_PKG_VERSION_PATCH").parse().unwrap(),
        );

        let app_info = vk::ApplicationInfo::builder()
            .application_name(&app_name)
            .application_version(version.to_owned())
            .engine_name(&engine_name)
            .engine_version(version.to_owned())
            .api_version(make_api_version(0, 1, 3, 238));

        let extension_names = self.get_required_extensions(e_loop);

        let layers: Vec<CString> = if self.is_using_validation_layers() {
            VALIDATION_LAYERS
                .iter()
                .map(|layer| CString::new(*layer).unwrap())
                .collect()
        } else {
            vec![]
        };

        let layers: Vec<*const i8> = layers.iter().map(|s| s.as_ptr()).collect();

        let mut create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(&extension_names)
            .enabled_layer_names(&layers);

        create_info.p_next = if self.is_using_validation_layers() {
            let debug_info = populate_debug_messenger_create_info();
            &debug_info as *const vk::DebugUtilsMessengerCreateInfoEXT as *const c_void
        } else {
            std::ptr::null()
        };

        let instance = unsafe {
            self.entry
                .create_instance(&create_info, None)
                .expect("Failed to create Vulkan instance!")
        };
        self.instance = Some(Arc::new(instance));
    }

    /// Returns if [`Vulkan`] is using validation layers.
    pub fn is_using_validation_layers(&self) -> bool {
        self.use_validation_layers
    }

    /// Returns vulkan library of this [`Vulkan`].
    pub fn get_vulkan_entry(&self) -> Arc<Entry> {
        self.entry.clone()
    }

    /// Returns whether this [`Vulkan`] is using validation layers.
    ///
    /// # Panics
    ///
    /// Panics if the validation layers are not available.
    pub fn check_validation_layers_support(&self) -> bool {
        let props = self
            .entry
            .enumerate_instance_layer_properties()
            .expect("Failed to enumerate Instance Layers Properties!");

        if props.is_empty() {
            eprintln!("No validation layers available!");
            false
        } else {
            // This has O(M * N) but with a larger constant, constant is neglible in computational
            // time, however this allows for a ton more VALIDATION_LAYERS in the future
            // without compromising performance
            VALIDATION_LAYERS
                .iter()
                .fold(false, |_acc, required_layer| {
                    props
                        .iter()
                        .map(|ep| vk_to_str(&ep.layer_name))
                        .any(|s| required_layer == &s.as_str())
                })
        }
    }

    pub fn get_swap_required_extensions(&self) -> Vec<&str> {
        ["VK_KHR_swapchain"].to_vec()
    }

    pub fn get_required_extensions(&self, event_loop: &EventLoop<()>) -> Vec<*const c_char> {
        let mut res = enumerate_required_extensions(event_loop.display_handle().unwrap().as_raw())
            .expect("Could not enumerate required extensions")
            .to_vec();

        if self.use_validation_layers {
            res.push(DebugUtils::name().as_ptr());
        }

        res
    }

    pub fn create_and_set_debug_callback(&mut self) {
        if !self.use_validation_layers {
            return;
        }

        let debug_utils_loader = DebugUtils::new(&self.entry, &self.instance.clone().unwrap());

        let utils_messenger = unsafe {
            let messenger = populate_debug_messenger_create_info();

            debug_utils_loader
                .create_debug_utils_messenger(&messenger, None)
                .expect("Debug utils messenger creation failed!")
        };

        let debug_callback = DebugUtilsMessenger {
            messenger: utils_messenger,
            loader: debug_utils_loader,
        };

        self.debug_message = Some(debug_callback);
    }

    fn query_spawnchain_support(
        &self,
        physical_device: &vk::PhysicalDevice,
    ) -> SwapChainSupportDetail {
        let surface_info = self.surface.clone().unwrap();
        unsafe {
            let capabilities = surface_info
                .surface_loader
                .get_physical_device_surface_capabilities(*physical_device, surface_info.surface)
                .expect("Failed to query for surface capabilites");

            let formats = surface_info
                .surface_loader
                .get_physical_device_surface_formats(*physical_device, surface_info.surface)
                .expect("Failed to query for surface formats");

            let present_modes = surface_info
                .surface_loader
                .get_physical_device_surface_present_modes(*physical_device, surface_info.surface)
                .expect("Failed to query for surface present modes.");

            SwapChainSupportDetail {
                capabilities,
                formats,
                present_modes,
            }
        }
    }

    pub fn toggle_debug_message(&mut self) {
        if self.debug_message.is_some() {
            std::mem::drop(self.debug_message.take().unwrap());
        } else {
            self.create_and_set_debug_callback();
        }
    }

    fn query_spawchain_support(&self) -> SwapChainSupportDetail {
        let surface_info = self.surface.clone().unwrap();

        unsafe {
            let capabilities = surface_info
                .surface_loader
                .get_physical_device_surface_capabilities(
                    *self.physical_device.clone().unwrap(),
                    surface_info.surface,
                )
                .expect("Failed to query for surface capabilites");

            let formats = surface_info
                .surface_loader
                .get_physical_device_surface_formats(
                    *self.physical_device.clone().unwrap(),
                    surface_info.surface,
                )
                .expect("Failed to query for surface formats");

            let present_modes = surface_info
                .surface_loader
                .get_physical_device_surface_present_modes(
                    *self.physical_device.clone().unwrap(),
                    surface_info.surface,
                )
                .expect("Failed to query for surface present modes.");

            SwapChainSupportDetail {
                capabilities,
                formats,
                present_modes,
            }
        }
    }

    fn choose_swap_surface_format(
        &self,
        available_formats: &Vec<vk::SurfaceFormatKHR>,
    ) -> vk::SurfaceFormatKHR {
        // check if list contains most widley used R8G8B8A8 format with nonlinear color space
        for format in available_formats {
            if format.format == vk::Format::B8G8R8_SRGB
                && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            {
                return *format;
            }
        }

        *available_formats.first().unwrap()
    }

    fn choose_swap_present_mode(
        &self,
        available_present_modes: &Vec<vk::PresentModeKHR>,
    ) -> vk::PresentModeKHR {
        // check if list contains mailbox mode
        for mode in available_present_modes {
            if *mode == vk::PresentModeKHR::MAILBOX {
                return *mode;
            }
        }

        // if not, return FIFO as guaranteed to be available
        vk::PresentModeKHR::FIFO
    }

    fn choose_swap_extent(
        &self,
        capabilities: &vk::SurfaceCapabilitiesKHR,
        window: Arc<Mutex<Window>>,
    ) -> vk::Extent2D {
        if capabilities.current_extent.width != u32::MAX {
            capabilities.current_extent
        } else {
            let window = window.lock().unwrap();
            let min = capabilities.min_image_extent;
            let max = capabilities.max_image_extent;
            let width = window.dims.unwrap()[0].min(max.width).max(min.width);
            let height = window.dims.unwrap()[1].min(max.height).max(min.height);

            Extent2D { width, height }
        }
    }

    pub fn create_image_views(&mut self) {
        let mut swapchain_image_views = Vec::new();

        let sw = self.swapchain.clone().unwrap();
        for &image in sw.swapchain_images.iter() {
            let imageview_create_info = vk::ImageViewCreateInfo::builder()
                .view_type(ImageViewType::TYPE_2D)
                .format(sw.swapchain_format)
                .components(
                    *ComponentMapping::builder()
                        .r(vk::ComponentSwizzle::IDENTITY)
                        .g(vk::ComponentSwizzle::IDENTITY)
                        .b(vk::ComponentSwizzle::IDENTITY)
                        .a(vk::ComponentSwizzle::IDENTITY),
                )
                .subresource_range(
                    *vk::ImageSubresourceRange::builder()
                        .aspect_mask(ImageAspectFlags::COLOR)
                        .base_mip_level(0)
                        .level_count(1)
                        .base_array_layer(0)
                        .layer_count(1),
                )
                .image(image);

            let imageview = unsafe {
                self.device
                    .clone()
                    .unwrap()
                    .create_image_view(&imageview_create_info, None)
                    .expect("Failed to create image view")
            };
            swapchain_image_views.push(Arc::new(imageview));
        }

        self.images = Some(swapchain_image_views);
    }

    pub fn create_render_pass(&mut self) {
        let color_attachment = [*vk::AttachmentDescription::builder()
            .format(self.swapchain.clone().unwrap().swapchain_format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)];

        let color_attachment_ref = [
            *vk::AttachmentReference::builder().layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
        ];

        let subpass = [*vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_attachment_ref)];

        let dependency = [*vk::SubpassDependency::builder()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)];

        let render_pass_info = vk::RenderPassCreateInfo::builder()
            .attachments(&color_attachment)
            .subpasses(&subpass)
            .dependencies(&dependency);

        self.render_pass = Some(Arc::new(unsafe {
            self.device
                .clone()
                .unwrap()
                .create_render_pass(&render_pass_info, None)
                .expect("unable to create render pass")
        }));
    }

    pub fn create_descriptor_set_layout(&mut self) {
        let bindings = [*DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_type(DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(ShaderStageFlags::VERTEX)];

        let layout_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(&bindings);

        let layout = unsafe {
            self.get_device()
                .create_descriptor_set_layout(&layout_info, None)
                .expect("Failed to create descriptor layout")
        };
        self.descriptor_set_layout = Some(Arc::new(layout));
    }

    pub fn create_framebuffers(&mut self) {
        let images = self.images.clone().expect("No image views");
        let mut framebuffers = Vec::with_capacity(images.len());

        for image in &images {
            let attachments = [*image.as_ref()];

            let framebuffer_info = vk::FramebufferCreateInfo::builder()
                .render_pass(*self.render_pass.clone().unwrap())
                .attachments(&attachments)
                .width(self.swapchain.clone().unwrap().swapchain_extent.width)
                .height(self.swapchain.clone().unwrap().swapchain_extent.height)
                .layers(1);

            let framebuffer = unsafe {
                self.device
                    .clone()
                    .unwrap()
                    .create_framebuffer(&framebuffer_info, None)
                    .expect("Could not create framebuffer")
            };

            framebuffers.push(Arc::new(framebuffer));
        }

        self.framebuffers = Some(framebuffers);
    }

    pub fn create_command_pool(&mut self) {
        let pool_info = vk::CommandPoolCreateInfo::builder()
            .flags(CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(self.indicies.clone().unwrap().graphics_family.unwrap());

        let pool = unsafe {
            self.device
                .clone()
                .unwrap()
                .create_command_pool(&pool_info, None)
                .expect("Failed to create command pool")
        };

        self.command_pool = Some(Arc::new(pool));
    }

    // pub(crate) fn create_texture_image(&self, arg: &str) -> _ {
    //     todo!()
    // }

    pub fn create_vertex_buffer(&mut self) {
        self.vertex_buffer = Some(Arc::new(BufferMem::new::<VertexBuffer>(self)));
    }

    pub(crate) fn create_index_buffer(&mut self) {
        self.index_buffer = Some(Arc::new(BufferMem::new::<IndexBuffer>(self)));
    }

    pub fn create_uniform_buffers(&mut self) {
        self.uniform_buffers = Some(Arc::new(UniformBufferMem::new(self)));
    }

    pub fn create_descriptor_pool(&mut self) {
        let pool_size = [*DescriptorPoolSize::builder()
            .descriptor_count(MAX_FRAMES_IN_FLIGHT as u32)
            .ty(DescriptorType::UNIFORM_BUFFER)];

        let pool_info = DescriptorPoolCreateInfo::builder()
            .pool_sizes(&pool_size)
            .max_sets(MAX_FRAMES_IN_FLIGHT as u32);

        let pool = unsafe {
            self.get_device()
                .create_descriptor_pool(&pool_info, None)
                .expect("Failed to create descriptor pool")
        };

        self.descriptor_pool = Some(Arc::new(pool));
    }

    pub fn create_descriptor_sets(&mut self) {
        let layouts = vec![*self.descriptor_set_layout.clone().unwrap(); MAX_FRAMES_IN_FLIGHT];

        let alloc_info = DescriptorSetAllocateInfo::builder()
            .descriptor_pool(*self.descriptor_pool.clone().unwrap())
            .set_layouts(&layouts);

        let sets = unsafe {
            self.get_device()
                .allocate_descriptor_sets(&alloc_info)
                .expect("Failed to allocate descriptor sets")
        };

        for (i, &descriptor_set) in sets.iter().enumerate() {
            let desc_buffer_info = [*DescriptorBufferInfo::builder()
                .buffer(self.uniform_buffers.as_ref().unwrap().buffers[i])
                .offset(0)
                .range(size_of::<UniformBufferObject>() as u64)];

            let descriptor_write_set = [*WriteDescriptorSet::builder()
                .dst_set(descriptor_set)
                .dst_binding(0)
                .dst_array_element(0)
                .descriptor_type(DescriptorType::UNIFORM_BUFFER)
                .buffer_info(&desc_buffer_info)];

            unsafe {
                self.get_device()
                    .update_descriptor_sets(&descriptor_write_set, &[]);
            }
        }

        self.descriptor_sets = Some(sets);
    }

    pub fn create_command_buffers(&mut self) {
        let alloc_info = CommandBufferAllocateInfo::builder()
            .command_pool(*self.get_command_pool())
            .level(CommandBufferLevel::PRIMARY)
            .command_buffer_count(MAX_FRAMES_IN_FLIGHT as u32);

        self.command_buffers = Some(unsafe {
            self.get_device()
                .allocate_command_buffers(&alloc_info)
                .expect("Failed to allocate command buffers")
        });
    }

    pub fn create_sync_objects(&mut self) {
        self.sync = Some(Arc::new(Mutex::new(InFlightFrames::new(self))));
    }

    pub fn get_swapchain(&self) -> Arc<SwapChain> {
        self.swapchain
            .clone()
            .expect("Could not retrieve the swapchain")
    }

    pub fn get_descriptor_set_layout(&self) -> Arc<vk::DescriptorSetLayout> {
        self.descriptor_set_layout
            .clone()
            .expect("Could not get descript_set")
    }

    pub fn get_render_pass(&self) -> Arc<RenderPass> {
        self.render_pass.clone().expect("Could not get render pass")
    }

    pub fn create_graphics_pipeline(&mut self) {
        let pipeline = Pipeline::new(self);
        self.pipeline = Some(Arc::new(pipeline));
    }

    pub(crate) fn get_physical_device(&self) -> Arc<PhysicalDevice> {
        self.physical_device
            .clone()
            .expect("Could not get physical device")
    }

    pub fn get_command_pool(&self) -> Arc<vk::CommandPool> {
        self.command_pool
            .clone()
            .expect("Could not get command pool")
    }

    pub fn get_queues(&self) -> Arc<Queues> {
        self.queues.clone().expect("Could not get queues")
    }

    /// Cleans up the swapchain of this [`Vulkan`] releasing every resources aquired by it.
    ///
    /// # Panics
    ///
    /// Panics if the swapchain is not initialized.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it is not guaranteed that the swapchain is not in use.
    pub unsafe fn cleanup_swapchain(&mut self) {
        let device = &self.get_device();

        self.get_framebuffers()
            .iter()
            .for_each(|buffer| device.destroy_framebuffer(**buffer, None));

        for image in self.images.clone().unwrap().iter() {
            device.destroy_image_view(**image, None);
        }

        self.get_swapchain()
            .swapchain_loader
            .destroy_swapchain(self.get_swapchain().swapchain, None);
    }

    pub fn update_uniform_buffer(&self, start_time: Instant, dims: &[u32; 2]) {
        let elapsed = Instant::now().duration_since(start_time).as_secs_f32();

        let model = glam::Mat4::from_rotation_z(elapsed * 90.0f32.to_radians());

        let view = glam::Mat4::look_at_rh(
            glam::vec3(2.0, 2.0, 2.0),
            glam::vec3(0.0, 0.0, 0.0),
            glam::vec3(0.0, 0.0, 1.0),
        );

        let aspect = dims[0] as f32 / dims[1] as f32;
        let mut proj = glam::Mat4::perspective_rh_gl(45.0f32.to_radians(), aspect, 0.1, 10.0);

        proj.y_axis *= -1.;

        let ubo = [UniformBufferObject { model, view, proj }];

        unsafe {
            let data_ptr =
                self.get_device()
                    .map_memory(
                        self.uniform_buffers.clone().unwrap().mems[self.get_current_frame_idx()],
                        0,
                        size_of::<UniformBufferObject>() as u64,
                        MemoryMapFlags::empty(),
                    )
                    .expect("Could not map memory") as *mut UniformBufferObject;

            data_ptr.copy_from_nonoverlapping(ubo.as_ptr(), ubo.len());

            self.get_device().unmap_memory(
                self.uniform_buffers.clone().unwrap().mems[self.get_current_frame_idx()],
            )
        }
    }

    pub fn recreate_swapchain(&mut self, window: Arc<Mutex<Window>>) {
        unsafe {
            self.get_device()
                .device_wait_idle()
                .expect("Failed to wait idle");

            self.cleanup_swapchain();
        }

        self.create_swapchain(window);

        self.create_image_views();

        self.create_framebuffers();
    }

    pub fn get_sync(&self) -> Arc<Mutex<InFlightFrames>> {
        self.sync.clone().unwrap()
    }

    pub fn get_command_buffers(&self) -> &Vec<vk::CommandBuffer> {
        self.command_buffers.as_ref().unwrap()
    }

    pub fn get_framebuffers(&self) -> &Vec<Arc<Framebuffer>> {
        self.framebuffers.as_ref().unwrap()
    }

    pub fn get_t_pipeline(&self) -> Arc<Pipeline> {
        let Some(pipeline) = self.pipeline.clone() else {
            panic!("pipeline is not initialized");
        };

        pipeline
    }

    pub fn get_vertex_buffer(&self) -> Arc<BufferMem> {
        self.vertex_buffer.clone().unwrap()
    }

    pub fn get_index_buffer(&self) -> Arc<BufferMem> {
        self.index_buffer.clone().unwrap()
    }

    pub fn get_current_frame_idx(&self) -> usize {
        self.sync.clone().unwrap().lock().unwrap().current_frame
    }

    pub fn get_descriptor_set(&self, idx: usize) -> vk::DescriptorSet {
        self.descriptor_sets.clone().unwrap()[idx]
    }

    pub(crate) fn create_surface(&mut self, window: &Mutex<Window>) {
        let surface = unsafe {
            let window = window.lock().unwrap();
            create_surface(
                &self.get_vulkan_entry(),
                &self.get_instance(),
                window.get_raw_display_handle(),
                window.get_raw_window_handle(),
                None,
            )
            .expect("Could not create surface")
        };

        let surface_loader = Surface::new(&self.get_vulkan_entry(), &self.get_instance());

        let info = SurfaceInfo::new(surface, surface_loader);

        self.surface = Some(Arc::new(info));
    }
}

impl Default for Vulkan {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Vulkan {
    fn drop(&mut self) {
        unsafe {
            self.cleanup_swapchain();

            let d = self.descriptor_pool.as_deref().unwrap();

            self.get_device().destroy_descriptor_pool(*d, None);

            for i in 0..MAX_FRAMES_IN_FLIGHT {
                self.get_device()
                    .destroy_buffer(self.uniform_buffers.as_deref().unwrap().buffers[i], None);
                self.get_device()
                    .free_memory(self.uniform_buffers.as_deref().unwrap().mems[i], None);
            }

            self.get_device()
                .destroy_buffer(self.index_buffer.as_deref().unwrap().buffer, None);
            self.get_device()
                .free_memory(self.index_buffer.as_deref().unwrap().memory, None);

            self.get_device()
                .destroy_buffer(self.vertex_buffer.as_deref().unwrap().buffer, None);
            self.get_device()
                .free_memory(self.vertex_buffer.as_deref().unwrap().memory, None);

            self.get_device().destroy_descriptor_set_layout(
                *self.descriptor_set_layout.as_deref().unwrap(),
                None,
            );

            self.get_device()
                .destroy_pipeline(self.pipeline.as_deref().unwrap().get_pipeline(), None);
            self.get_device()
                .destroy_pipeline_layout(self.pipeline.as_deref().unwrap().get_layout(), None);

            self.get_device()
                .destroy_render_pass(*self.render_pass.as_deref().unwrap(), None);

            for object in &self.sync.as_deref().unwrap().lock().unwrap().sync_objects {
                self.get_device().destroy_fence(object.fence, None);
                self.get_device()
                    .destroy_semaphore(object.image_available_semaphores, None);
                self.get_device()
                    .destroy_semaphore(object.render_finished_semaphore, None);
            }

            self.get_device()
                .destroy_command_pool(*self.command_pool.as_deref().unwrap(), None);

            self.get_device().destroy_device(None);

            if self.is_using_validation_layers() && self.debug_message.is_some() {
                std::mem::drop(self.debug_message.take())
            }

            self.surface
                .clone()
                .unwrap()
                .surface_loader
                .destroy_surface(self.surface.clone().unwrap().surface, None);

            self.instance.as_deref().unwrap().destroy_instance(None);
        }
    }
}

impl Drop for DebugUtilsMessenger {
    fn drop(&mut self) {
        unsafe {
            self.loader
                .destroy_debug_utils_messenger(self.messenger, None);
        }
    }
}
