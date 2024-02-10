use tracing::info;
use winit::{
    event::{
        ElementState, KeyEvent,
        WindowEvent::{self, *},
    },
    event_loop::EventLoopWindowTarget,
    keyboard::{ModifiersState, NamedKey},
};

pub fn windowevents(
    modifiers: &mut ModifiersState,
    event: &WindowEvent,
    elwt: &EventLoopWindowTarget<()>,
) {
    match event {
        // WindowEvent::ActivationTokenDone { serial, token } => todo!(),
        Resized(_) => {}
        // Destroyed => todo!("Destroyed"),
        DroppedFile(_) => todo!(),
        HoveredFile(_) => todo!(),
        HoveredFileCancelled => todo!(),
        // KeyboardInput {
        //     event:
        //         KeyEvent {
        //             logical_key: key,
        //             state: ElementState::Pressed,
        //             repeat,
        //             ..
        //         },
        //     ..
        // } => {
        //     if !repeat {
        //         match key.as_ref() {
        //             winit::keyboard::Key::Named(NamedKey::F1) => elwt.exit(),
        //             Character("t") => {
        //                 println!()
        //             }
        //             _ => {}
        //         }
        //     }
        // }
        ModifiersChanged(new) => {
            *modifiers = new.state();
        }
        CursorMoved {
            device_id: _,
            position: _,
        } => {}
        CursorEntered { device_id: _ } => {}
        CursorLeft { device_id: _ } => {}
        MouseWheel {
            device_id: _,
            delta: _,
            phase: _,
        } => {}
        MouseInput {
            device_id: _,
            state: _,
            button: _,
        } => {}
        ScaleFactorChanged {
            scale_factor: _,
            inner_size_writer: _,
        } => todo!("Resolution"),
        RedrawRequested => {}
        KeyboardInput {
            event:
                KeyEvent {
                    state: ElementState::Pressed,
                    logical_key: winit::keyboard::Key::Named(NamedKey::Escape),
                    ..
                },
            ..
        } => {
            elwt.exit();
            info!("Exiting window");
        }
        CloseRequested => {
            elwt.exit();
            info!("Requested close: Exiting window");
        }
        ActivationTokenDone {
            serial: _,
            token: _,
        } => todo!(),
        Destroyed => todo!(),
        ThemeChanged(_) => todo!(),
        _ => {}
    }
}
