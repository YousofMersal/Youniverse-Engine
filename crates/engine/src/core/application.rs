use std::time::Instant;

use vulkano::device::{Device, DeviceExtensions};
use winit::{
    event::{
        ElementState, Event::*, KeyboardInput, ModifiersState, VirtualKeyCode::*, WindowEvent::*,
    },
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
        let window = Window::init_window(
            event_loop,
            vk.get_instance().expect("Culd not get isntance"),
        );

        let debug_callback = vk.create_debug_callback();

        vk.set_debug_message(debug_callback);

        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };

        vk.select_physical_device(&window, &device_extensions);

        vk.set_up_device(&device_extensions);

        vk.create_swapchain(&window);

        vk.set_mem_alloc();

        Self {
            window,
            vk,
            start_time: Instant::now(),
        }
    }

    #[allow(unused_variables)]
    fn main_loop(mut self, event_loop: EventLoop<()>) {
        let mut dirty_swap = false;

        let monitor = event_loop
            .available_monitors()
            .next()
            .expect("no monitor found!");

        let mut modifiers = ModifiersState::default();

        event_loop.run(move |event, _, ctr_flow| {
            ctr_flow.set_poll();

            let state = modifiers.shift();

            match event {
                WindowEvent { event, .. } => match event {
                    Resized(size) => {
                        self.window.dims = Some([size.width, size.height]);
                        dirty_swap = true;
                    }
                    CloseRequested => *ctr_flow = ControlFlow::Exit,
                    Focused(_) => {}
                    winit::event::WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(v_code),
                                state: ElementState::Pressed,
                                ..
                            },
                        ..
                    } => match v_code {
                        Escape => {
                            *ctr_flow = ControlFlow::Exit;
                        }
                        F1 => {
                            // *ctr_flow = ControlFlow::Exit;
                            self.vk.toggle_debug_message();
                        }
                        B => {
                            if modifiers.shift() {
                                let fullscreen = Some(winit::window::Fullscreen::Borderless(Some(
                                    monitor.clone(),
                                )));
                                self.window.window.set_fullscreen(fullscreen);
                            } else {
                                self.window.window.set_fullscreen(None);
                            }
                        }
                        _ => (),
                    },
                    ModifiersChanged(mf) => modifiers = mf,
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
