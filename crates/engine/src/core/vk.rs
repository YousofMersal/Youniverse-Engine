use std::sync::Arc;

use vulkano::{
    device::{
        physical::{PhysicalDevice, PhysicalDeviceType},
        Device, DeviceCreateInfo, DeviceExtensions, QueueCreateInfo, QueueFlags,
    },
    image::{ImageUsage, SwapchainImage},
    instance::{
        debug::{
            DebugUtilsMessageSeverity, DebugUtilsMessageType, DebugUtilsMessenger,
            DebugUtilsMessengerCreateInfo,
        },
        Instance, InstanceCreateInfo, InstanceExtensions,
    },
    memory::{allocator::StandardMemoryAllocator, DeviceMemory, MemoryAllocateInfo},
    swapchain::{Swapchain, SwapchainCreateInfo},
    VulkanLibrary,
};
use vulkano_win::required_extensions;

use super::window::Window;

pub struct Vulkan {
    instance: Option<Arc<Instance>>,
    lib: Arc<VulkanLibrary>,
    use_validation_layers: bool,
    debug_message: Option<DebugUtilsMessenger>,
    physical_device: Option<(Arc<PhysicalDevice>, u32)>,
    queue: Option<Arc<vulkano::device::Queue>>,
    device: Option<Arc<Device>>,
    swapchain: Option<Arc<Swapchain>>,
    images: Option<Vec<Arc<SwapchainImage>>>,
    mem_alloc: Option<StandardMemoryAllocator>,
}

const VALIDATION_LAYERS: &[&str] = &["VK_LAYER_KHRONOS_validation"];

impl Vulkan {
    pub fn new() -> Self {
        let lib = VulkanLibrary::new().expect("Could not load Vulkan library");

        Self {
            lib,
            use_validation_layers: cfg!(debug_assertions),
            instance: None,
            debug_message: None,
            physical_device: None,
            queue: None,
            device: None,
            swapchain: None,
            images: None,
            mem_alloc: None,
        }
    }

    pub fn set_debug_message(&mut self, debug_message: Option<DebugUtilsMessenger>) {
        self.debug_message = debug_message;
    }

    pub fn get_instance(&self) -> Option<Arc<Instance>> {
        self.instance.as_ref().cloned()
    }

    pub fn select_physical_device(
        &mut self,
        window: &Window,
        device_extensions: &DeviceExtensions,
    ) {
        let int = self.get_instance();

        let Some(int) = int else {
            panic!("Could not create Vulkan instance!");
        };

        let d = int
            .enumerate_physical_devices()
            .expect("Could not enumerate physical devices")
            .filter(|p| p.supported_extensions().contains(device_extensions))
            .filter_map(|p| {
                p.queue_family_properties()
                    .iter()
                    .enumerate()
                    .position(|(i, q)| {
                        q.queue_flags.intersects(QueueFlags::GRAPHICS)
                            && p.surface_support(i as u32, &window.surface)
                                .unwrap_or(false)
                    })
                    .map(|q| (p, q as u32))
            })
            .min_by_key(|(p, _)| match p.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                PhysicalDeviceType::Other => 4,
                _ => 5,
            })
            .expect("No device available");

