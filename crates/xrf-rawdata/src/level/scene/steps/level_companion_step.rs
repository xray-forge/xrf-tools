use byteorder::ByteOrder;
use xrf_editable_level::EditableSceneTool;
use xrf_error::XrfResult;
use xrf_level::{LevelFogVolFile, LevelPsStaticFile, LevelSndEnvFile, LevelSndStaticFile, LevelWallmarksFile};

use crate::level::level_fog_volume_conversion::LevelFogVolumeConversion;
use crate::level::level_particle_conversion::LevelParticleConversion;
use crate::level::level_sound_environment_conversion::LevelSoundEnvironmentConversion;
use crate::level::level_sound_source_conversion::LevelSoundSourceConversion;
use crate::level::level_wallmark_conversion::LevelWallmarkConversion;
use crate::level::scene::level_decompile_options::LevelDecompileOptions;
use crate::level::scene::level_scene_draft::LevelSceneDraft;
use crate::level::scene::level_scene_sources::LevelSceneSources;
use crate::level::scene::steps::level_scene_step::LevelSceneStep;
use crate::provenance::level_scene_provenance::LevelSceneInference;

/// The files the build writes each straight from one part (`BuilderGame.cpp`), each back into its part: particles,
/// sounds, sound environments, fog volumes and wall marks.
pub(crate) struct LevelCompanionStep;

impl LevelSceneStep for LevelCompanionStep {
  const FILES: &'static [&'static str] = &[
    LevelPsStaticFile::FILE_NAME,
    LevelSndStaticFile::FILE_NAME,
    LevelSndEnvFile::FILE_NAME,
    LevelFogVolFile::FILE_NAME,
    LevelWallmarksFile::FILE_NAME,
  ];

  fn decompile<T: ByteOrder>(
    sources: &mut LevelSceneSources,
    _options: &LevelDecompileOptions,
    draft: &mut LevelSceneDraft,
  ) -> XrfResult {
    if let Some(file) = sources.take_read(LevelPsStaticFile::FILE_NAME, LevelPsStaticFile::read_from_bytes::<T>)? {
      let part = LevelParticleConversion::decompile(&file, &mut draft.context)?;

      draft.add(part.to_scene_tool()?, &[LevelPsStaticFile::FILE_NAME], sources);
    }

    if let Some(file) = sources.take_read(LevelSndStaticFile::FILE_NAME, LevelSndStaticFile::read_from_bytes::<T>)? {
      let part = LevelSoundSourceConversion::decompile(&file, &mut draft.context)?;

      draft.infer_if(!part.objects.is_empty(), &[LevelSceneInference::SoundSourceDistances]);
      draft.add(part.to_scene_tool()?, &[LevelSndStaticFile::FILE_NAME], sources);
    }

    if let Some(file) = sources.take_read(LevelSndEnvFile::FILE_NAME, LevelSndEnvFile::read_from_bytes::<T>)? {
      let part = LevelSoundEnvironmentConversion::decompile(&file, &mut draft.context)?;

      draft.add(part.to_scene_tool()?, &[LevelSndEnvFile::FILE_NAME], sources);
    }

    if let Some(file) = sources.take_read(LevelFogVolFile::FILE_NAME, LevelFogVolFile::read_from_bytes::<T>)? {
      let part = LevelFogVolumeConversion::decompile(&file, &mut draft.context)?;

      draft.add(part.to_scene_tool()?, &[LevelFogVolFile::FILE_NAME], sources);
    }

    if let Some(file) = sources.take_read(LevelWallmarksFile::FILE_NAME, LevelWallmarksFile::read_from_bytes::<T>)? {
      let tool = LevelWallmarkConversion::decompile(&file)?;
      let scene_tool: EditableSceneTool = tool.to_scene_tool::<T>(draft.context.guid)?;

      draft.infer_if(!tool.slots.is_empty(), &[LevelSceneInference::WallmarkBrush]);
      draft.add(scene_tool, &[LevelWallmarksFile::FILE_NAME], sources);
    }

    Ok(())
  }
}
