use anyhow::Result;
use ash::vk;

#[derive(Default, Debug)]
pub struct Frame {
    pub swapchain_sem: vk::Semaphore,
    pub rendering_sem: vk::Semaphore,
    pub command_pool: vk::CommandPool,
    pub main_command_buffer: vk::CommandBuffer,
}

impl Frame {
    pub fn new(device: &ash::Device, index: u32) -> Result<Self> {
        let command_pool_info = vk::CommandPoolCreateInfo {
            flags: vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
            queue_family_index: index,
            ..Default::default()
        };
        let command_pool = unsafe { device.create_command_pool(&command_pool_info, None) }.unwrap();
        let allocate_info = vk::CommandBufferAllocateInfo {
            command_pool,
            level: vk::CommandBufferLevel::PRIMARY,
            command_buffer_count: 1,
            ..Default::default()
        };
        let command_buffer = unsafe { device.allocate_command_buffers(&allocate_info).unwrap() };
        Ok(Frame {
            swapchain_sem: Self::create_sync_objects(device)?,
            rendering_sem: Self::create_sync_objects(device)?,
            command_pool,
            main_command_buffer: command_buffer[0],
        })
    }

    pub fn create_sync_objects(device: &ash::Device) -> Result<vk::Semaphore> {
        let info = vk::SemaphoreCreateInfo::default();
        Ok(unsafe { device.create_semaphore(&info, None)? })
    }
}
