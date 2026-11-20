use xrf_editable_level::{EditableLevelGuid, EditableLevelScene};
use xrf_vfs::XrayRoots;

use crate::scene::{
  SceneBuildParamsDescription, SceneCameraDescription, SceneGameData, SceneProblem, SceneSource, SceneToolDescription,
  SelectedSceneDescription,
};

/// The Level Editor scene a tool has open: its names, its `.level` and the parts the editor would load, and every part
/// file beside it with the problems the open found.
///
/// What the parts hold, typed, is read on first use into session snapshot parts (`SceneObjectsPart`,
/// `SceneBinaryToolsPart`).
pub struct SelectedScene {
  pub source: SceneSource,
  /// The `.level`'s name, which names its part directory and every file of the scene's search.
  pub name: String,
  /// The roots the scene was opened in, kept so a later read searches what the open searched.
  pub roots: XrayRoots,
  /// Where the scene objects' library objects are looked for.
  pub objects_roots: XrayRoots,
  /// The game data the scene is drawn with, kept so every draw searches what the open was given.
  pub game: SceneGameData,
  pub guid: EditableLevelGuid,
  /// The `.level` and, in the editor's read order, each tool's parts the editor would load; those it would skip are
  /// left out.
  pub scene: EditableLevelScene,
  /// Every tool with a part file beside the scene, in the editor's order, then the tools it does not save, by name.
  pub tools: Vec<SceneToolDescription>,
  /// Why each part file left out of [`Self::scene`] was left out.
  pub problems: Vec<SceneProblem>,
}

impl SelectedScene {
  /// What the page shows of the open scene itself, reading nothing; its parts have queries of their own.
  pub fn describe(&self) -> SelectedSceneDescription {
    SelectedSceneDescription {
      source: self.source.clone(),
      name: self.name.clone(),
      guid: Self::format_guid(&self.guid),
      objects_roots: self.objects_roots.clone(),
      game: self.game.clone(),
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

  /// A scene's GUID as the page shows it, `g0:g1`, the two words of the editor's `xrGUID`.
  pub fn format_guid(guid: &EditableLevelGuid) -> String {
    format!("{}:{}", guid.g0, guid.g1)
  }
}
