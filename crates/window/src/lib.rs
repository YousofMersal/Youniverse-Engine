mod events;
mod raw_handle;
use ash::vk;
use render::Vk;

use anyhow::{bail, Result};
use events::windowevents;
use tracing::error;
use winit::{
    event::Event,
    event_loop::EventLoop,
    keyboard::ModifiersState,
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::{Window as WinitWindow, WindowBuilder},
};

pub struct Window {
    window: WinitWindow,
    evt_loop: EventLoop<()>,
    ctx: WindowContext,
}

pub struct WindowContext {
    vk: Vk,
    surface: vk::SurfaceKHR,
}

impl WindowContext {
    pub fn new(ext: Vec<*const i8>) -> Self {
        let vk = Vk::new(&ext);

        Self {
            vk,
            surface: vk::SurfaceKHR::null(),
        }
    }

    pub fn set_surface(&mut self, surface: vk::SurfaceKHR) {
        self.surface = surface;
    }
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

        let ext =
            raw_handle::enumerate_required_extensions(window.display_handle().unwrap()).unwrap();

        let ctx = WindowContext::new(ext);

        let mut wind = Window {
            window,
            evt_loop,
            ctx,
        };

        let surface = wind.create_surface_khr();
        wind.ctx.set_surface(surface);

        Ok(wind)
    }

    pub fn run(self) {
        let mut modifiers = ModifiersState::default();

        let evt_res = self.evt_loop.run(move |event, elwt| match event {
            // Event::NewEvents(_) => todo!(),
            Event::WindowEvent { window_id, event } if window_id == self.window.id() => {
                windowevents(&mut modifiers, &event, &elwt);
            }
            Event::DeviceEvent { event, .. } => match event {
                // winit::event::DeviceEvent::MouseMotion { delta } => todo!("Mouse mothion"),
                // winit::event::DeviceEvent::MouseWheel { delta } => todo!("Mouse wheel"),
                // winit::event::DeviceEvent::Motion { axis, value } => todo!("Mouse motion"),
                // winit::event::DeviceEvent::Button { button, state } => todo!(),
                _ => {}
            },
            _ => {}
        });

        match evt_res {
            Ok(_) => {}
            Err(e) => panic!("Event loop error: {}", e),
        }
    }

    pub fn create_surface_khr(&self) -> vk::SurfaceKHR {
        let rdh = self
            .window
            .display_handle()
            .expect("Could not get display handle");

        let rwh = self
            .window
            .window_handle()
            .expect("Could not get window handle");

        unsafe {
            if let Ok(surface) = crate::raw_handle::create_surface(
                self.ctx.vk.get_entry(),
                self.ctx.vk.get_instance(),
                rwh,
                rdh,
                None,
            ) {
                surface
            } else {
                panic!("Could not create surface")
            }
        }
    }
}
