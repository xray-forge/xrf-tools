use byteorder::ByteOrder;
use xrf_error::XrfResult;
use xrf_level::LevelDetailsFile;

use crate::level::level_detail_conversion::{LevelDetailConversion, LevelDetailDecompile};
use crate::level::level_detail_library::LevelDetailLibrary;
use crate::level::scene::level_decompile_options::LevelDecompileOptions;
use crate::level::scene::level_scene_draft::LevelSceneDraft;
use crate::level::scene::level_scene_sources::LevelSceneSources;
use crate::level::scene::steps::level_scene_step::LevelSceneStep;
use crate::provenance::level_scene_provenance::LevelSceneInference;

/// The level's details ([`LevelDetailConversion`]): the detail object tool, each model found among the detail objects
/// of the options' object libraries.
pub(crate) struct LevelDetailStep;

impl LevelSceneStep for LevelDetailStep {
  const FILES: &'static [&'static str] = &[LevelDetailsFile::FILE_NAME];

  fn decompile<T: ByteOrder>(
    sources: &mut LevelSceneSources,
    options: &LevelDecompileOptions,
    draft: &mut LevelSceneDraft,
  ) -> XrfResult {
    let Some(file) = sources.take_read(LevelDetailsFile::FILE_NAME, LevelDetailsFile::read_from_bytes::<T>)? else {
      return Ok(());
    };
    let library: LevelDetailLibrary = LevelDetailLibrary::read_from_roots::<T>(&options.libraries)?;
    let decompile: LevelDetailDecompile = LevelDetailConversion::decompile(&file, &library);

    draft.refused.extend(decompile.refused);
    draft.infer_if(decompile.tool.is_some(), &[LevelSceneInference::DetailObjects]);

    if let Some(tool) = decompile.tool {
      draft.add(tool.to_scene_tool::<T>(draft.context.guid)?, Self::FILES, sources);
    }

    Ok(())
  }
}
