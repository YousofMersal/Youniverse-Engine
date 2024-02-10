#!/usr/bin/env rust-script
//!
//!```cargo
//! [dependencies]
//! ash = "0.37.2"
//! ```
use ash::vk;
fn main() {
    dbg!(vk::LayerProperties::default());
}