        self.physical_device = Some(d);
    }

    pub fn set_up_device(&mut self, device_extensions: &DeviceExtensions) {
        if let Some(device) = &self.physical_device {
            let queue_family_index = device.1;
            let (device, mut queues) = Device::new(
                device.0.clone(),
                DeviceCreateInfo {
                    enabled_extensions: *device_extensions,
                    queue_create_infos: vec![QueueCreateInfo {
                        queue_family_index,
                        ..Default::default()
                    }],
                    ..Default::default()
                },
            )
            .expect("Could not create Vulkan device");

            self.device = Some(device);

            self.queue = Some(queues.next().expect("No queue available"));
        } else {
            panic!("No physical device selected!");
        }
    }

    pub fn create_swapchain(&mut self, window: &Window) {
        let (swapchain, images) = {
            let Some(device) = &self.device else {
                    panic!("No device selected!");
                };

            let surface_capabilities = device
                .physical_device()
                .surface_capabilities(&window.surface, Default::default())
                .expect("Could not craete surface capabilities");

            let image_format = Some(
                device
                    .physical_device()
                    .surface_formats(&window.surface, Default::default())
                    .expect("Could not make image format")[0]
                    .0,
            );

            let wind = window
                .surface
                .object()
                .expect("Could not get surface object")
                .downcast_ref::<winit::window::Window>()
                .expect("Could not downcast surface object to window");

            Swapchain::new(
                device.clone(),
                window.surface.clone(),
                SwapchainCreateInfo {
                    min_image_count: surface_capabilities.min_image_count,
                    image_format,
                    image_extent: wind.inner_size().into(),
                    image_usage: ImageUsage::COLOR_ATTACHMENT,
                    composite_alpha: surface_capabilities
                        .supported_composite_alpha
                        .into_iter()
                        .next()
                        .expect("No supported composite alpha"),
                    ..Default::default()
                },
            )
            .expect("Could not create swapchain")
        };

        self.swapchain = Some(swapchain);
        self.images = Some(images);
    }

    pub fn set_mem_alloc(&mut self) {
        if let Some(device) = &self.device {
            self.mem_alloc = Some(StandardMemoryAllocator::new_default(device.clone()));
        } else {
            panic!("Vulkan device not set!\nFailed to create memory allocator!");
        }
    }

    pub fn create_instance(&mut self) {
        if self.is_using_validation_layers() && !self.check_validation_layers() {
            panic!("Validation layers requested, but not available!");
        }

        let lib = self.get_vulkan_library();
        let supported_extensions = lib.supported_extensions();

        println!("Supported extensions: {:?}", supported_extensions);

        let mut info = InstanceCreateInfo::application_from_cargo_toml();
        info.engine_name = Some("TempestForge Engine".to_string());

        if self.is_using_validation_layers() && self.check_validation_layers() {
            info.enabled_layers = VALIDATION_LAYERS
                .iter()
                .map(|layer| String::from(*layer))
                .collect();
        }

        info.enabled_extensions = self.get_required_extensions();

        self.instance =
            Some(Instance::new(self.lib.clone(), info).expect("Could not create Vulkan instance"));
    }

    /// Returns if [`Vulkan`] is using validation layers.
    pub fn is_using_validation_layers(&self) -> bool {
        self.use_validation_layers
    }

    /// Returns vulkan library of this [`Vulkan`].
    pub fn get_vulkan_library(&self) -> Arc<VulkanLibrary> {
        self.lib.clone()
    }

    /// Returns whether this [`Vulkan`] is using validation layers.
    ///
    /// # Panics
    ///
    /// Panics if the validation layers are not available.
    pub fn check_validation_layers(&self) -> bool {
        let mut props: Vec<_> = self
            .lib
            .layer_properties()
            .expect("Could not get layer properties")
            .collect();

        // This has O(M * N) but with a larger constant, constant is neglible in computational time,
        // however this allows for a ton more VALIDATION_LAYERS in the future without compromising performance
        props.retain(|x| !VALIDATION_LAYERS.contains(&x.name()));
        !props.is_empty()
    }

    pub fn get_required_extensions(&self) -> InstanceExtensions {
        let mut extensions = required_extensions(&self.lib);
        if self.use_validation_layers {
            extensions.ext_debug_utils = true;
        }

        extensions
    }

    pub fn create_debug_callback(&self) -> Option<DebugUtilsMessenger> {
        if !self.use_validation_layers {
            return None;
        }

        let msg_tp = DebugUtilsMessengerCreateInfo {
            message_severity: DebugUtilsMessageSeverity::ERROR
                | DebugUtilsMessageSeverity::WARNING
                | DebugUtilsMessageSeverity::INFO
                | DebugUtilsMessageSeverity::VERBOSE,
            message_type: DebugUtilsMessageType::GENERAL
                | DebugUtilsMessageType::VALIDATION
                | DebugUtilsMessageType::PERFORMANCE,
            ..DebugUtilsMessengerCreateInfo::user_callback(Arc::new(|msg| {
                let severity = if msg.severity.intersects(DebugUtilsMessageSeverity::ERROR) {
                    "ERROR"
                } else if msg.severity.intersects(DebugUtilsMessageSeverity::WARNING) {
                    "WARNING"
                } else if msg.severity.intersects(DebugUtilsMessageSeverity::INFO) {
                    "INFO"
                } else if msg.severity.intersects(DebugUtilsMessageSeverity::VERBOSE) {
                    "VERBOSE"
                } else {
                    panic!("no-impl")
                };

                let ty = if msg.ty.intersects(DebugUtilsMessageType::GENERAL) {
                    "GENERAL"
                } else if msg.ty.intersects(DebugUtilsMessageType::VALIDATION) {
                    "VALIDATION"
                } else if msg.ty.intersects(DebugUtilsMessageType::PERFORMANCE) {
                    "PERFORMANCE"
                } else {
                    panic!("no-impl")
                };

                println!(
                    "{} {} {}: {}",
                    msg.layer_prefix.unwrap_or("unknown"),
                    severity,
                    ty,
                    msg.description
                );
            }))
        };

        unsafe {
            DebugUtilsMessenger::new(
                self.get_instance().expect("No instance on Vulkan object"),
                msg_tp,
            )
            .ok()
        }
    }

    pub fn toggle_debug_message(&mut self) {
        if self.debug_message.is_some() {
            self.debug_message = None;
        } else {
            self.debug_message = self.create_debug_callback();
        }
    }
}

impl Default for Vulkan {
    fn default() -> Self {
        Self::new()
    }
}
