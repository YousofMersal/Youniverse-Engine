use ash::{
    extensions::khr,
    prelude::*,
    vk::{self, AllocationCallbacks, WaylandSurfaceCreateInfoKHR},
    Entry, Instance,
};
use std::ffi::c_void;
// use winit::raw_window_handle::{RawDisplayHandle, RawWindowHandle};
use winit::raw_window_handle::{DisplayHandle, WindowHandle};

/// Create a surface from a raw surface handle.
/// through [`enumerate_required_extensions()`].
///
/// # Safety
///
/// In order for the created [`vk::SurfaceKHR`] to be valid for the duration of its
/// usage, the [`Instance`] this was called on must be dropped later than the
/// resulting [`vk::SurfaceKHR`].
pub unsafe fn create_surface(
    entry: &Entry,
    instance: &Instance,
    window_handle: WindowHandle,
    display_handle: DisplayHandle,
    alloc_clb: Option<&AllocationCallbacks>,
) -> VkResult<vk::SurfaceKHR> {
    use ash::vk::Win32SurfaceCreateInfoKHR;
    use winit::raw_window_handle::*;

    match (window_handle.as_raw(), display_handle.as_raw()) {
        (RawWindowHandle::Win32(win), RawDisplayHandle::Windows(_)) => {
            let surface_desc = Win32SurfaceCreateInfoKHR {
                hinstance: win
                    .hinstance
                    .expect("Could not get hinstance of surface")
                    .get() as *const c_void,
                hwnd: win.hwnd.get() as *const c_void,
                ..Default::default()
            };
            let surface_fn = khr::Win32Surface::new(entry, instance);
            surface_fn.create_win32_surface(&surface_desc, alloc_clb)
        }
        (RawWindowHandle::Xlib(win), RawDisplayHandle::Xlib(display)) => {
            let surface_desc = vk::XlibSurfaceCreateInfoKHR::builder()
                .dpy(display.display.unwrap().as_ptr().cast())
                .window(win.window);
            let surface_fn = khr::XlibSurface::new(entry, instance);
            surface_fn.create_xlib_surface(&surface_desc, alloc_clb)
        }
        (RawWindowHandle::Xcb(win), RawDisplayHandle::Xcb(display)) => {
            let surface_desc = vk::XcbSurfaceCreateInfoKHR {
                connection: display.connection.unwrap().as_ptr(),
                window: win.window.into(),
                ..Default::default()
            };
            let surface_fn = khr::XcbSurface::new(entry, instance);
            surface_fn.create_xcb_surface(&surface_desc, alloc_clb)
        }
        (RawWindowHandle::Wayland(win), RawDisplayHandle::Wayland(display)) => {
            let surface_desc = WaylandSurfaceCreateInfoKHR {
                display: display.display.as_ptr(),
                surface: win.surface.as_ptr(),
                ..Default::default()
            };
            let surface_fn = khr::WaylandSurface::new(entry, instance);
            surface_fn.create_wayland_surface(&surface_desc, alloc_clb)
        }
        _ => panic!("Unsupported window/display handle combination for creating surface."),
    }
}

/// Query the required instance extensions for creating a surface from a display handle.
///
/// This [`RawDisplayHandle`] can typically be acquired from a window, but is usually also
/// accessible earlier through an "event loop" concept to allow querying required instance
/// extensions and creation of a compatible Vulkan instance prior to creating a window.
///
/// The returned extensions will include all extension dependencies.
pub fn enumerate_required_extensions(display_handle: DisplayHandle) -> VkResult<Vec<*const i8>> {
    use winit::raw_window_handle::RawDisplayHandle;

    // let extensions: Vec<*const i8> = vec![ash::extensions::ext::DebugUtils::name().as_ptr()];
    let mut extensions: Vec<*const i8> = match display_handle.as_raw() {
        RawDisplayHandle::Windows(_) => {
            let windows_exts: Vec<*const i8> = vec![
                khr::Surface::name().as_ptr(),
                khr::Win32Surface::name().as_ptr(),
            ];
            windows_exts
        }

        RawDisplayHandle::Wayland(_) => {
            let wayland_exts: Vec<*const i8> = vec![
                khr::Surface::name().as_ptr(),
                khr::WaylandSurface::name().as_ptr(),
            ];
            wayland_exts
        }

        RawDisplayHandle::Xlib(_) => {
            let xlib_exts: Vec<*const i8> = vec![
                khr::Surface::name().as_ptr(),
                khr::XlibSurface::name().as_ptr(),
            ];
            xlib_exts
        }

        RawDisplayHandle::Xcb(_) => {
            let xcb_exts: Vec<*const i8> = vec![
                khr::Surface::name().as_ptr(),
                khr::XcbSurface::name().as_ptr(),
            ];
            xcb_exts
        }

        _ => return Err(vk::Result::ERROR_EXTENSION_NOT_PRESENT),
    };

    #[cfg(debug_assertions)]
    extensions.push(ash::extensions::ext::DebugUtils::name().as_ptr());

    Ok(extensions)
}
