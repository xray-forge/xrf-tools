use std::path::{Path, PathBuf};

use xrf_gamemtl::GameMtlFile;

/// What a build reads besides the scene: the objects roots (`$objects$`) its library objects are found under, the
/// first holding each, the game material library their surfaces name, and the textures root (`$game_textures$`) the
/// detail objects' textures are merged from, which a scene of details needs.
#[derive(Clone, Copy, Debug)]
pub struct LevelBuildSources<'a> {
  pub objects: &'a [PathBuf],
  pub game_materials: &'a GameMtlFile,
  pub textures: Option<&'a Path>,
}
