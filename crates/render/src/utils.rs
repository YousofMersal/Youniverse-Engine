use std::ffi::{c_char, CStr};

use ash::Entry;

pub const VALIDATION_LAYERS: [&str; 2] = ["VK_LAYER_KHRONOS_validation", "VK_LAYER_LUNARG_monitor"];
pub const REQUIRED_EXTENSIONS: [&str; 3] =
    ["VK_KHR_swapchain", "VK_EXT_mesh_shader", "VK_KHR_surface"];

pub fn vk_to_str(raw: &[c_char]) -> String {
    let raw = unsafe { CStr::from_ptr(raw.as_ptr()) };

    raw.to_str()
        .expect("Failed to convert CString to String!")
        .to_owned()
}

pub fn check_validation_layer_support(entry: &Entry) -> bool {
    let layer_properties = entry
        .enumerate_instance_layer_properties()
        .expect("failed to enumerate instance layers properties!");

    if layer_properties.is_empty() {
        eprintln!("no available layers.");
        return false;
    }

    println!("instance available layers: ");
    for layer in &layer_properties {
        println!("\t {}", vk_to_str(&layer.layer_name));
    }

    VALIDATION_LAYERS
        .iter()
        .fold(false, |_acc, required_layer| {
            layer_properties
                .iter()
                .map(|ep| vk_to_str(&ep.layer_name))
                .any(|s| required_layer == &s.as_str())
        })
}
