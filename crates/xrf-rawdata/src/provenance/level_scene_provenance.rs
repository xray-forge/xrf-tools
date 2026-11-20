use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use xrf_error::XrfResult;

use crate::provenance::level_scene_file::LevelSceneFile;
use crate::provenance::level_scene_geometry::LevelSceneGeometry;
use crate::provenance::level_scene_inference::LevelSceneInference;
use crate::provenance::level_scene_refusal::LevelSceneRefusal;
use crate::provenance::level_scene_root_spawn::LevelSceneRootSpawn;
use crate::provenance::level_scene_source::LevelSceneSource;
use crate::provenance::sidecar::Sidecar;

/// The sidecar written beside a decompiled scene, `<name>.level.xrf.json`: the compiled files it came
/// from, what made it and when, every file it wrote and from what, and what in it was chosen rather than read.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelSceneProvenance {
  /// The level's name.
  pub level: String,
  /// The front-end that made the scene, such as `xrf-cli level decompile`, and its version.
  pub tool: String,
  pub tool_version: String,
  /// Seconds since the Unix epoch.
  pub decompiled_at: u64,
  pub sources: Vec<LevelSceneSource>,
  pub files: Vec<LevelSceneFile>,
  pub inferred: Vec<LevelSceneInference>,
  pub refused: Vec<LevelSceneRefusal>,
  /// The root spawn read, where one was given.
  pub root_spawn: Option<LevelSceneRootSpawn>,
  /// The static geometry, where the level's directory held `level.geom`.
  #[serde(default)]
  pub geometry: Option<LevelSceneGeometry>,
}

impl LevelSceneProvenance {
  /// What the sidecar's name adds to its scene's main file's, `<name>.level.xrf.json`.
  pub const SIDECAR_SUFFIX: &'static str = Sidecar::SUFFIX;

  /// The provenance of a scene of `level` decompiled now by `tool`, recording no source, file or finding yet.
  pub fn new(level: impl Into<String>, tool: impl Into<String>, tool_version: impl Into<String>) -> Self {
    Self {
      level: level.into(),
      tool: tool.into(),
      tool_version: tool_version.into(),
      decompiled_at: SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs()),
      sources: Vec::new(),
      files: Vec::new(),
      inferred: Vec::new(),
      refused: Vec::new(),
      root_spawn: None,
      geometry: None,
    }
  }

  /// The sidecar's text.
  ///
  /// # Errors
  ///
  /// Returns an error when the provenance does not serialise, which it always does.
  pub fn to_json(&self) -> XrfResult<String> {
    Sidecar::format_json(self)
  }

  /// Where the sidecar of the scene whose main file is `level` stands: `<level>.xrf.json`, beside it.
  pub fn sidecar_path(level: &Path) -> PathBuf {
    Sidecar::get_path(level)
  }
}
