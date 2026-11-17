use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::level::build::level_build_impostor_limit::LevelBuildImpostorLimit;

/// A scene's build into a level directory, as a front-end asks for it: the game files always, and the level compiler's
/// files (`build.prj`, the occluders, `level.ltx`, details, AI map, impostors, breakables) where the game material
/// library is named, the objects read from the objects roots ([`Self::get_objects_roots`]).
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelBuildRequest {
  /// The scene's main file, `<name>.level`, its parts in the directory beside it named after it.
  pub source: PathBuf,
  /// The level directory the files are written into.
  pub destination: PathBuf,
  /// The objects roots (`$objects$`) the scene's objects are read from, each from the first holding it; none for
  /// `<rawdata>/objects` beside the scene, where a decompile writes them.
  pub objects: Vec<PathBuf>,
  /// The game material library, `gamemtl.xr`; the level compiler's files are written only where it is named.
  pub game_materials: Option<PathBuf>,
  /// The textures root (`$game_textures$`) the details' textures and the impostors' strips are read from.
  pub textures: Option<PathBuf>,
  /// How large the impostor atlas may grow.
  pub impostor_limit: LevelBuildImpostorLimit,
  /// Whether files the build writes are overwritten where the directory holds them rather than refused.
  pub is_forced: bool,
}

impl LevelBuildRequest {
  /// The directory a rawdata tree keeps objects in, beside the `levels` directory holding its scenes.
  pub const OBJECTS_DIRECTORY: &'static str = "objects";

  /// The objects roots the scene's objects are read from: the ones named, else `<rawdata>/objects` beside the scene's
  /// `levels` directory where it exists, as the SDK's `fs.ltx` puts `$maps$` and `$objects$` side by side and a
  /// decompile writes the scene's objects and the library objects it places there; none otherwise.
  pub fn get_objects_roots(&self) -> Vec<PathBuf> {
    if !self.objects.is_empty() {
      return self.objects.clone();
    }

    self
      .source
      .parent()
      .and_then(Path::parent)
      .filter(|rawdata| !rawdata.as_os_str().is_empty())
      .map(|rawdata| rawdata.join(Self::OBJECTS_DIRECTORY))
      .filter(|objects| objects.is_dir())
      .into_iter()
      .collect()
  }
}
