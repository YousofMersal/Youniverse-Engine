use std::sync::Arc;

use ash::{
    vk::{self, PhysicalDevice},
    Instance,
};

use crate::utils::vk_to_str;

pub struct QueueFamilyIndices {
    pub graphics_family: Option<u32>,
    pub present_family: Option<u32>,
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

pub fn select_physical_device(instance: &Instance) -> Option<PhysicalDevice> {
    let physical_devices = unsafe {
        let dev = instance
            .enumerate_physical_devices()
            .expect("Could not enumerate physical devices");

        println!("Found {} devices with vulkan support", dev.len());
        dev
    };

    Some(
        *physical_devices
            .iter()
            .filter(|device| is_p_device_suitable(instance, device))
            .map(|device| (device, rate_device_suitability(instance, device)))
            .max_by(|(_, x), (_, y)| x.cmp(y))
            .expect("Could not find any suitable GPU!")
            .0,
    )
}

fn is_p_device_suitable(instance: &Instance, device: &PhysicalDevice) -> bool {
    let device_props = unsafe { instance.get_physical_device_properties(*device) };
    let device_features = unsafe { instance.get_physical_device_features(*device) };
    // let queue_families = find_queue_families(instance, device);

    let is_extensions_supported = check_device_extension_support(instance, device);
    // let swapchain_support = if is_extensions_supported {
    //     let swapchain_support = self.query_spawnchain_support(device);
    //     !swapchain_support.formats.is_empty() && !swapchain_support.present_modes.is_empty()
    // } else {
    //     false
    // };

    match device_props.device_type {
        vk::PhysicalDeviceType::DISCRETE_GPU | vk::PhysicalDeviceType::INTEGRATED_GPU => {
            println!("Found a GPU");

            device_features.geometry_shader > 0
                // && queue_families.is_complete()
                && is_extensions_supported
            // && swapchain_support
        }
        _ => false,
    }
}

fn rate_device_suitability(instance: &Instance, device: &PhysicalDevice) -> i32 {
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

fn check_device_extension_support(instance: &Instance, device: &PhysicalDevice) -> bool {
    let extensions = unsafe {
        instance
            .clone()
            .enumerate_device_extension_properties(*device)
    }
    .expect("Could not enumerate device extension properties");

    let req_extension = get_swap_required_extensions();

    let res = req_extension.iter().all(|extension| {
        extensions
            .iter()
            .map(|elem| vk_to_str(&elem.extension_name))
            .any(|x| x == *extension)
    });

    res
}

// fn find_queue_families(instance: &Instance, device: &PhysicalDevice) -> QueueFamilyIndices {
//     let queue_familys = unsafe { instance.get_physical_device_queue_family_properties(*device) };

//     let mut res = QueueFamilyIndices::new();

//     for (i, family) in queue_familys.iter().enumerate() {
//         if family.queue_count > 0 && family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
//             res.graphics_family = Some(i as u32);
//         }

//         let is_present_support = unsafe {
//             self.surface
//                 .clone()
//                 .unwrap()
//                 .surface_loader
//                 .get_physical_device_surface_support(
//                     *device,
//                     i as u32,
//                     self.surface.clone().unwrap().surface,
//                 )
//                 .unwrap()
//         };

//         if family.queue_count > 0 && is_present_support {
//             res.present_family = Some(i as u32);
//         }

//         if res.is_complete() {
//             break;
//         }
//     }

//     res
// }

// fn query_spawnchain_support(
//     instance: &Instance,
//     physical_device: &vk::PhysicalDevice,
// ) -> SwapChainSupportDetail {
//     let surface_info = self.surface.clone().unwrap();
//     unsafe {
//         let capabilities = surface_info
//             .surface_loader
//             .get_physical_device_surface_capabilities(*physical_device, surface_info.surface)
//             .expect("Failed to query for surface capabilites");

//         let formats = surface_info
//             .surface_loader
//             .get_physical_device_surface_formats(*physical_device, surface_info.surface)
//             .expect("Failed to query for surface formats");

//         let present_modes = surface_info
//             .surface_loader
//             .get_physical_device_surface_present_modes(*physical_device, surface_info.surface)
//             .expect("Failed to query for surface present modes.");

//         SwapChainSupportDetail {
//             capabilities,
//             formats,
//             present_modes,
//         }
//     }
// }

pub fn get_swap_required_extensions() -> Vec<String> {
    ["VK_KHR_swapchain".to_owned()].to_vec()
}
