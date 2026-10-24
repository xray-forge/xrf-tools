use std::collections::HashMap;
use std::path::Path;

use serde::Serialize;
use xrf_app_core::assets::AssetTextureDescriptor;
use xrf_app_core::session::{SessionScopes, Sessions};
use xrf_material::{XrayMaterialDescriptor, XraySurfaceDescriptor};
use xrf_vfs::{XrayAsset, XrayRoots};
use xrf_visual::{VisualDependencies, VisualDescription, VisualMotions, VisualPackage};

use crate::posed_visual_motion::PosedVisualMotion;

/// The models open, the motions they are posed by and the roots browsed, each held by the tool that opened it.
pub struct VisualState {
  pub models: Sessions<SelectedVisual>,
  /// The motion each viewer plays its model by, an opening of the viewer's rather than a part of the model.
  pub poses: Sessions<PosedVisualMotion>,
  pub browses: Sessions<XrayRoots>,
}

impl VisualState {
  /// Whatever scope `scopes` releases closes the models, poses and browses it held.
  pub fn new(scopes: &SessionScopes) -> Self {
    Self {
      models: Sessions::new("visual", scopes),
      poses: Sessions::new("visual motion", scopes),
      browses: Sessions::new("visual browse", scopes),
    }
  }
}

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

/// Where a visual is read from.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum VisualSource {
  /// A loose `.ogf` file on disk, named by its filesystem path.
  File { path: String },
  /// An asset of the roots, loose or archived, named by its engine identity.
  Asset { logical_path: String },
}

impl VisualSource {
  pub fn label(&self) -> &str {
    match self {
      Self::File { path } => path,
      Self::Asset { logical_path } => logical_path,
    }
  }

  /// Returns the visual's filesystem path when its source provides one.
  pub fn physical_path(&self) -> Option<&Path> {
    match self {
      Self::File { path } => Some(Path::new(path)),
      Self::Asset { .. } => None,
    }
  }
}

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
