use std::sync::Arc;

use ash::{extensions::khr::Surface, vk::SurfaceKHR, Entry, Instance};
use ash_window::create_surface;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use winit::{event_loop::EventLoop, window::WindowBuilder};

const DEFAULT_WIDTH: u32 = 1920;
const DEFAULT_HEIGHT: u32 = 1080;

pub struct Window {
    pub dims: Option<[u32; 2]>,
    pub window: Arc<winit::window::Window>,
    pub surface: Option<SurfaceInfo>,
    pub event_loop: Arc<EventLoop<()>>,
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

impl Window {
    pub fn init_window(event_loop: Arc<EventLoop<()>>) -> Self {
        let window = Arc::new(
            WindowBuilder::new()
                .with_title("TempestForge Engine")
                .with_inner_size(winit::dpi::LogicalSize::new(DEFAULT_WIDTH, DEFAULT_HEIGHT))
                // .with_resizable(false)
                .build(&event_loop)
                .expect("Could not build window"),
        );

        let dims = Some([DEFAULT_WIDTH, DEFAULT_HEIGHT]);
        Self {
            window,
            dims,
            surface: None,
            event_loop,
        }
    }

    pub fn create_surface(&mut self, entry: Arc<Entry>, instance: Arc<Instance>) {
        let surface = unsafe {
            create_surface(
                &entry,
                &instance,
                self.window.raw_display_handle(),
                self.window.raw_window_handle(),
                None,
            )
            .expect("Could not create surface")
        };

        let surface_loader = Surface::new(&entry, &instance);

        let info = SurfaceInfo::new(surface, surface_loader);

        self.surface = Some(info);
    }
}
