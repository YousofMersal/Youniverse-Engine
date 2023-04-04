use std::time::Instant;

use winit::{
    event::{Event::*, VirtualKeyCode::*, WindowEvent::*},
    event_loop::{ControlFlow, EventLoop},
};

use super::window::Window;

pub struct Application {
    window: Window,
    start_time: std::time::Instant,
}

impl Application {
    pub fn get_start_time(&self) -> std::time::Instant {
        self.start_time
    }

    fn initialize(event_loop: &EventLoop<()>) -> Self {
        let window = Window::init_window(&event_loop);

        Self {
            window,
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
                    CloseRequested => {
                        *ctr_flow = ControlFlow::Exit;
                    }
                    Destroyed => todo!(),
                    DroppedFile(_) => todo!(),
                    HoveredFile(_) => todo!(),
                    HoveredFileCancelled => todo!(),
                    Focused(_) => {}
                    KeyboardInput { input, .. } => match input.virtual_keycode {
                        Some(Escape) => {
                            *ctr_flow = ControlFlow::Exit;
                        }
                        _ => (),
                    },
                    CursorMoved {
                        device_id,
                        position,
                        ..
                    } => {}
                    CursorEntered { device_id } => {}
                    CursorLeft { device_id } => {}
                    MouseWheel {
                        device_id,
                        delta,
                        phase,
                        ..
                    } => {}
                    MouseInput {
                        device_id,
                        state,
                        button,
                        ..
                    } => {}
                    _ => {}
                },
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
                // NewEvents(_) => {}
                // DeviceEvent { device_id, event } => {}
                // UserEvent(_) => {}
                // Suspended => {}
                // Resumed => {}
                // RedrawRequested(_) => {}
                // RedrawEventsCleared => {}
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
