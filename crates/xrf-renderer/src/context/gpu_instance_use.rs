/// What an instance of a backend is made for: listing its GPUs, drawing on a hardware one ready for DLSS, or drawing on
/// a software one (WARP under D3D12) where no hardware one exists.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GpuInstanceUse {
  Listing,
  Hardware,
  Software,
}

impl GpuInstanceUse {
  /// Whether the instance is ready for DLSS.
  pub fn is_upscaled(self) -> bool {
    self == Self::Hardware
  }

  /// Whether the backend's software adapter is asked for.
  pub fn is_fallback(self) -> bool {
    self == Self::Software
  }
}
