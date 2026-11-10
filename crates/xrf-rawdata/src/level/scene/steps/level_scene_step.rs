use byteorder::ByteOrder;
use xrf_error::XrfResult;

use crate::level::scene::level_decompile_options::LevelDecompileOptions;
use crate::level::scene::level_scene_draft::LevelSceneDraft;
use crate::level::scene::level_scene_sources::LevelSceneSources;

/// One step of a level's decompile: the files of the level's directory it reads, and how it turns those it finds into
/// the draft's parts, objects and level options. The decompile reads the files every step declares, hashes them into
/// the scene's GUID, and runs the steps in order over one draft, so a step's files are named once, here.
pub(crate) trait LevelSceneStep {
  /// The files the step reads, by their names in the level's directory.
  const FILES: &'static [&'static str];

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
