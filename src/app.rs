use anyhow::{anyhow, Result};
use std::ffi::CStr;

use ash::vk;
use winit::{
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
};

pub struct App {
    entry: ash::Entry,
    instance: ash::Instance,
    debug_instance: ash::ext::debug_utils::Instance,
    debug_messenger: vk::DebugUtilsMessengerEXT,
    physical_device: vk::PhysicalDevice,
    device: ash::Device,
    event_loop: EventLoop<()>,
    window: winit::window::Window,
}

impl App {
    pub const LAYERS: [&CStr; 1] =
        [unsafe { CStr::from_bytes_with_nul_unchecked(b"VK_LAYER_KHRONOS_validation\0") }];

    pub const REQUIRED_EXTENSIONS: [*const i8; 2] = [
        ash::khr::swapchain::NAME.as_ptr(),
        ash::khr::dynamic_rendering::NAME.as_ptr(),
    ];

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
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
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
                Self::find_queue_family_indicies(&instance, physical_device, &surface)?
                    .graphics
                    .unwrap(),
            )];
        let device_info = vk::DeviceCreateInfo::default()
            .enabled_extension_names(&Self::REQUIRED_EXTENSIONS)
            .queue_create_infos(&queue_info)
            .enabled_features(&features);
        let device = unsafe { instance.create_device(physical_device, &device_info, None) }?;
        Ok(Self {
            entry,
            instance,
            debug_messenger,
            physical_device,
            device,
            window,
            debug_instance,
            event_loop,
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
        surface: &Surface,
    ) -> Result<QueueFamilyIndices> {
        let properties =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

        let mut indices = QueueFamilyIndices::default();

        for (i, family) in properties.iter().enumerate() {
            if family.queue_count > 0 && family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                indices.graphics = Some(i as u32);
            }
            if unsafe {
                surface.instance.get_physical_device_surface_support(
                    physical_device,
                    i as u32,
                    surface.surface,
                )
            }? {
                indices.present = Some(i as u32);
            }

            if indices.is_completed() {
                break;
            }
        }

        Ok(indices)
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

#[derive(Debug, Default)]
struct QueueFamilyIndices {
    graphics: Option<u32>,
    present: Option<u32>,
}
impl QueueFamilyIndices {
    fn is_completed(&self) -> bool {
        self.graphics.is_some() && self.present.is_some()
    }
}
