use winit::{event_loop::EventLoop, window::WindowBuilder};

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;

pub struct Window {
    pub dims: Option<[u32; 2]>,
    pub window: winit::window::Window,
}

impl Window {
    pub fn init_window(event_loop: &EventLoop<()>) -> Self {
        let window = WindowBuilder::new()
            .with_title("TempestForge Engine")
            .with_inner_size(winit::dpi::LogicalSize::new(WIDTH, HEIGHT))
            // .with_resizable(false)
            .build(&event_loop)
            .expect("Could not build window");

        let dims = Some([WIDTH, HEIGHT]);
        Self { window, dims }
    }
}
