#[cfg(windows)]
use wgpu::hal::api::Dx12;
use wgpu::hal::api::Vulkan;

/// The graphics API NGX runs on, the one a device was started with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NgxApi {
  Vulkan,
  #[cfg(windows)]
  D3d12,
}

impl NgxApi {
  /// The API `device` draws with, none where NGX has none of it.
  pub(crate) fn of(device: &wgpu::Device) -> Option<Self> {
    if unsafe { device.as_hal::<Vulkan>() }.is_some() {
      return Some(Self::Vulkan);
    }

    #[cfg(windows)]
    if unsafe { device.as_hal::<Dx12>() }.is_some() {
      return Some(Self::D3d12);
    }

    None
  }
}
