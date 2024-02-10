use std::{thread::sleep, time::Duration};

use youniverse_engine::prelude::*;

fn main() {
    tracing_subscriber::fmt().init();
    let Ok(win) = Window::new(String::from("Tempest Engine: Test"), 1920, 1080) else {
        panic!("Could not create window client")
    };
    win.run();
}
