use std::ffi::{c_char, c_void, CStr};

use ash::vk;
use ash::vk::{DebugUtilsMessageSeverityFlagsEXT, DebugUtilsMessageTypeFlagsEXT};

pub fn vk_to_str(raw: &[c_char]) -> String {
    let raw = unsafe { CStr::from_ptr(raw.as_ptr()) };

    raw.to_str()
        .expect("Failed to convert CString to String!")
        .to_owned()
}

#[inline(never)]
pub fn populate_debug_messenger_create_info() -> vk::DebugUtilsMessengerCreateInfoEXT {
    *vk::DebugUtilsMessengerCreateInfoEXT::builder()
        .message_severity(
            DebugUtilsMessageSeverityFlagsEXT::WARNING | DebugUtilsMessageSeverityFlagsEXT::ERROR,
        )
        .message_type(
            DebugUtilsMessageTypeFlagsEXT::GENERAL
                | DebugUtilsMessageTypeFlagsEXT::VALIDATION
                | DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
        )
        .pfn_user_callback(Some(vulkan_debug_utils_callback))
    // vk::DebugUtilsMessengerCreateInfoEXT {
    //     s_type: vk::StructureType::DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT,
    //     message_severity: DebugUtilsMessageSeverityFlagsEXT::WARNING
    //         | DebugUtilsMessageSeverityFlagsEXT::ERROR,
    //     message_type: DebugUtilsMessageTypeFlagsEXT::GENERAL
    //         | DebugUtilsMessageTypeFlagsEXT::VALIDATION
    //         | DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
    //     pfn_user_callback: Some(vulkan_debug_utils_callback),
    //     flags: vk::DebugUtilsMessengerCreateFlagsEXT::empty(),
    //     p_next: std::ptr::null(),
    //     p_user_data: std::ptr::null_mut(),
    // }
}

#[inline(never)]
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
