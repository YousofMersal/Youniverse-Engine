use std::sync::Arc;

use ash::{
    vk::{self, PhysicalDevice},
    Device, Instance,
};
use tracing::info;

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
    let mut physical_devices = unsafe {
        let devs = instance
            .enumerate_physical_devices()
            .expect("Could not enumerate physical devices");

        info!("Found {} devices with vulkan support", devs.len());
        devs
    };

    physical_devices.sort_by_key(|device| {
        let device_props = unsafe { instance.get_physical_device_properties(*device) };
        match device_props.device_type {
            vk::PhysicalDeviceType::DISCRETE_GPU => 0,
            vk::PhysicalDeviceType::INTEGRATED_GPU => 1,
            _ => 2,
        }
    });

    // physical_devices.iter().find(|device| {
    //     let device = *device;

    //     let props  = unsafe {instance.get_physical_device_queue_family_properties(device)};
    //     // for (index, family) in
    //     props.iter().filter(|f| f.queue_count > 0).enumerate().fold((None, None), |acc, (idx,
    // fam)| {         acc.0 = None;
    //         acc.1 = None;

    //         if fam.queue_flags.contains(vk::QueueFlags::GRAPHICS) &&
    // fam.queue_flags.contains(vk::QueueFlags::COMPUTE) {             acc.0 = Some(idx);
    //         };

    //         let present_support = unsafe {

    //         };
    //     });
    //     //     {
    //     //     let index = index as u32;

    //     //     if family.queue_flags.contains(vk::QueueFlags::GRAPHICS) &&
    // family.queue_flags.contains(vk::QueueFlags::COMPUTE) && gra     // }
    // });

    Some(
        *physical_devices
            .iter()
            // .filter(|device| is_p_device_suitable(instance, device))
            .map(|device| (device, rate_device_suitability(instance, device)))
            .max_by(|(_, x), (_, y)| x.cmp(y))
            .expect("Could not find any suitable GPU!")
            .0,
    )
}

pub fn create_device(instance: Instance, p_dev: vk::PhysicalDevice) -> vk::Device {
    todo!()
}

// fn is_p_device_suitable(instance: &Instance, device: &PhysicalDevice) -> bool {
//     let device_features = unsafe { instance.get_physical_device_features(*device) };
//     // let queue_families = find_queue_families(instance, device);

//     let is_extensions_supported = check_device_extension_support(instance, device);
//     // let swapchain_support = if is_extensions_supported {
//     //     let swapchain_support = self.query_spawnchain_support(device);
//     //     !swapchain_support.formats.is_empty() && !swapchain_support.present_modes.is_empty()
//     // } else {
//     //     false
//     // };

//     // match device_props.device_type {
//     //     vk::PhysicalDeviceType::DISCRETE_GPU | vk::PhysicalDeviceType::INTEGRATED_GPU => {
//     //         info!("Found a GPU");

//     //         device_features.geometry_shader > 0
//     //             // && queue_families.is_complete()
//     //             && is_extensions_supported
//     //         // && swapchain_support
//     //     }
//     //     _ => false,
//     // }
// }

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

pub fn create_logical_device(instance: &Instance, p_dev: vk::PhysicalDevice) -> Device {
    let mut features13 = vk::PhysicalDeviceVulkan13Features::builder()
        .dynamic_rendering(true)
        .synchronization2(true)
        .build();
    let mut features12 = vk::PhysicalDeviceVulkan12Features::builder()
        .buffer_device_address(true)
        .descriptor_indexing(true)
        .build();
    features12.p_next =
        &mut features13 as *mut vk::PhysicalDeviceVulkan13Features as *mut std::ffi::c_void;
    let mut features = vk::PhysicalDeviceFeatures2::builder()
        .push_next(&mut features12)
        .build();

    let device_info = vk::DeviceCreateInfo::builder()
        // .enabled_extension_names(enumerate_required_extensions())
        // .enabled_features(&mut features)
        .push_next(&mut features)
        .build();

    let device = unsafe {
        let Ok(d) = instance.create_device(p_dev, &device_info, None) else {
            panic!("Could not create device!");
        };
        d
    };

    device
}

pub fn create_physical_devices(instance: &Instance) -> vk::PhysicalDevice {
    let mut features13_check = vk::PhysicalDeviceVulkan13Features::builder().build();
    let mut features12_check = vk::PhysicalDeviceVulkan12Features::builder().build();
    features12_check.p_next =
        &mut features13_check as *mut vk::PhysicalDeviceVulkan13Features as *mut std::ffi::c_void;
    let mut features_check = vk::PhysicalDeviceFeatures2::builder()
        .push_next(&mut features12_check)
        .build();

    unsafe {
        instance
            .enumerate_physical_devices()
            .expect("Physical device error!")
    }
    .iter()
    .find_map(|p_dev| unsafe {
        let p_props = instance.get_physical_device_properties(*p_dev);
        // let p_features = instance.get_physical_device_features(*p_dev);
        instance.get_physical_device_features2(*p_dev, &mut features_check);
        // find a discrete gpu
        match p_props.device_type {
            vk::PhysicalDeviceType::DISCRETE_GPU | vk::PhysicalDeviceType::INTEGRATED_GPU => {
                if features13_check.dynamic_rendering == vk::TRUE
                    || features13_check.synchronization2 == vk::TRUE
                    || features12_check.buffer_device_address == vk::TRUE
                    || features12_check.descriptor_indexing == vk::TRUE
                {
                    Some(*p_dev)
                } else {
                    None
                }
            }
            _ => None,
        }
    })
    .expect("No suitable device found!")
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
