use serde::Deserialize;
use xrf_vfs::XrayRoots;

use crate::plugins::textures::source::TextureSource;
use crate::plugins::textures::surface::texture_surface_alpha::TextureSurfaceAlpha;
use crate::plugins::textures::surface::texture_surface_shape::TextureSurfaceShape;

/// A texture laid on a body, and how it is laid there.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureSurfaceRequest {
  pub source: TextureSource,
  /// The roots the texture is resolved in, as its description was.
  pub roots: XrayRoots,
  pub shape: TextureSurfaceShape,
  /// How many times the texture repeats across the body, which is how a tiling seam becomes visible.
  pub tiling: f32,
  pub alpha: TextureSurfaceAlpha,
  /// Width over height of the texture, which the plane is stretched to.
  pub aspect: f32,
}
