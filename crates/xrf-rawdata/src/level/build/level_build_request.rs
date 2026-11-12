use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::level::build::level_build_impostor_limit::LevelBuildImpostorLimit;

/// A scene's build into a level directory, as a front-end asks for it: the game files always, and the level compiler's
/// files (`build.prj`, the occluders, `level.ltx`, details, AI map, impostors, breakables) where the game material
/// library is named, the objects read from the objects roots.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelBuildRequest {
  /// The scene's main file, `<name>.level`, its parts in the directory beside it named after it.
  pub source: PathBuf,
  /// The level directory the files are written into.
  pub destination: PathBuf,
  /// The objects roots (`$objects$`) the scene's objects are read from, each from the first holding it.
  pub objects: Vec<PathBuf>,
  /// The game material library, `gamemtl.xr`; the level compiler's files are written only where it is named.
  pub game_materials: Option<PathBuf>,
  /// The textures root (`$game_textures$`) the details' textures and the impostors' strips are read from.
  pub textures: Option<PathBuf>,
  /// Whether `build.prj` is lit at proof quality ([`crate::LevelBuildLighting::PROOF`]) whatever the scene sets.
  pub is_proof_lighting: bool,
  pub impostor_limit: LevelBuildImpostorLimit,
  /// Whether files the build writes are overwritten where the directory holds them rather than refused.
  pub is_forced: bool,
}
