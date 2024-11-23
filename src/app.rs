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
    debug_instance: ash::ext::debug_utils::Instance,
    debug_messenger: vk::DebugUtilsMessengerEXT,
    physical_device: vk::PhysicalDevice,
    device: ash::Device,
    surface: Surface,
    event_loop: EventLoop<()>,
    swapchain: Swapchain,
    frames: Vec<frame::Frame>,
    window: winit::window::Window,
}

impl App {
    pub const FRAMES_IN_FLIGHT: u32 = 2;
    pub const LAYERS: [&CStr; 1] =
        [unsafe { CStr::from_bytes_with_nul_unchecked(b"VK_LAYER_KHRONOS_validation\0") }];

    pub const REQUIRED_EXTENSIONS: [*const i8; 1] = [ash::khr::swapchain::NAME.as_ptr()];

    pub fn new(window: winit::window::Window, event_loop: EventLoop<()>) -> Result<Self> {
        let entry = unsafe { ash::Entry::load() }?;
        let app_info = vk::ApplicationInfo::default().api_version(vk::API_VERSION_1_3);
        let layer_names: Vec<_> = Self::LAYERS.iter().map(|name| name.as_ptr()).collect();
        let debug_extensions = ash::ext::debug_utils::NAME;
        let mut extension_names: Vec<&CStr> =
            ash_window::enumerate_required_extensions(window.display_handle()?.as_raw())?
                .iter()
                .map(|ext| unsafe { CStr::from_ptr(ext.clone()) })
                .collect::<Vec<&CStr>>();
        extension_names.push(debug_extensions);
        let mut names: Vec<*const i8> = Vec::with_capacity(extension_names.len());
        {
            for name in extension_names {
                names.push(name.as_ptr());
            }
        }
        let instance_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_layer_names(&layer_names)
            .enabled_extension_names(&names);
        let instance = unsafe { entry.create_instance(&instance_info, None) }?;

        let debug_instance = ash::ext::debug_utils::Instance::new(&entry, &instance);
        let debug_info = vk::DebugUtilsMessengerCreateInfoEXT {
            message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
            message_type: vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                | vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                | vk::DebugUtilsMessageTypeFlagsEXT::DEVICE_ADDRESS_BINDING,
            pfn_user_callback: Some(Self::debug_callback),

            ..Default::default()
        };
        let debug_messenger =
            unsafe { debug_instance.create_debug_utils_messenger(&debug_info, None) }?;

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
            (0..Self::FRAMES_IN_FLIGHT).into_iter().for_each(|_| {
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
        };

        Ok(Self {
            entry,
            instance,
            debug_messenger,
            physical_device,
            device,
            window,
            debug_instance,
            event_loop,
            surface,
            swapchain,
            frames,
        })
    }
    fn pick_physical_device(instance: &ash::Instance) -> Result<vk::PhysicalDevice> {
        let devices = unsafe { instance.enumerate_physical_devices() }?;
        match devices.len() {
            0 => Err(anyhow!("No GPU???")),
            _ => Ok(devices[0]),
        }
    }
    unsafe extern "system" fn debug_callback(
        message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
        message_type: vk::DebugUtilsMessageTypeFlagsEXT,
        callback_data_ptr: *const vk::DebugUtilsMessengerCallbackDataEXT,
        _user_data: *mut std::ffi::c_void,
    ) -> vk::Bool32 {
        let level = match message_severity {
            vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => log::Level::Debug,
            vk::DebugUtilsMessageSeverityFlagsEXT::INFO => log::Level::Info,
            vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => log::Level::Warn,
            vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => log::Level::Error,
            _ => log::Level::Warn,
        };
        let message = std::ffi::CStr::from_ptr((*callback_data_ptr).p_message);
        log::log!(level, "[{:?}] {:?}", message_type, message);

        vk::FALSE
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

    pub fn run(self) -> Result<()> {
        Ok(self
            .event_loop
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
            })?)
    }
}

struct Surface {
    instance: ash::khr::surface::Instance,
    surface: vk::SurfaceKHR,
}
