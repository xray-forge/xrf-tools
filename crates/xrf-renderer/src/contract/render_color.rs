use serde::{Deserialize, Serialize};

/// An opaque colour as CSS states it, eight bits a channel in sRGB.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct RenderColor {
  pub r: u8,
  pub g: u8,
  pub b: u8,
}

impl RenderColor {
  /// The clear of a non-sRGB target, which stores the channels as given and so shows them as the page does.
  pub fn to_clear(self) -> wgpu::Color {
    wgpu::Color {
      r: self.r as f64 / 255.0,
      g: self.g as f64 / 255.0,
      b: self.b as f64 / 255.0,
      a: 1.0,
    }
  }
}
