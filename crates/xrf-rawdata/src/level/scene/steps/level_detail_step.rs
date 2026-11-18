use byteorder::ByteOrder;
use xrf_error::XrfResult;
use xrf_level::LevelDetailsFile;

use crate::level::build::level_build_details::LevelBuildDetails;
use crate::level::level_detail_atlas::LevelDetailAtlas;
use crate::level::level_detail_conversion::{LevelDetailConversion, LevelDetailDecompile};
use crate::level::level_detail_library::LevelDetailLibrary;
use crate::level::scene::level_decompile_options::LevelDecompileOptions;
use crate::level::scene::level_scene_draft::LevelSceneDraft;
use crate::level::scene::level_scene_sources::LevelSceneSources;
use crate::level::scene::steps::level_scene_step::LevelSceneStep;
use crate::provenance::level_scene_provenance::{LevelSceneInference, LevelSceneRefusal};

/// The level's details ([`LevelDetailConversion`]): the detail object tool, each model found among the detail objects
/// of the options' object libraries, and their textures cut from the level's atlas, `build_details.dds`.
pub(crate) struct LevelDetailStep;

impl LevelSceneStep for LevelDetailStep {
  const FILES: &'static [&'static str] = &[LevelDetailsFile::FILE_NAME, LevelBuildDetails::ATLAS_FILE_NAME];

  fn decompile<T: ByteOrder>(
    sources: &mut LevelSceneSources,
    _options: &LevelDecompileOptions,
    draft: &mut LevelSceneDraft,
  ) -> XrfResult {
    let Some(file) = sources.take_read(LevelDetailsFile::FILE_NAME, LevelDetailsFile::read_from_bytes::<T>)? else {
      return Ok(());
    };
    let atlas: Option<LevelDetailAtlas> = Self::read_atlas(sources, draft);
    let library: LevelDetailLibrary =
      LevelDetailLibrary::read_from_library::<T>(&mut draft.object_library, &draft.job)?;
    let decompile: LevelDetailDecompile = LevelDetailConversion::decompile(&file, &library, atlas.as_ref())?;

    draft.refused.extend(decompile.refused);
    draft.textures.extend(decompile.textures);
    draft.infer_if(decompile.tool.is_some(), &[LevelSceneInference::DetailObjects]);

    if let Some(tool) = decompile.tool {
      draft.add(tool.to_scene_tool::<T>(draft.context.guid)?, Self::FILES, sources);
    }

    Ok(())
  }
}

impl LevelDetailStep {
  /// The details' atlas, where the directory holds it; one that does not read is refused, and the detail objects come
  /// back without their textures.
  fn read_atlas(sources: &mut LevelSceneSources, draft: &mut LevelSceneDraft) -> Option<LevelDetailAtlas> {
    LevelDetailAtlas::read_from_bytes(sources.take(LevelBuildDetails::ATLAS_FILE_NAME)?)
      .inspect_err(|error| {
        draft.refused.push(LevelSceneRefusal {
          file: String::from(LevelBuildDetails::ATLAS_FILE_NAME),
          object: String::from(LevelBuildDetails::ATLAS_FILE_NAME),
          reason: format!("The detail atlas was not read, so the detail objects come back without textures: {error}"),
        });
      })
      .ok()
  }
}
