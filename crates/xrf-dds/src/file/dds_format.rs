use ddsfile::{D3DFormat, DxgiFormat};

/// Format identity reported by a DDS header.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum DdsFormat {
  D3d(D3DFormat),
  Dxgi(DxgiFormat),
  Unknown,
}
