use std::path::PathBuf;

use xrf_utils::HostDestination;

use crate::level::scene::level_scene_texture_file::LevelSceneTextureFile;

/// Where a decompile writes, every path resolved below its root and checked against the root's link policy before
/// anything is written or removed ([`crate::LevelSceneDecompile`]'s `check_write`).
pub(crate) struct LevelSceneWrite<'a> {
  pub(crate) level_path: PathBuf,
  pub(crate) scene_root: HostDestination,
  pub(crate) objects_root: HostDestination,
  pub(crate) textures_root: HostDestination,
  /// The scene's main file, its sidecar and its parts.
  pub(crate) scene: Vec<PathBuf>,
  pub(crate) objects: Vec<PathBuf>,
  /// Each library copy and its thumbnail.
  pub(crate) library: Vec<(PathBuf, PathBuf)>,
  pub(crate) textures: Vec<(&'a LevelSceneTextureFile, PathBuf)>,
}
