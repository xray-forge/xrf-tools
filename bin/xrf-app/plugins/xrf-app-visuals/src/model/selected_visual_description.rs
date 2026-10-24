use std::collections::HashMap;

use serde::Serialize;
use xrf_app_core::assets::{AssetSource, AssetTextureDescriptor};
use xrf_material::{XrayMaterialDescriptor, XraySurfaceDescriptor};
use xrf_vfs::{XrayAsset, XrayRoots};
use xrf_visual::{VisualDependencies, VisualDescription};

/// What the viewer is showing, paired with where it came from: described once at the open and answered as held.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectedVisualDescription {
  pub source: AssetSource,
  /// The roots the visual was opened in, which every later read searches.
  pub roots: XrayRoots,
  pub description: VisualDescription,
  /// What the visual's own references came to, decided at open so a read is a lookup rather than a search.
  pub dependencies: VisualDependencies,
  /// What each located texture file is, keyed by the logical path that located it.
  pub textures: HashMap<String, AssetTextureDescriptor>,
  /// What the renderer builds for each declared texture, keyed by the reference as the mesh declares it.
  pub materials: HashMap<String, XrayMaterialDescriptor>,
  /// How the renderer draws each submesh, in the order the model declares them.
  pub surfaces: Vec<XraySurfaceDescriptor>,
  /// A `textures.ltx` the searched roots hold, or `None`, since its declarations are not read.
  pub textures_ltx: Option<XrayAsset>,
}
