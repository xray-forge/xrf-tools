use byteorder::ByteOrder;
use xrf_editable_level::EditableAiMapTool;
use xrf_error::XrfResult;
use xrf_level::{LevelAiFile, LevelAiNodes};

use crate::level::level_ai_map_conversion::LevelAiMapConversion;
use crate::level::scene::level_decompile_options::LevelDecompileOptions;
use crate::level::scene::level_scene_draft::LevelSceneDraft;
use crate::level::scene::level_scene_sources::LevelSceneSources;
use crate::level::scene::steps::level_scene_step::LevelSceneStep;
use crate::provenance::level_scene_provenance::LevelSceneInference;

/// The level's AI map ([`LevelAiMapConversion`]): the AI map tool, where the directory holds a `level.ai`.
pub(crate) struct LevelAiMapStep;

impl LevelSceneStep for LevelAiMapStep {
  const FILES: &'static [&'static str] = &[LevelAiFile::FILE_NAME];

  fn decompile<T: ByteOrder>(
    sources: &mut LevelSceneSources,
    _options: &LevelDecompileOptions,
    draft: &mut LevelSceneDraft,
  ) -> XrfResult {
    let Some((file, nodes)): Option<(LevelAiFile, LevelAiNodes)> =
      sources.take_read(LevelAiFile::FILE_NAME, LevelAiFile::read_with_nodes_from_bytes::<T>)?
    else {
      return Ok(());
    };
    let tool: EditableAiMapTool = LevelAiMapConversion::decompile(&file.header, nodes.iter_nodes::<T>());

    draft.infer_if(!tool.nodes.is_empty(), &[LevelSceneInference::AiMapSettings]);
    draft.add(tool.to_scene_tool::<T>(draft.context.guid)?, Self::FILES, sources);

    Ok(())
  }
}
