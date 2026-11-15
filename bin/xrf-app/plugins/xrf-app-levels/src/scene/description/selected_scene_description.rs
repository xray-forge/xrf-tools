use serde::Serialize;
use xrf_vfs::XrayRoots;

use crate::scene::{
  SceneBuildParamsDescription, SceneCameraDescription, SceneGameData, SceneProblem, SceneSource, SceneToolDescription,
};

/// What the page is shown of an open scene itself: what names it, what its `.level` holds worth showing, its tools'
/// part files and what is wrong with them. The tools' objects are parts with queries of their own.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectedSceneDescription {
  pub source: SceneSource,
  /// The `.level`'s name, which names its part directory.
  pub name: String,
  /// The scene's GUID, `g0:g1`, which each of its parts carries.
  pub guid: String,
  /// Where its scene objects' library objects are looked for.
  pub objects_roots: XrayRoots,
  /// The game data it is drawn with: the roots its textures and shaders are searched in, none where it has none.
  pub game: SceneGameData,
  /// What its `[build_params]` set, none where they do not read.
  pub build_params: Option<SceneBuildParamsDescription>,
  /// Where the editor's camera stood, none where `[camera]` does not read.
  pub camera: Option<SceneCameraDescription>,
  /// Every tool it holds part files of, in the editor's order, those it does not save after them by name.
  pub tools: Vec<SceneToolDescription>,
  /// What the open found the editor would skip or refuse of its part files.
  pub problems: Vec<SceneProblem>,
}
