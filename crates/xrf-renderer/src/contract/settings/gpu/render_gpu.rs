use serde::{Deserialize, Serialize};

/// A GPU identified by its PCI vendor and device IDs. D3D12 and Vulkan report the same IDs; two identical cards share
/// one identity.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderGpu {
  pub vendor: u32,
  pub device: u32,
}

impl RenderGpu {
  /// The identity of the GPU an adapter runs on.
  pub fn of(info: &wgpu::AdapterInfo) -> Self {
    Self {
      vendor: info.vendor,
      device: info.device,
    }
  }
}
