use std::ops::{Deref, DerefMut};

use ash::vk::*;

use super::vk::Vulkan;

pub const MAX_FRAMES_IN_FLIGHT: usize = 2;

#[derive(Clone, Copy)]
pub struct SyncObjects {
    pub image_available_semaphores: Semaphore,
    pub render_finished_semaphore: Semaphore,
    pub fence: Fence,
}

pub struct InFlightFrames {
    pub sync_objects: Vec<SyncObjects>,
    pub current_frame: usize,
}

impl InFlightFrames {
    pub fn new(vk: &Vulkan) -> InFlightFrames {
        let mut sync_objects = Vec::new();

        let semaphore_info = SemaphoreCreateInfo::builder().flags(SemaphoreCreateFlags::empty());
        let fence_info = FenceCreateInfo::builder().flags(FenceCreateFlags::SIGNALED);

        (0..MAX_FRAMES_IN_FLIGHT).for_each(|_| unsafe {
            let semaphore = vk
                .get_device()
                .create_semaphore(&semaphore_info, None)
                .expect("Could not create semaphore");

            let semaphore2 = vk
                .get_device()
                .create_semaphore(&semaphore_info, None)
                .expect("Could not create semaphore");

            let fence = vk
                .get_device()
                .create_fence(&fence_info, None)
                .expect("Could not create fence");

            sync_objects.push(SyncObjects {
                image_available_semaphores: semaphore,
                render_finished_semaphore: semaphore2,
                fence,
            })
        });

        Self {
            sync_objects,
            current_frame: 0,
        }
    }
}

impl Iterator for InFlightFrames {
    type Item = SyncObjects;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.sync_objects[self.current_frame];

        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;

        Some(next)
    }
}

impl Deref for InFlightFrames {
    type Target = Vec<SyncObjects>;

    fn deref(&self) -> &Self::Target {
        &self.sync_objects
    }
}

impl DerefMut for InFlightFrames {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.sync_objects
    }
}

// impl InFlightFrames {
//     pub fn new(vk: &Vulkan) -> Self {

//     }
// }
