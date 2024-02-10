use std::sync::Arc;

use winit::{
    raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle},
    window::WindowBuilder,
};

const DEFAULT_WIDTH: u32 = 1920;
const DEFAULT_HEIGHT: u32 = 1080;
pub type EventLoop = winit::event_loop::EventLoop<()>;

pub struct Window {
    pub dims: Option<[u32; 2]>,
    pub window: Arc<winit::window::Window>,
}

impl Window {
    pub fn init_window() -> (Self, EventLoop) {
        let event_loop = EventLoop::new();

        let window = Arc::new(
            WindowBuilder::new()
                .with_title("TempestForge Engine")
                .with_inner_size(winit::dpi::LogicalSize::new(DEFAULT_WIDTH, DEFAULT_HEIGHT))
                // .with_resizable(false)
                .build(&event_loop)
                .expect("Could not build window"),
        );

        let dims = Some([DEFAULT_WIDTH, DEFAULT_HEIGHT]);
        (Self { window, dims }, event_loop)
    }

    pub fn get_raw_window_handle(&self) -> RawWindowHandle {
        self.window
            .window_handle()
            .expect("Could not retrieve window handle")
            .as_raw()
    }

    pub fn get_raw_display_handle(&self) -> RawDisplayHandle {
        self.window
            .display_handle()
            .expect("Could not retrieve window handle")
            .as_raw()
    }
}
