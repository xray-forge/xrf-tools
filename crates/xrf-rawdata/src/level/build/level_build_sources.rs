use std::path::{Path, PathBuf};

use xrf_gamemtl::GameMtlFile;

use crate::level::build::level_build_impostor_limit::LevelBuildImpostorLimit;
use crate::level::build::level_build_lighting::LevelBuildLighting;
use crate::level::scene::level_scene_decompile::LevelSceneDecompile;

/// What a build reads besides the scene, and how it builds: the objects roots (`$objects$`) its library objects are
/// found under, the first holding each, the game material library their surfaces name, the textures root
/// (`$game_textures$`) the detail objects' textures are merged from, which a scene of details needs, the lighting to
/// build with in place of the scene's, where one is given, and how large the impostor atlas may grow.
#[derive(Clone, Copy, Debug)]
pub struct LevelBuildSources<'a> {
  pub objects: &'a [PathBuf],
  pub game_materials: &'a GameMtlFile,
  pub textures: Option<&'a Path>,
  pub lighting: Option<LevelBuildLighting>,
  pub impostor_limit: LevelBuildImpostorLimit,
  /// The decompile the scene came from, where it is built without being written: its objects and textures taken
  /// before any root.
  pub held: Option<&'a LevelSceneDecompile>,
}
