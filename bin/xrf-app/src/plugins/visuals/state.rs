use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use xrf_material::XrayMaterialDescriptor;
use xrf_vfs::{XrayAsset, XrayRoots};
use xrf_visual::{VisualDependencies, VisualDescription, VisualMotionPose, VisualPackage};

use crate::core::assets::AssetTextureDescriptor;
use crate::plugins::visuals::skeleton::SelectedSkeleton;

/// What the viewer currently points at: the roots being browsed, and the visual open inside it.
///
/// Both are state for the same reason an open archive is: a reload re-provisions the frontend, and without them the
/// viewer would come back empty while the window still says a model is open. Loading itself is not stateful - every
/// command takes the source it acts on - so this only ever answers what was selected, never gates what can be read.
///
/// The mounted sources are not here. They live in `core/`'s asset roots, shared with every other domain, so opening the
/// same gamedata in two surfaces indexes it once. What is here is the intent: which roots the user chose to browse.
pub struct VisualState {
  pub selected: Mutex<Option<SelectedVisual>>,
  /// The roots being browsed, or `None` when a single visual was opened directly.
  ///
  /// The listing is not kept beside it. It is derived from the roots by the generic asset listing, and the mounts that
  /// listing reads are already cached, so re-deriving it after a reload costs a walk of an index that is in memory.
  pub browsed: Mutex<Option<XrayRoots>>,
}

impl VisualState {
  pub fn new() -> Self {
    Self {
      selected: Mutex::new(None),
      browsed: Mutex::new(None),
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
  pub skeleton: Option<SelectedSkeleton>,
  /// The motion baked by the last `open_motion`, so reading its bytes serves that pose rather than composing again.
  pub posed: Option<VisualMotionPose>,
  /// What the located texture files are, described at open so a reload reports them without reading anything again.
  pub textures: HashMap<String, AssetTextureDescriptor>,
  /// What the renderer builds for each declared texture, keyed by the reference as the mesh declares it.
  pub materials: HashMap<String, XrayMaterialDescriptor>,
  /// The `textures.ltx` the roots hold, when they hold one, since its declarations are not read.
  pub textures_ltx: Option<XrayAsset>,
}

/// Where a visual is read from.
///
/// Both variants are self-describing, and neither is a handle into mount state: an asset is named by its engine
/// identity, which any surface can spell without having opened anything. The roots it is looked for in travels beside
/// the source on every command that takes one, so one call can never mix two roots.
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
  ///
  /// An asset has none to give: it may live inside a volume, and the point of addressing it logically is not having to
  /// care. Its own neighborhood is therefore not searched — the roots it came from already covers it.
  pub fn physical_path(&self) -> Option<&Path> {
    match self {
      Self::File { path } => Some(Path::new(path)),
      Self::Asset { .. } => None,
    }
  }
}

/// What the viewer is showing, paired with where it came from.
///
/// The source travels back so a frontend that reloaded knows what to ask geometry for, without having to remember
/// anything of its own across the reload.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectedVisualDescription {
  pub source: VisualSource,
  /// The roots the selection was opened in, so a reloaded frontend asks for geometry the same way.
  pub roots: XrayRoots,
  pub description: VisualDescription,
  pub dependencies: VisualDependencies,
  /// What each located texture file is, keyed by the logical path that located it.
  pub textures: HashMap<String, AssetTextureDescriptor>,
  /// What the renderer builds for each declared texture, keyed by the reference verbatim as the mesh declares it.
  pub materials: HashMap<String, XrayMaterialDescriptor>,
  /// A `textures.ltx` the searched roots hold, or `None`.
  pub textures_ltx: Option<XrayAsset>,
}
