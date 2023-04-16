use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

use ash::vk::{
    self, ClearColorValue, ClearValue, CommandBufferBeginInfo, CommandBufferResetFlags, DeviceSize,
    Fence, IndexType, Offset2D, PipelineBindPoint, PresentInfoKHR, Rect2D, RenderPassBeginInfo,
    SubmitInfo, SubpassContents, Viewport,
};
use winit::{
    event::{
        ElementState, Event::*, KeyboardInput, ModifiersState, VirtualKeyCode::*, WindowEvent::*,
    },
    event_loop::ControlFlow,
};

use super::{shaders::INDICES, window::Window};
use super::{vk::Vulkan, window::EventLoop};

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

    fn initialize() -> (Self, EventLoop) {
        let (window, event_loop) = Window::init_window();
        let window = Arc::new(Mutex::new(window));
        let event_loop = event_loop;

        let mut vk = Vulkan::new();
        vk.create_instance(&event_loop);

        window
            .lock()
            .unwrap()
            .create_surface(vk.get_vulkan_entry().clone(), vk.get_instance().clone());

        vk.create_and_set_debug_callback();

        vk.select_physical_device(window.clone());

        vk.create_logical_device(window.clone());

        vk.make_queues();

        vk.create_swapchain(window.clone());

        vk.create_image_views();

        vk.create_render_pass();

        vk.create_descriptor_set_layout();

        vk.create_graphics_pipeline();

        vk.create_framebuffers();

        vk.create_command_pool();

        // let texture_img = vk.create_texture_image("textures/texture.jpg");

        vk.create_vertex_buffer();

        vk.create_index_buffer();

        vk.create_uniform_buffers();

        vk.create_descriptor_pool();

        vk.create_descriptor_sets();

        vk.create_command_buffers();

        vk.create_sync_objects();

        (
            Self {
                window,
                vk,
                start_time: Instant::now(),
            },
            event_loop,
        )
    }

    #[allow(unused_variables)]
    fn main_loop(mut self, event_loop: EventLoop) {
        let mut dirty_swap = false;

        let monitor = event_loop
            .available_monitors()
            .next()
            .expect("no monitor found!");

        let mut modifiers = ModifiersState::default();

        event_loop.run(move |event, _, ctr_flow| {
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
                                let fullscreen = Some(winit::window::Fullscreen::Borderless(Some(
                                    monitor.clone(),
                                )));
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
                    if dirty_swap {
                        let size = self.window.clone().lock().unwrap().window.inner_size();
                        if size.width > 0 && size.height > 0 {
                            self.vk.recreate_swapchain(self.window.clone());
                        } else {
                            return;
                        }
                    }
                    self.draw_frame();
                }
                LoopDestroyed => unsafe {
                    self.vk
                        .get_device()
                        .device_wait_idle()
                        .expect("Failed to wait device idle!")
                },
                _ => {}
            }
        });
    }

    pub fn run() {
        let (app, event_loop) = Application::initialize();

        app.main_loop(event_loop);
    }

    fn draw_frame(&mut self) {
        unsafe {
            let sync = self.vk.get_sync().lock().unwrap().next().unwrap();
            let image_available_semaphore = sync.image_available_semaphores;
            let render_finished_semaphore = sync.render_finished_semaphore;
            let in_flight_fence = sync.fence;
            let wait_fences = [in_flight_fence];

            self.vk
                .get_device()
                .wait_for_fences(&wait_fences, true, u64::MAX)
                .expect("Failed to wait for fences!");

            let result = self.vk.get_swapchain().swapchain_loader.acquire_next_image(
                self.vk.get_swapchain().swapchain,
                u64::MAX,
                image_available_semaphore,
                Fence::null(),
            );

            let image_index = match result {
                Ok((idx, is_suboptimal)) => {
                    if !is_suboptimal {
                        idx
                    } else {
                        panic!("Failed to acquire swapchain image!");
                    }
                }
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                    self.vk.recreate_swapchain(self.window.clone());
                    return;
                }
                Err(err) => panic!("Failed to acquire swapchain image: {:?}", err),
            };

            self.vk
                .get_device()
                .reset_fences(&wait_fences)
                .expect("Failed to reset fences!");

            self.vk
                .get_device()
                .reset_command_buffer(
                    self.vk.get_command_buffers()[self.vk.get_current_frame_idx()],
                    CommandBufferResetFlags::empty(),
                )
                .expect("Failed to reset command buffer!");

            self.record_command_buffer(
                &self.vk.get_command_buffers()[self.vk.get_current_frame_idx()],
                image_index as usize,
            );

            let dims = self.window.lock().unwrap().dims.unwrap();

            self.vk.update_uniform_buffer(self.start_time, &dims);

            let wait_semaphores = &[image_available_semaphore];
            let signal_semaphores = &[render_finished_semaphore];
            let wait_stages = &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];

            let submit_info = [*SubmitInfo::builder()
                .wait_semaphores(wait_semaphores)
                .signal_semaphores(signal_semaphores)
                .wait_dst_stage_mask(wait_stages)
                .command_buffers(
                    &[self.vk.get_command_buffers()[self.vk.get_current_frame_idx()]],
                )];

            self.vk
                .get_device()
                .queue_submit(
                    *self.vk.get_queues().graphics_queue,
                    &submit_info,
                    in_flight_fence,
                )
                .expect("Failed to submit draw command buffer!");

            let present_info = *PresentInfoKHR::builder()
                .wait_semaphores(signal_semaphores)
                .image_indices(&[image_index])
                .swapchains(&[self.vk.get_swapchain().swapchain]);

            // let result = self
            //     .vk
            //     .get_swapchain()
            //     .swapchain_loader
            //     .queue_present(*self.vk.get_queues().present_queue, &present_info);

            let res = self
                .vk
                .get_swapchain()
                .swapchain_loader
                .queue_present(*self.vk.get_queues().present_queue, &present_info);

            match res {
                Ok(false) | Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                    self.vk.recreate_swapchain(self.window.clone())
                }
                Ok(true) => {}
                Err(e) => panic!("failed to present queue. Cause: {}", e),
            }
        }
    }

    pub fn record_command_buffer(&self, command_buffer: &vk::CommandBuffer, image_index: usize) {
        let begin_info =
            CommandBufferBeginInfo::builder().flags(vk::CommandBufferUsageFlags::SIMULTANEOUS_USE);

        unsafe {
            self.vk
                .get_device()
                .begin_command_buffer(*command_buffer, &begin_info)
                .expect("Could not make command buffer");
        };

        let clear_value = [ClearValue {
            color: ClearColorValue {
                float32: [0., 0., 0., 1.],
            },
        }];

        let render_pass_info = RenderPassBeginInfo::builder()
            .render_pass(*self.vk.get_render_pass())
            .framebuffer(*self.vk.get_framebuffers()[image_index])
            .render_area(Rect2D {
                offset: Offset2D { x: 0, y: 0 },
                extent: self.vk.get_swapchain().swapchain_extent,
            })
            .clear_values(&clear_value);

        unsafe {
            self.vk.get_device().cmd_begin_render_pass(
                *command_buffer,
                &render_pass_info,
                SubpassContents::INLINE,
            );

            self.vk.get_device().cmd_bind_pipeline(
                *command_buffer,
                PipelineBindPoint::GRAPHICS,
                self.vk.get_t_pipeline().get_pipeline(),
            );

            let viewports = [*Viewport::builder()
                .x(0.)
                .y(0.)
                .width(self.vk.get_swapchain().swapchain_extent.width as f32)
                .height(self.vk.get_swapchain().swapchain_extent.height as f32)
                .min_depth(0.)
                .max_depth(1.)];

            self.vk
                .get_device()
                .cmd_set_viewport(*command_buffer, 0, &viewports);

            let scissors = [*Rect2D::builder()
                .offset(Offset2D { x: 0, y: 0 })
                .extent(self.vk.get_swapchain().swapchain_extent)];
            self.vk
                .get_device()
                .cmd_set_scissor(*command_buffer, 0, &scissors);

            let offset: Vec<DeviceSize> = vec![0];

            self.vk.get_device().cmd_bind_vertex_buffers(
                *command_buffer,
                0,
                &[self.vk.get_vertex_buffer().buffer],
                &offset,
            );

            self.vk.get_device().cmd_bind_index_buffer(
                *command_buffer,
                self.vk.get_index_buffer().buffer,
                0,
                IndexType::UINT32,
            );

            self.vk.get_device().cmd_bind_descriptor_sets(
                *command_buffer,
                PipelineBindPoint::GRAPHICS,
                self.vk.get_t_pipeline().get_layout(),
                0,
                &[self.vk.get_descriptor_set(self.vk.get_current_frame_idx())],
                &[],
            );
            // self.device
            //     .cmd_draw(*command_buffer, VERTS.len() as u32, 1, 0, 0);
            self.vk.get_device().cmd_draw_indexed(
                *command_buffer,
                INDICES.len() as u32,
                1,
                0,
                0,
                0,
            );

            self.vk.get_device().cmd_end_render_pass(*command_buffer);

            self.vk
                .get_device()
                .end_command_buffer(*command_buffer)
                .expect("Could not end command buffer");
        };
    }
}

// impl Drop for Application {
//     fn drop(&mut self) {
//         std::mem::drop(self.window.lock());
//         std::mem::drop(&self.vk);
//     }
// }
