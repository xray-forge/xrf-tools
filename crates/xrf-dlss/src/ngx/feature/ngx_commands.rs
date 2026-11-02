#[cfg(windows)]
use crate::ngx::bindings::ID3D12GraphicsCommandList;

/// The command list NGX records into, of the device's API.
#[derive(Clone, Copy)]
pub(crate) enum NgxCommands {
  Vulkan(ash::vk::CommandBuffer),
  #[cfg(windows)]
  D3d12(*mut ID3D12GraphicsCommandList),
}
