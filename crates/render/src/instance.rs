use std::{
    ffi::{c_void, CStr, CString},
    ptr,
};

use ash::{
    vk::{self, DebugUtilsMessageSeverityFlagsEXT, DebugUtilsMessageTypeFlagsEXT},
    Entry, Instance,
};

use crate::utils::{check_validation_layer_support, VALIDATION_LAYERS};

/// .
///
/// # Panics
///
/// Panics if .
pub fn init(entry: &Entry, ext: &[*const i8]) -> Instance {
    #[cfg(debug_assertions)]
    assert!(
        check_validation_layer_support(entry),
        "Validation layers requested but none were found!"
    );

    let engine_name =
        CString::new("Youniverse Engine").expect("Could not make the C String for the engine");

    // these unwraps are safe because the environment variables are set by cargo and are guaranteed
    // to be valid
    let major = std::env::var("CARGO_PKG_VERSION_MAJOR")
        .unwrap()
        .parse()
        .unwrap();
    let minor = std::env::var("CARGO_PKG_VERSION_MINOR")
        .unwrap()
        .parse()
        .unwrap();
    let patch = std::env::var("CARGO_PKG_VERSION_PATCH")
        .unwrap()
        .parse()
        .unwrap();

    let app_info = vk::ApplicationInfo::builder()
        .engine_name(&engine_name)
        .engine_version(vk::make_api_version(0, major, minor, patch))
        .api_version(vk::make_api_version(0, 1, 3, 0));

    let tmp: Vec<CString> = VALIDATION_LAYERS
        .iter()
        .map(|s| CString::new(*s).unwrap())
        .collect();

    let layers = if cfg!(debug_assert) {
        tmp.iter().map(|s| s.as_ptr()).collect()
    } else {
        vec![]
    };
    // let ext = crate::raw::enumerate_required_extensions(entry, window);
    // let ext = window.

    let mut create_info = vk::InstanceCreateInfo::builder()
        .application_info(&app_info)
        .enabled_extension_names(ext)
        .enabled_layer_names(&layers);

    create_info.p_next = if cfg!(debug_assert) {
        let debug_info = populate_debug_messenger_create_info();
        std::ptr::addr_of!(debug_info).cast::<c_void>()
    } else {
        ptr::null()
    };

    unsafe {
        entry
            .create_instance(&create_info, None)
            .expect("Could not create instance")
    }
}

#[inline(never)]
pub fn populate_debug_messenger_create_info() -> vk::DebugUtilsMessengerCreateInfoEXT {
    *vk::DebugUtilsMessengerCreateInfoEXT::builder()
        .message_severity(
            DebugUtilsMessageSeverityFlagsEXT::WARNING | DebugUtilsMessageSeverityFlagsEXT::ERROR, // | DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                                                                                                   // | DebugUtilsMessageSeverityFlagsEXT::INFO,
        )
        .message_type(
            DebugUtilsMessageTypeFlagsEXT::GENERAL
                | DebugUtilsMessageTypeFlagsEXT::VALIDATION
                | DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
        )
        .pfn_user_callback(Some(vulkan_debug_utils_callback))
}

unsafe extern "system" fn vulkan_debug_utils_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    message_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut c_void,
) -> vk::Bool32 {
    let serverity = match message_severity {
        DebugUtilsMessageSeverityFlagsEXT::VERBOSE => "[\x1b[34mVerbose\x1b[0m]",
        DebugUtilsMessageSeverityFlagsEXT::INFO => "[\x1b[36mInfo\x1b[0m]",
        DebugUtilsMessageSeverityFlagsEXT::WARNING => "[\x1b[33mWarning\x1b[0m]",
        DebugUtilsMessageSeverityFlagsEXT::ERROR => "[\x1b[31mError\x1b[0m]",
        _ => "[Unknown]",
    };

    let m_type = match message_type {
        DebugUtilsMessageTypeFlagsEXT::VALIDATION => "[Validation]",
        DebugUtilsMessageTypeFlagsEXT::PERFORMANCE => "[Performance]",
        DebugUtilsMessageTypeFlagsEXT::GENERAL => "[General]",
        _ => "[Unknown]",
    };

    let message = CStr::from_ptr((*message_data).p_message);
    println!("[Debug]{}{}{:?}", serverity, m_type, message);

    vk::FALSE
}
