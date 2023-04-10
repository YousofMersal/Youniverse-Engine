#[cfg(target_os = "windows")]
use ash::extensions::khr::Win32Surface;

#[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
use ash::extensions::khr::XlibSurface;
use ash::vk::{DebugUtilsMessengerEXT, SurfaceKHR};
use ash::{vk, Entry, Instance};

// #[cfg(debug_assertions)]
use ash::extensions::ext::DebugUtils;
use ash::extensions::khr::Surface;
use winit::window::Window;

use super::util::populate_debug_messenger_create_info;

#[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
))]
pub fn required_extensions() -> Vec<*const i8> {
    let mut res = vec![Surface::name().as_ptr(), XlibSurface::name().as_ptr()];

    // If compiled with debug assertions,
    // add debug extensions to Vulkan for debugging
    if cfg!(debug_assertions) {
        res.push(DebugUtils::name().as_ptr())
    }

    res
}

#[cfg(target_os = "windows")]
pub fn required_extensions() -> Vec<*const i8> {
    let mut res = vec![Surface::name().as_ptr(), Win32Surface::name().as_ptr()];

    // If compiled with debug assertions,
    // add debug extensions to Vulkan for debugging
    if cfg!(debug_assertions) {
        res.push(DebugUtils::name().as_ptr())
    }

    res
}

#[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
))]
pub unsafe fn create_platform_surface(
    entry: &Entry,
    instance: &Instance,
    window: &Window,
) -> Result<SurfaceKHR, vk::Result> {
    //! Create a Vulkan surface with specified as an
    //! x11 surface and get the winit window as an x11 window
    //! this function will only compile for a unix target with is neither
    //! android nor maxos. With only leaves *nix OS's such as linux
    //!
    //! # Safety
    //!
    //! This function is marked unsafe due to how it assumes every
    //! argument is instantiated correctly.
    //! Will segfault if entry, instance, or window is sat incorrectly
    // use winit::platform::unix::WindowExtUnix;
    use winit::platform::x11::WindowExtX11;

    let x11_display = window.xlib_display().unwrap();
    let x11_window = window.xlib_window().unwrap();
    let x11_create_info = vk::XlibSurfaceCreateInfoKHR::builder()
        .window(x11_window)
        .dpy(x11_display as *mut vk::Display);

    let xlib_surace_loader = XlibSurface::new(entry, instance);
    xlib_surace_loader.create_xlib_surface(&x11_create_info, None)
}

// pub fn setup_debug_utils(
//     entry: &Entry,
//     instance: &Instance,
//     is_validation_enabled: bool,
// ) -> (DebugUtils, DebugUtilsMessengerEXT) {
//     let debug_utils_loader = DebugUtils::new(entry, instance);

//     if !is_validation_enabled {
//         (debug_utils_loader, DebugUtilsMessengerEXT::null())
//     } else {
//         let messenger = populate_debug_messenger_create_info();

//         let utils_messenger = unsafe {
//             debug_utils_loader
//                 .create_debug_utils_messenger(&messenger, None)
//                 .expect("Debug utils callback failed to be created")
//         };

//         (debug_utils_loader, utils_messenger)
//     }
// }
