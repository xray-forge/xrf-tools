use std::collections::HashMap;

use serde::Serialize;
use xrf_app_core::assets::AssetTextureDescriptor;
use xrf_material::{XrayMaterialDescriptor, XraySurfaceDescriptor};
use xrf_vfs::{XrayAsset, XrayRoots};
use xrf_visual::{VisualDependencies, VisualDescription};

use crate::visual_source::VisualSource;

/// What the viewer is showing, paired with where it came from.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectedVisualDescription {
  pub source: VisualSource,
  /// The roots used to resolve this selection's texture files.
  pub roots: XrayRoots,
  pub description: VisualDescription,
  pub dependencies: VisualDependencies,
  /// What each located texture file is, keyed by the logical path that located it.
  pub textures: HashMap<String, AssetTextureDescriptor>,
  /// What the renderer builds for each declared texture, keyed by the reference as the mesh declares it.
  pub materials: HashMap<String, XrayMaterialDescriptor>,
  /// How the renderer draws each submesh, in the order the model declares them.
  pub surfaces: Vec<XraySurfaceDescriptor>,
  /// A `textures.ltx` the searched roots hold, or `None`.
  pub textures_ltx: Option<XrayAsset>,
}
