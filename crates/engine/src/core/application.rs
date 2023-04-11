use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

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
    window: Arc<Mutex<Window>>,
    start_time: std::time::Instant,
    vk: Vulkan,
}

impl Application {
    pub fn get_start_time(&self) -> std::time::Instant {
        self.start_time
    }

    fn initialize(event_loop: EventLoop<()>) -> Self {
        let window = Arc::new(Mutex::new(Window::init_window(event_loop)));

        let mut vk = Vulkan::new();
        vk.create_instance(window.lock().unwrap().event_loop.clone().unwrap());

        window.lock().unwrap().create_surface(
            vk.get_vulkan_entry().clone(),
            vk.get_instance().unwrap().clone(),
        );

        vk.create_and_set_debug_callback();

        vk.select_physical_device(window.clone());

        vk.create_logical_device(window.clone());

        vk.make_queues();

        vk.create_swapchain(window.clone());

        vk.create_image_views();

        vk.create_render_pass();

        // vk.set_mem_alloc();

        Self {
            window,
            vk,
            start_time: Instant::now(),
        }
    }

    #[allow(unused_variables)]
    fn main_loop(mut self) {
        let mut dirty_swap = false;

        let monitor = self
            .window
            .lock()
            .unwrap()
            .event_loop
            .clone()
            .unwrap()
            .available_monitors()
            .next()
            .expect("no monitor found!");

        let mut modifiers = ModifiersState::default();

        let wind = Arc::try_unwrap(std::mem::take(&mut self.window));

        self.window
            .lock()
            .unwrap()
            .event_loop
            .take()
            .unwrap()
            .run(move |event, _, ctr_flow| {
                ctr_flow.set_poll();

                let state = modifiers.shift();

                match event {
                    WindowEvent { event, .. } => match event {
                        Resized(size) => {
                            let mut window =
                                self.window.lock().expect("Could not lock mutex on window");
                            window.dims = Some([size.width, size.height]);
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
                                let window =
                                    self.window.lock().expect("Could not lock mutex on window");
                                if modifiers.shift() {
                                    let fullscreen = Some(winit::window::Fullscreen::Borderless(
                                        Some(monitor.clone()),
                                    ));
                                    window.window.set_fullscreen(fullscreen);
                                } else {
                                    window.window.set_fullscreen(None);
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
                        let window = self.window.lock().expect("Could not lock mutex on window");
                        if dirty_swap {
                            let size = window.window.inner_size();
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
        let app = Application::initialize(event_loop);

        app.main_loop();
    }
}
