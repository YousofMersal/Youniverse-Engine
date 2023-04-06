use std::sync::Arc;

use vulkano::{instance::Instance, swapchain::Surface};
// use vulkano_win::VkSurfaceBuild;
use vulkano_win::create_surface_from_winit;
use winit::{event_loop::EventLoop, window::WindowBuilder};

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;

pub struct Window {
    pub dims: Option<[u32; 2]>,
    pub window: Arc<winit::window::Window>,
    pub surface: Arc<Surface>,
}

impl Window {
    pub fn init_window(event_loop: &EventLoop<()>, instance: Arc<Instance>) -> Self {
        let window = Arc::new(
            WindowBuilder::new()
                .with_title("TempestForge Engine")
                .with_inner_size(winit::dpi::LogicalSize::new(WIDTH, HEIGHT))
                // .with_resizable(false)
                .build(event_loop)
                // .build_vk_surface(event_loop, instance)
                .expect("Could not build window"),
        );

        let surface =
            create_surface_from_winit(window.clone(), instance).expect("Could not create surface");

        let dims = Some([WIDTH, HEIGHT]);
        Self {
            window,
            dims,
            surface,
        }
    }
}
