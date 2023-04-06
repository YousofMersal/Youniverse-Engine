use std::time::Instant;

use vulkano::device::{physical::PhysicalDeviceType, DeviceExtensions, QueueFlags};
use winit::{
    event::{Event::*, VirtualKeyCode::*, WindowEvent::*},
    event_loop::{ControlFlow, EventLoop},
};

use super::vk::Vulkan;
use super::window::Window;

#[allow(dead_code)]
pub struct Application {
    window: Window,
    start_time: std::time::Instant,
    vk: Vulkan,
}

impl Application {
    pub fn get_start_time(&self) -> std::time::Instant {
        self.start_time
    }

    fn initialize(event_loop: &EventLoop<()>) -> Self {
        let mut vk = Vulkan::new();
        vk.create_instance();
        let window = Window::init_window(&event_loop, vk.get_instance().unwrap());

        let Some(int) = vk.get_instance() else {
            panic!("Could not create Vulkan instance!");
        };

        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };

        int.enumerate_physical_devices()
            .expect("Could not enumerate physical devices")
            .filter(|p| p.supported_extensions().contains(&device_extensions))
            .filter_map(|p| {
                p.queue_family_properties()
                    .iter()
                    .enumerate()
                    .position(|(i, q)| {
                        q.queue_flags.intersects(QueueFlags::GRAPHICS)
                            && p.surface_support(i as u32, &window.surface)
                                .unwrap_or(false)
                    })
                    .map(|q| (p, q as u32))
            })
            .min_by_key(|(p, _)| match p.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                PhysicalDeviceType::Other => 4,
                _ => 5,
            })
            .expect("No device available");

        Self {
            window,
            vk,
            start_time: Instant::now(),
        }
    }

    #[allow(unused_variables)]
    fn main_loop(mut self, event_loop: EventLoop<()>) {
        let mut dirty_swap = false;

        event_loop.run(move |event, _, ctr_flow| {
            *ctr_flow = ControlFlow::Poll;

            match event {
                WindowEvent { event, .. } => match event {
                    Resized(size) => {
                        self.window.dims = Some([size.width, size.height]);
                        dirty_swap = true;
                    }
                    CloseRequested => *ctr_flow = ControlFlow::Exit,
                    Focused(_) => {}
                    KeyboardInput { input, .. } => match input.virtual_keycode {
                        Some(Escape) => {
                            *ctr_flow = ControlFlow::Exit;
                        }
                        _ => (),
                    },
                    _ => {}
                },
                RedrawEventsCleared => {}
                MainEventsCleared => {
                    // swapchain is invalid if window is resized
                    if dirty_swap {
                        let size = self.window.window.inner_size();
                        // let size = window.inner_size();
                        if size.width > 0 && size.height > 0 {
                            // recreate_swapchain(&mut state.swapchain, &window);
                            // state.resize = false;
                        } else {
                            return;
                        }
                    }
                    // self.draw_frame();
                }
                LoopDestroyed => {
                    // self.device
                    //     .device_wait_idle()
                    //     .expect("Failed to wait device idle!")
                }
                _ => {}
            }
        });
    }

    pub fn run() {
        let event_loop = EventLoop::new();
        let app = Application::initialize(&event_loop);

        app.main_loop(event_loop);
    }
}
