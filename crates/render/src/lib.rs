// pub fn add(left: usize, right: usize) -> usize {
//     left + right
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }

mod device;
mod instance;
mod swapchain;
mod utils;

use ash::{vk::PhysicalDevice, Entry, Instance};
use device::{create_logical_device, create_physical_devices};

pub struct Vk {
    entry: Entry,
    instance: Instance,
    physical_device: PhysicalDevice,
    device: ash::Device,
}

impl Vk {
    /// Creates a new [`Vk`].
    ///
    /// # Panics
    ///
    /// Panics if .
    pub fn new(ext: &[*const i8]) -> Self {
        let entry = Entry::linked();
        let instance = instance::init(&entry, ext);
        let physical_device = create_physical_devices(&instance);
        let device = create_logical_device(&instance, physical_device);
        // let device = todo!();
        Vk {
            entry,
            instance,
            physical_device,
            device,
        }
    }

    pub fn get_entry(&self) -> &Entry {
        &self.entry
    }

    pub fn get_instance(&self) -> &Instance {
        &self.instance
    }
}
