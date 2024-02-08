mod raw_handle;

use anyhow::{bail, Result};
use tracing::{event, info, span, trace, Level};
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    raw_window_handle::HasWindowHandle,
    window::{Window as WinitWindow, WindowBuilder},
};

pub struct Window {
    wind: WinitWindow,
    evt_loop: EventLoop<()>,
}

impl Window {
    pub fn new(name: String, width: u16, height: u16) -> Result<Window> {
        let evt_loop = match EventLoop::new() {
            Ok(evt_loop) => evt_loop,
            Err(e) => anyhow::bail!("Could not create event loop: {}", e),
        };

        let Ok(window) = WindowBuilder::new()
            .with_title(name)
            .with_inner_size(winit::dpi::LogicalSize::new(width, height))
            .build(&evt_loop)
        else {
            bail!("Failed to create window!")
        };

        // envent_loop.run(move |event, elwt| {
        //     trace!("{event:?}");

        //     match event {
        //         Event::WindowEvent { window_id, event } if window_id == window.id() => match
        // event {             WindowEvent::CloseRequested => elwt.exit(),
        //             WindowEvent::RedrawRequested => {
        //                 window.pre_present_notify();
        //             }
        //             _ => {}
        //         },
        //         _ => (),
        //     }
        // });

        Ok(Window {
            wind: window,
            evt_loop,
        })
    }

    pub fn logical_loop(self) {
        let window = match Window::new(String::from("youniverse-engine"), 1920, 1080) {
            Ok(window) => window,
            Err(e) => {
                panic!("Could not initialize window!: {}", e);
            }
        };
    }
}
