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
mod utils;

use ash::{
    vk::{self, PhysicalDevice, PhysicalDeviceFeatures2},
    Entry, Instance,
};

pub struct Vk {
    entry: Entry,
    instance: Instance,
    physical_device: PhysicalDevice,
    // device: Device,
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
        let physical_device = Self::create_physical_device(&instance);
        // let device = todo!();
        Vk {
            entry,
            instance,
            physical_device,
            // device,
        }
    }

    pub fn get_entry(&self) -> &Entry {
        &self.entry
    }

    pub fn get_instance(&self) -> &Instance {
        &self.instance
    }

    pub fn create_physical_device(instance: &Instance) -> vk::PhysicalDevice {
        let mut features13 = vk::PhysicalDeviceVulkan13Features::builder().build();

        let mut features12 = vk::PhysicalDeviceVulkan12Features::builder().build();
        features12.p_next =
            &mut features13 as *mut vk::PhysicalDeviceVulkan13Features as *mut std::ffi::c_void;

        let mut features = vk::PhysicalDeviceFeatures2::builder()
            .push_next(&mut features12)
            .build();

        let p_device: PhysicalDevice = unsafe {
            instance
                .enumerate_physical_devices()
                .expect("Physical device error!")
        }
        .iter()
        .find_map(|p_dev| unsafe {
            let p_props = instance.get_physical_device_properties(*p_dev);
            // let p_features = instance.get_physical_device_features(*p_dev);
            instance.get_physical_device_features2(*p_dev, &mut features);

            dbg!("Device Name: {}", features.p_next);

            // find a discrete gpu
            match p_props.device_type {
                vk::PhysicalDeviceType::DISCRETE_GPU | vk::PhysicalDeviceType::INTEGRATED_GPU => {
                    Some(*p_dev)
                }
                _ => None,
            }
        })
        .expect("No suitable device found!");

        p_device
    }
}
