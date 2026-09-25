use serde::Serialize;
use xrf_material::XraySurfaceDescriptor;
use xrf_visual::DetailsDescription;

use crate::plugins::levels::state::selection::level_texture_reference::LevelTextureReference;

/// A level's grass as packed, and what dresses each of its models.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelDetailsDescription {
  pub details: DetailsDescription,
  /// How the renderer draws each model, in the library's order.
  pub surfaces: Vec<XraySurfaceDescriptor>,
  /// What each texture a model binds resolved to.
  pub textures: Vec<LevelTextureReference>,
}
