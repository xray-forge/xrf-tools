use byteorder::ByteOrder;
use xrf_error::XrfResult;

use crate::level::scene::level_decompile_options::LevelDecompileOptions;
use crate::level::scene::level_scene_draft::LevelSceneDraft;
use crate::level::scene::level_scene_listing::LevelSceneListing;
use crate::level::scene::level_scene_sources::LevelSceneSources;

/// One step of a level's decompile: the files it reads, and how it turns them into the draft's parts, objects and
/// level options.
///
/// The decompile reads and hashes every step's declared files into the scene's GUID before running the steps in order,
/// so a step names its files here and nowhere else.
pub(crate) trait LevelSceneStep {
  /// The files the step reads, by their names in the level's directory.
  const FILES: &'static [&'static str];

  /// The files the step reads by what their names hold, read and hashed with [`Self::FILES`] under their path in the
  /// level's directory, `<directory>/<name>`.
  const LISTINGS: &'static [LevelSceneListing] = &[];

  /// Adds what the step makes of `sources` to `draft`.
  ///
  /// # Errors
  ///
  /// Returns an error when a file it reads is not the file its name says, or a conversion refuses what it holds.
  fn decompile<T: ByteOrder>(
    sources: &mut LevelSceneSources,
    options: &LevelDecompileOptions,
    draft: &mut LevelSceneDraft,
  ) -> XrfResult;
}
