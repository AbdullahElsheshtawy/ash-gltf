use anyhow::{anyhow, Result};
use std::ffi::CStr;

use ash::vk;
use winit::{
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
};
mod swapchain;
use swapchain::Swapchain;
mod frame;

pub struct App {
    entry: ash::Entry,
    instance: ash::Instance,
    physical_device: vk::PhysicalDevice,
    device: ash::Device,
    surface: Surface,
    swapchain: Swapchain,
    frames: Vec<frame::Frame>,
    window: winit::window::Window,
}

impl App {
    pub const FRAMES_IN_FLIGHT: u32 = 2;
    pub const REQUIRED_EXTENSIONS: [*const i8; 1] = [ash::khr::swapchain::NAME.as_ptr()];

    pub fn new(window: winit::window::Window) -> Result<Self> {
        let entry = unsafe { ash::Entry::load() }?;
        let app_info = vk::ApplicationInfo::default().api_version(vk::API_VERSION_1_3);

        let extension_names: Vec<&CStr> =
            ash_window::enumerate_required_extensions(window.display_handle()?.as_raw())?
                .iter()
                .map(|ext| unsafe { CStr::from_ptr(*ext) })
                .collect::<Vec<&CStr>>();
        let names: Vec<*const i8> = extension_names.iter().map(|name| name.as_ptr()).collect();

        let instance_info = vk::InstanceCreateInfo {
            p_application_info: &app_info,
            enabled_extension_count: names.len() as u32,
            pp_enabled_extension_names: names.as_ptr(),
            ..Default::default()
        };
        let instance = unsafe { entry.create_instance(&instance_info, None) }?;

        let physical_device = Self::pick_physical_device(&instance)?;

        let surface = Surface {
            instance: ash::khr::surface::Instance::new(&entry, &instance),
            surface: unsafe {
                ash_window::create_surface(
                    &entry,
                    &instance,
                    window.display_handle()?.as_raw(),
                    window.window_handle()?.as_raw(),
                    None,
                )
            }?,
        };

        let features = unsafe { instance.get_physical_device_features(physical_device) };
        let queue_info = [vk::DeviceQueueCreateInfo::default()
            .queue_priorities(&[1.0])
            .queue_family_index(
                Self::find_queue_family_indicies(
                    &instance,
                    physical_device,
                    vk::QueueFlags::GRAPHICS,
                )
                .unwrap(),
            )];
        let device_info = vk::DeviceCreateInfo::default()
            .enabled_extension_names(&Self::REQUIRED_EXTENSIONS)
            .queue_create_infos(&queue_info)
            .enabled_features(&features);
        let device = unsafe { instance.create_device(physical_device, &device_info, None) }?;
        let swapchain = Swapchain::new(
            &instance,
            &device,
            physical_device,
            surface.surface,
            &surface.instance,
        )?;
        let mut frames = Vec::with_capacity(Self::FRAMES_IN_FLIGHT as usize);
        (0..Self::FRAMES_IN_FLIGHT).for_each(|_| {
            frames.push(
                frame::Frame::new(
                    &device,
                    Self::find_queue_family_indicies(
                        &instance,
                        physical_device,
                        vk::QueueFlags::GRAPHICS,
                    )
                    .unwrap(),
                )
                .unwrap(),
            )
        });

        Ok(Self {
            entry,
            instance,
            device,
            window,
            surface,
            swapchain,
            frames,
            physical_device,
        })
    }
    fn pick_physical_device(instance: &ash::Instance) -> Result<vk::PhysicalDevice> {
        let devices = unsafe { instance.enumerate_physical_devices() }?;
        match devices.len() {
            0 => Err(anyhow!("No GPU???")),
            _ => Ok(devices[0]),
        }
    }

    fn find_queue_family_indicies(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        queue_type: vk::QueueFlags,
    ) -> Option<u32> {
        let properties =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

        for (i, family) in properties.iter().enumerate() {
            if family.queue_count > 0 && family.queue_flags.contains(queue_type) {
                return Some(i as u32);
            }
        }

        None
    }

    pub fn run(&mut self, event_loop: EventLoop<()>) {
        event_loop
            .run(move |event, control_flow| match event {
                winit::event::Event::WindowEvent {
                    ref event,
                    window_id,
                } => {
                    if self.window.id() == window_id {
                        match event {
                            WindowEvent::CloseRequested => {
                                control_flow.exit();
                            }

                            WindowEvent::RedrawRequested => self.window.request_redraw(),
                            WindowEvent::KeyboardInput {
                                event:
                                    KeyEvent {
                                        physical_key: PhysicalKey::Code(key),
                                        state: ElementState::Pressed,
                                        ..
                                    },
                                ..
                            } => match key {
                                KeyCode::Escape => control_flow.exit(),
                                KeyCode::F11 => {
                                    if self.window.fullscreen().is_some() {
                                        self.window.set_fullscreen(None);
                                    } else {
                                        self.window.set_fullscreen(Some(
                                            winit::window::Fullscreen::Borderless(None),
                                        ));
                                    }
                                }
                                _ => {}
                            },

                            _ => {}
                        }
                    }
                }
                _ => {}
            })
            .unwrap();
    }
}

struct Surface {
    instance: ash::khr::surface::Instance,
    surface: vk::SurfaceKHR,
}

impl Drop for App {
    fn drop(&mut self) {
        unsafe {
            for view in &self.swapchain.image_views {
                self.device.destroy_image_view(*view, None);
            }

            for frame in &self.frames {
                self.device.destroy_semaphore(frame.swapchain_sem, None);
                self.device.destroy_semaphore(frame.rendering_sem, None);
                self.device.destroy_command_pool(frame.command_pool, None);
            }
            self.swapchain
                .device
                .destroy_swapchain(self.swapchain.swapchain, None);
            self.device.destroy_device(None);
            self.surface
                .instance
                .destroy_surface(self.surface.surface, None);
            self.instance.destroy_instance(None);
        }
    }
}
