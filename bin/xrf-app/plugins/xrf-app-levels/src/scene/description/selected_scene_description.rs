use serde::Serialize;
use xrf_vfs::XrayRoots;

use crate::scene::{
  SceneBuildParamsDescription, SceneCameraDescription, SceneGameData, SceneProblem, SceneSource, SceneToolDescription,
};

/// What the page shows of an open scene itself: its names, its `.level` settings, its tools' part files and their
/// problems. The tools' objects are parts with queries of their own.
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
  /// The game data roots its textures and shaders are searched in, if it has any.
  pub game: SceneGameData,
  /// What its `[build_params]` set, none where they do not read.
  pub build_params: Option<SceneBuildParamsDescription>,
  /// Where the editor's camera stood, none where `[camera]` does not read.
  pub camera: Option<SceneCameraDescription>,
  /// Every tool it holds part files of, in the editor's order, then the tools the editor does not save, by name.
  pub tools: Vec<SceneToolDescription>,
  /// What the open found the editor would skip or refuse of its part files.
  pub problems: Vec<SceneProblem>,
}
