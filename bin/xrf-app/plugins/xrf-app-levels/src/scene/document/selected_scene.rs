use xrf_editable_level::{EditableLevelGuid, EditableLevelScene};
use xrf_vfs::XrayRoots;

use crate::scene::{
  SceneBuildParamsDescription, SceneCameraDescription, SceneProblem, SceneSource, SceneToolDescription,
  SelectedSceneDescription,
};

/// The Level Editor scene a tool has open as the open read it: what names it, its `.level` and the tools' parts the
/// editor would load, and every part file beside it with what the open found wrong with them.
///
/// What a tool's parts hold, typed, is kept as the session snapshot's parts by the part type that reads it
/// (`SceneObjectsPart`, `SceneBinaryToolsPart`), read the first time anything asks.
pub struct SelectedScene {
  pub source: SceneSource,
  /// The `.level`'s name, which names its part directory and every file of the scene's search.
  pub name: String,
  /// The roots the scene was opened in, kept so a later read searches what the open searched.
  pub roots: XrayRoots,
  /// Where the scene objects' library objects are looked for.
  pub objects_roots: XrayRoots,
  pub guid: EditableLevelGuid,
  /// The `.level`, and each tool's parts that read, carry the scene's GUID and are the tool's to load, in the order
  /// the editor reads them; the parts it would skip are left out.
  pub scene: EditableLevelScene,
  /// Every tool with a part file beside the scene, in the editor's order and those it does not save after them by
  /// name, the files the editor skips among them.
  pub tools: Vec<SceneToolDescription>,
  /// Why each part file left out of [`Self::scene`] was.
  pub problems: Vec<SceneProblem>,
}

impl SelectedScene {
  /// What the page is shown of the open scene itself, which reads nothing: each part it shows beside it has a query
  /// of its own.
  pub fn describe(&self) -> SelectedSceneDescription {
    SelectedSceneDescription {
      source: self.source.clone(),
      name: self.name.clone(),
      guid: Self::format_guid(&self.guid),
      objects_roots: self.objects_roots.clone(),
      build_params: self
        .scene
        .level
        .get_build_params()
        .ok()
        .as_ref()
        .map(SceneBuildParamsDescription::from),
      camera: self
        .scene
        .level
        .get_camera()
        .ok()
        .as_ref()
        .map(SceneCameraDescription::from),
      tools: self.tools.clone(),
      problems: self.problems.clone(),
    }
  }

  /// A scene's GUID as the page is shown it, its two words as the editor's `xrGUID` holds them.
  pub fn format_guid(guid: &EditableLevelGuid) -> String {
    format!("{}:{}", guid.g0, guid.g1)
  }
}
