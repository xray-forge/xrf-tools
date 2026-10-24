use std::collections::HashMap;

use xrf_app_core::assets::AssetTextureDescriptor;
use xrf_material::{XrayMaterialDescriptor, XraySurfaceDescriptor};
use xrf_vfs::{XrayAsset, XrayRoots};
use xrf_visual::{VisualDependencies, VisualMotions, VisualPackage};

use crate::selected_visual_description::SelectedVisualDescription;
use crate::visual_source::VisualSource;

/// One opened model: the visual read, what its references came to, and how the renderer builds and draws it.
pub struct SelectedVisual {
  pub source: VisualSource,
  /// The roots the visual was opened in, kept so a later read searches what the open searched.
  pub roots: XrayRoots,
  pub package: VisualPackage,
  /// What the visual's own references came to, decided at open so a read is a lookup rather than a search.
  pub dependencies: VisualDependencies,
  /// What posing needs from the file, or `None` when the visual carries no bind pose.
  pub motions: Option<VisualMotions>,
  /// What the located texture files are, described at open so a reload reports them without reading anything again.
  pub textures: HashMap<String, AssetTextureDescriptor>,
  /// What the renderer builds for each declared texture, keyed by the reference as the mesh declares it.
  pub materials: HashMap<String, XrayMaterialDescriptor>,
  /// How the renderer draws each submesh, in the order the model declares them.
  pub surfaces: Vec<XraySurfaceDescriptor>,
  /// The `textures.ltx` the roots hold, when they hold one, since its declarations are not read.
  pub textures_ltx: Option<XrayAsset>,
}

impl SelectedVisual {
  pub fn describe(&self) -> SelectedVisualDescription {
    SelectedVisualDescription {
      source: self.source.clone(),
      roots: self.roots.clone(),
      description: self.package.description.clone(),
      dependencies: self.dependencies.clone(),
      textures: self.textures.clone(),
      materials: self.materials.clone(),
      surfaces: self.surfaces.clone(),
      textures_ltx: self.textures_ltx.clone(),
    }
  }
}
