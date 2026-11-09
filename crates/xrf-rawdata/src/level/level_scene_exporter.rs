use std::collections::BTreeMap;

use byteorder::ByteOrder;
use xrf_chunk::ChunkWriter;
use xrf_editable_level::{
  EditableFogVolumeObject, EditableLevelGuid, EditableLevelScene, EditableObjectPart, EditableParticleObject,
  EditableSoundEnvironmentObject, EditableSoundSourceObject, EditableSpawnPointObject, EditableToolObject,
  EditableWallmarkTool, EditableWayObject,
};
use xrf_error::XrfResult;
use xrf_level::{
  LevelEnvModFile, LevelFogVolFile, LevelGameFile, LevelPsStaticFile, LevelSndEnvFile, LevelSndStaticFile,
  LevelWallmarksFile,
};

use crate::level::level_fog_volume_conversion::LevelFogVolumeConversion;
use crate::level::level_particle_conversion::LevelParticleConversion;
use crate::level::level_sound_environment_conversion::LevelSoundEnvironmentConversion;
use crate::level::level_sound_source_conversion::LevelSoundSourceConversion;
use crate::level::level_spawn_point_conversion::LevelSpawnPointConversion;
use crate::level::level_wallmark_conversion::LevelWallmarkConversion;
use crate::level::level_way_conversion::LevelWayConversion;

/// Exports a scene's parts into the level files the Level Editor's Build writes from them with no compiler between:
/// `SceneBuilder::BuildGame` (`BuilderGame.cpp:17`) and `BuildWallmarks` (`BuilderOGF.cpp:172`), plus `level.snd_env`,
/// whose export the SDK leaves commented out. The inverse of
/// [`LevelSceneDecompiler`](crate::LevelSceneDecompiler) for the files it reads.
pub struct LevelSceneExporter;

impl LevelSceneExporter {
  /// Each file the build writes from the scene, by name, as written in `T`'s byte order:
  ///
  /// - `level.game`, `level.env_mod` and `level.fog_vol` for every scene, from its way and spawn parts and its fog
  ///   volumes, each empty where the scene has no such tool;
  /// - `level.ps_static` and `level.snd_static` where the scene places a particle or a sound;
  /// - `level.snd_env` where it holds a sound environment;
  /// - `level.wallmarks` where its wall mark tool exports a slot (the build exports only a tool holding slots).
  ///
  /// # Errors
  ///
  /// Returns an error when the scene's GUID or a tool's part does not read, or a conversion refuses what a part holds.
  pub fn export<T: ByteOrder>(scene: &EditableLevelScene) -> XrfResult<BTreeMap<&'static str, Vec<u8>>> {
    let guid: EditableLevelGuid = scene.level.get_guid()?;
    let mut files: BTreeMap<&'static str, Vec<u8>> = BTreeMap::new();
    let spawn: EditableObjectPart<EditableSpawnPointObject> = Self::get_part(scene, guid)?;

    files.insert(
      LevelGameFile::FILE_NAME,
      Self::to_bytes(|writer| {
        LevelGameFile {
          rpoints: LevelSpawnPointConversion::export_respawn_points(&spawn),
          ways: LevelWayConversion::export(&Self::get_part::<EditableWayObject>(scene, guid)?)?,
        }
        .write::<T>(writer)
      })?,
    );
    files.insert(
      LevelEnvModFile::FILE_NAME,
      Self::to_bytes(|writer| LevelSpawnPointConversion::export_environment_modifiers(&spawn).write::<T>(writer))?,
    );

    let fog: LevelFogVolFile =
      LevelFogVolumeConversion::export(&Self::get_part::<EditableFogVolumeObject>(scene, guid)?)?;

    files.insert(
      LevelFogVolFile::FILE_NAME,
      Self::to_bytes(|writer| fog.write::<T>(writer))?,
    );

    if let Some(file) = LevelParticleConversion::export(&Self::get_part::<EditableParticleObject>(scene, guid)?) {
      files.insert(
        LevelPsStaticFile::FILE_NAME,
        Self::to_bytes(|writer| file.write::<T>(writer))?,
      );
    }

    if let Some(file) = LevelSoundSourceConversion::export(&Self::get_part::<EditableSoundSourceObject>(scene, guid)?) {
      files.insert(
        LevelSndStaticFile::FILE_NAME,
        Self::to_bytes(|writer| file.write::<T>(writer))?,
      );
    }

    if let Some(file) =
      LevelSoundEnvironmentConversion::export(&Self::get_part::<EditableSoundEnvironmentObject>(scene, guid)?)?
    {
      files.insert(
        LevelSndEnvFile::FILE_NAME,
        Self::to_bytes(|writer| file.write::<T>(writer))?,
      );
    }

    if let Some(tool) = scene.get_tool(EditableWallmarkTool::TOOL_NAME) {
      let file: LevelWallmarksFile =
        LevelWallmarkConversion::export(&EditableWallmarkTool::read_from_scene_tool::<T>(tool)?);

      if !file.slots.is_empty() {
        files.insert(
          LevelWallmarksFile::FILE_NAME,
          Self::to_bytes(|writer| file.write::<T>(writer))?,
        );
      }
    }

    Ok(files)
  }

  /// The scene's part of tool `O`, or an empty one where the scene has no such tool, as the editor holds every tool.
  fn get_part<O: EditableToolObject>(
    scene: &EditableLevelScene,
    guid: EditableLevelGuid,
  ) -> XrfResult<EditableObjectPart<O>> {
    match scene.get_tool(O::TOOL_NAME) {
      Some(tool) => EditableObjectPart::read_from_scene_tool(tool),
      None => Ok(EditableObjectPart::new(guid, Vec::new())),
    }
  }

  fn to_bytes(write: impl FnOnce(&mut ChunkWriter) -> XrfResult) -> XrfResult<Vec<u8>> {
    let mut writer: ChunkWriter = ChunkWriter::new();

    write(&mut writer)?;

    Ok(writer.buffer)
  }
}
