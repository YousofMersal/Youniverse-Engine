use std::sync::Arc;

use vulkano::{
    instance::{Instance, InstanceCreateInfo, InstanceExtensions},
    VulkanLibrary,
};
use vulkano_win::required_extensions;

pub struct Vulkan {
    instance: Option<Arc<Instance>>,
    lib: Arc<VulkanLibrary>,
    use_validation_layers: bool,
}

const VALIDATION_LAYERS: &[&str] = &["VK_LAYER_KHRONOS_validation"];

impl Vulkan {
    pub fn new() -> Self {
        let lib = VulkanLibrary::new().expect("Could not load Vulkan library");

        Self {
            lib,
            use_validation_layers: cfg!(debug_assertions),
            instance: None,
        }
    }

    pub fn get_instance(&self) -> Option<Arc<Instance>> {
        if let Some(instance) = &self.instance {
            Some(instance.clone())
        } else {
            None
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

    pub fn is_using_validation_layers(&self) -> bool {
        self.use_validation_layers
    }

    pub fn get_vulkan_library(&self) -> Arc<VulkanLibrary> {
        self.lib.clone()
    }

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
}
