use serde::{Deserialize, Serialize};

/// An opaque colour as CSS states it, eight bits a channel in sRGB.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct RenderColor {
  pub r: u8,
  pub g: u8,
  pub b: u8,
}
