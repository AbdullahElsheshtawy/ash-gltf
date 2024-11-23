use anyhow::Result;
use ash::vk;

pub struct SwapchainSupportDetails {
    caps: vk::SurfaceCapabilitiesKHR,
    formats: Vec<vk::SurfaceFormatKHR>,
    present_modes: Vec<vk::PresentModeKHR>,
}

impl SwapchainSupportDetails {
    pub fn new(
        physical_device: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
        surface_instance: &ash::khr::surface::Instance,
    ) -> Result<SwapchainSupportDetails> {
        Ok(SwapchainSupportDetails {
            caps: unsafe {
                surface_instance.get_physical_device_surface_capabilities(physical_device, surface)
            }?,
            formats: unsafe {
                surface_instance.get_physical_device_surface_formats(physical_device, surface)
            }?,
            present_modes: unsafe {
                surface_instance.get_physical_device_surface_present_modes(physical_device, surface)
            }?,
        })
    }

    pub fn choose_surface_format(
        &self,
        preferred_format: vk::SurfaceFormatKHR,
    ) -> vk::SurfaceFormatKHR {
        for format in &self.formats {
            if *format == preferred_format {
                return *format;
            }
        }
        self.formats[0]
    }

    pub fn choose_present_mode(&self, preferred_mode: vk::PresentModeKHR) -> vk::PresentModeKHR {
        for mode in &self.present_modes {
            if *mode == preferred_mode {
                return *mode;
            }
        }

        vk::PresentModeKHR::FIFO
    }

    pub fn choose_extent(&self, width: u32, height: u32) -> vk::Extent2D {
        if self.caps.current_extent.width != u32::MAX {
            return self.caps.current_extent;
        }

        vk::Extent2D {
            width: u32::clamp(
                width,
                self.caps.min_image_extent.width,
                self.caps.max_image_extent.width,
            ),
            height: u32::clamp(
                height,
                self.caps.min_image_extent.height,
                self.caps.max_image_extent.height,
            ),
        }
    }

    fn optimal_min_image_count(&self) -> u32 {
        self.caps.min_image_count + 1
    }
}
pub struct Swapchain {
    pub device: ash::khr::swapchain::Device,
    pub swapchain: vk::SwapchainKHR,
    pub extent: vk::Extent2D,
    pub images: Vec<vk::Image>,
    pub image_views: Vec<vk::ImageView>,
    pub details: SwapchainSupportDetails,
}

impl Swapchain {
    pub fn new(
        instance: &ash::Instance,
        device: &ash::Device,
        physical_device: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
        surface_instance: &ash::khr::surface::Instance,
    ) -> Result<Self> {
        let swapchain_device = ash::khr::swapchain::Device::new(instance, device);
        let details = SwapchainSupportDetails::new(physical_device, surface, surface_instance)?;
        let surface_format = details.choose_surface_format(vk::SurfaceFormatKHR {
            format: vk::Format::B8G8R8_UNORM,
            color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR,
        });

        let present_mode = details.choose_present_mode(vk::PresentModeKHR::MAILBOX);
        let extent = details.choose_extent(crate::WINDOW_WIDTH, crate::WINDOW_HEIGHT);
        let min_image_count = details.optimal_min_image_count();
        let swapchain_info = vk::SwapchainCreateInfoKHR {
            surface,
            min_image_count,
            image_format: surface_format.format,
            image_color_space: surface_format.color_space,
            image_extent: extent,
            image_array_layers: 1,
            image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
            image_sharing_mode: vk::SharingMode::EXCLUSIVE,
            pre_transform: details.caps.current_transform,
            composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
            present_mode,
            clipped: vk::TRUE,
            ..Default::default()
        };
        let swapchain = unsafe { swapchain_device.create_swapchain(&swapchain_info, None) }?;
        let images = unsafe { swapchain_device.get_swapchain_images(swapchain) }?;
        let image_views = Self::create_image_views(device, &images, surface_format.format);
        Ok(Self {
            device: swapchain_device,
            swapchain,
            extent,
            images,
            image_views,
            details,
        })
    }

    fn create_image_views(
        device: &ash::Device,
        images: &[vk::Image],
        format: vk::Format,
    ) -> Vec<vk::ImageView> {
        images
            .iter()
            .map(|image| {
                let create_info = vk::ImageViewCreateInfo {
                    image: *image,
                    view_type: vk::ImageViewType::TYPE_2D,
                    format,
                    components: vk::ComponentMapping {
                        r: vk::ComponentSwizzle::IDENTITY,
                        g: vk::ComponentSwizzle::IDENTITY,
                        b: vk::ComponentSwizzle::IDENTITY,
                        a: vk::ComponentSwizzle::IDENTITY,
                    },
                    subresource_range: vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    },
                    ..Default::default()
                };

                unsafe { device.create_image_view(&create_info, None).unwrap() }
            })
            .collect()
    }
}
