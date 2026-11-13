use serde::{Deserialize, Serialize};

/// The GPU the GPU backend traces on, by its PCI vendor and device ids: the renderer's `RenderGpu`,
/// spelled again here so a request names one in a build without the `gpu` feature, which links no graphics code.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RayGpu {
  pub vendor: u32,
  pub device: u32,
}

#[cfg(feature = "gpu")]
impl From<RayGpu> for xrf_renderer_core::RenderGpu {
  fn from(RayGpu { vendor, device }: RayGpu) -> Self {
    Self { vendor, device }
  }
}
