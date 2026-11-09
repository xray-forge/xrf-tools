use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use xrf_error::{XrfError, XrfResult};

/// One compiled file a scene was decompiled from.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelSceneSource {
  /// Its name in the level's directory, such as `level.ps_static`.
  pub file: String,
  /// Its SHA-256, lowercase hex.
  pub sha256: String,
}

/// One file the decompile wrote, and the compiled files it came from.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelSceneFile {
  /// Its path relative to the scene's main file's directory, such as `labx8/ps.part`.
  pub written: String,
  pub sources: Vec<String>,
}

/// What a decompiled scene holds that its sources do not say, which the decompile chose.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LevelSceneInference {
  /// Objects are named as the editor names new ones, `<prefix>_NNNN`, the prefix their file's name or their tool's.
  ObjectNames,
  /// The level options are a new level's, built into and prefixed by the level's name.
  LevelOptions,
  /// The level serves the game modes its respawn points answer to, or single player where it has none.
  GameTypes,
  /// The build parameters are the editor's defaults but for a weld distance of 0, for geometry already welded.
  BuildParameters,
  /// The camera stands at the origin, unturned.
  Camera,
  /// Sound sources take a new source's distances, which the editor reads again from each sound file.
  SoundSourceDistances,
  /// The wall mark tool takes a new tool's brush, and each decal the brush's size and turn.
  WallmarkBrush,
  /// The spawn part holds no entity: `level.spawn` is not decompiled yet.
  EntitiesNotDecompiled,
}

/// The sidecar written beside a decompiled scene, `<name>.level.xrf.json` (decision 22): the compiled files it came
/// from, what made it and when, every file it wrote and from what, and what in it was chosen rather than read.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
}

impl LevelSceneProvenance {
  /// What the sidecar's name adds to its scene's main file's, `<name>.level.xrf.json`.
  pub const SIDECAR_SUFFIX: &'static str = ".xrf.json";

  /// The provenance of a scene of `level` decompiled now by `tool`.
  pub fn new(
    level: impl Into<String>,
    tool: impl Into<String>,
    tool_version: impl Into<String>,
    sources: Vec<LevelSceneSource>,
    files: Vec<LevelSceneFile>,
    inferred: Vec<LevelSceneInference>,
  ) -> Self {
    Self {
      level: level.into(),
      tool: tool.into(),
      tool_version: tool_version.into(),
      decompiled_at: SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs()),
      sources,
      files,
      inferred,
    }
  }

  /// The sidecar's text.
  ///
  /// # Errors
  ///
  /// Returns an error when the provenance does not serialise, which it always does.
  pub fn to_json(&self) -> XrfResult<String> {
    serde_json::to_string_pretty(self)
      .map_err(|error| XrfError::new_serialization_error(format!("Provenance did not serialise: {error}")))
  }

  /// Where the sidecar of the scene whose main file is `level` stands: `<level>.xrf.json`, beside it.
  pub fn sidecar_path(level: &Path) -> PathBuf {
    let mut name = level.as_os_str().to_owned();

    name.push(Self::SIDECAR_SUFFIX);

    PathBuf::from(name)
  }
}
