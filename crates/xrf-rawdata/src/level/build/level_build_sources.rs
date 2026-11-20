use std::path::{Path, PathBuf};

use xrf_gamemtl::GameMtlFile;

use crate::level::build::level_build_impostor_limit::LevelBuildImpostorLimit;
use crate::level::scene::level_scene_decompile::LevelSceneDecompile;

/// What a build reads besides the scene: the objects roots (`$objects$`), each object read from the first holding it,
/// the game material library, and the textures root (`$game_textures$`), which details and impostors need.
#[derive(Clone, Copy, Debug)]
pub struct LevelBuildSources<'a> {
  pub objects: &'a [PathBuf],
  pub game_materials: &'a GameMtlFile,
  pub textures: Option<&'a Path>,
  pub impostor_limit: LevelBuildImpostorLimit,
  /// The decompile the scene came from, when built without being written: its objects and textures win over any root.
  pub held: Option<&'a LevelSceneDecompile>,
}
