use std::borrow::Borrow;
use std::collections::BTreeMap;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use byteorder::ByteOrder;
use xrf_chunk::ChunkWriter;
use xrf_editable_level::{
  EditableFogVolumeObject, EditableLevelGuid, EditableLevelScene, EditableObjectPart, EditableParticleObject,
  EditableSoundEnvironmentObject, EditableSoundSourceObject, EditableSpawnPointObject, EditableToolObject,
  EditableToolObjects, EditableWallmarkTool, EditableWayObject,
};
use xrf_error::{XrfError, XrfResult};
use xrf_level::{
  LevelEnvModFile, LevelFogVolFile, LevelGameFile, LevelPsStaticFile, LevelSndEnvFile, LevelSndStaticFile,
  LevelWallmarksFile,
};
use xrf_spawn::{AlifeObject, LevelSpawnFile, LevelSpawnObject};
use xrf_utils::{StagedFile, format_path};

use crate::level::build::level_exported_file::LevelExportedFile;
use crate::level::build::level_exported_previous::LevelExportedPrevious;
use crate::level::entities::level_entity_conversion::LevelEntityConversion;
use crate::level::entities::level_entity_export::LevelEntityExport;
use crate::level::parts::level_fog_volume_conversion::LevelFogVolumeConversion;
use crate::level::parts::level_particle_conversion::LevelParticleConversion;
use crate::level::parts::level_sound_environment_conversion::LevelSoundEnvironmentConversion;
use crate::level::parts::level_sound_source_conversion::LevelSoundSourceConversion;
use crate::level::parts::level_spawn_point_conversion::LevelSpawnPointConversion;
use crate::level::parts::level_wallmark_conversion::LevelWallmarkConversion;
use crate::level::parts::level_way_conversion::LevelWayConversion;

/// Exports a scene's parts into the level files the Level Editor's Build writes from them with no compiler between:
/// `SceneBuilder::BuildGame` (`BuilderGame.cpp:17`) and `BuildWallmarks` (`BuilderOGF.cpp:172`), plus `level.snd_env`,
/// whose export the SDK leaves commented out. The inverse of
/// [`LevelSceneDecompiler`](crate::LevelSceneDecompiler) for the files it reads.
pub struct LevelSceneExporter;

impl LevelSceneExporter {
  /// Every file [`Self::export`] may write, sorted by name.
  pub const FILE_NAMES: [&'static str; 9] = [
    LevelEnvModFile::FILE_NAME,
    LevelFogVolFile::FILE_NAME,
    LevelGameFile::FILE_NAME,
    LevelPsStaticFile::FILE_NAME,
    LevelSndEnvFile::FILE_NAME,
    LevelSndStaticFile::FILE_NAME,
    LevelEntityConversion::SPAWN_FILE_NAME,
    LevelWallmarksFile::FILE_NAME,
    LevelEntityConversion::RESPAWN_FILE_NAME,
  ];

  /// Each file the build writes from the scene, by name, as written in `T`'s byte order:
  ///
  /// - `level.game`, `level.env_mod` and `level.fog_vol` for every scene, from its way and spawn parts and its fog
  ///   volumes, each empty where the scene has no such tool;
  /// - `level.spawn` and `level_rs.spawn` where the spawn part places an entity there, `level.spawn` opening with
  ///   `built`, the entities a build of the geometry cut from it ([`LevelBuildExport::entities`](crate::LevelBuildExport)),
  ///   as the object tool exports before the spawn tool;
  /// - `level.ps_static` and `level.snd_static` where the scene places a particle or a sound;
  /// - `level.snd_env` where it holds a sound environment;
  /// - `level.wallmarks` where its wall mark tool exports a slot (the build exports only a tool holding slots).
  ///
  /// # Errors
  ///
  /// Returns an error when the scene's GUID or a tool's part does not read, or a conversion refuses what a part holds.
  pub fn export<T: ByteOrder>(
    scene: &EditableLevelScene,
    built: &[AlifeObject],
  ) -> XrfResult<BTreeMap<&'static str, Vec<u8>>> {
    let guid: EditableLevelGuid = scene.level.get_guid()?;
    let mut files: BTreeMap<&'static str, Vec<u8>> = BTreeMap::new();
    let spawn: EditableObjectPart<EditableSpawnPointObject> = Self::get_part(scene, guid)?;

    files.insert(
      LevelGameFile::FILE_NAME,
      Self::encode(|writer| {
        LevelGameFile {
          rpoints: LevelSpawnPointConversion::export_respawn_points(&spawn),
          ways: LevelWayConversion::export(&Self::get_part::<EditableWayObject>(scene, guid)?)?,
        }
        .write::<T>(writer)
      })?,
    );
    files.insert(
      LevelEnvModFile::FILE_NAME,
      Self::encode(|writer| LevelSpawnPointConversion::export_environment_modifiers(&spawn).write::<T>(writer))?,
    );

    let entities: LevelEntityExport = LevelEntityConversion::export::<T>(&spawn)?;
    let spawn_list: Option<LevelSpawnFile> = Self::prepend_built::<T>(built, entities.spawn)?;

    for (name, list) in [
      (LevelEntityConversion::SPAWN_FILE_NAME, spawn_list),
      (LevelEntityConversion::RESPAWN_FILE_NAME, entities.respawn),
    ] {
      if let Some(list) = list {
        files.insert(name, Self::encode(|writer| list.write::<T>(writer))?);
      }
    }

    let fog: LevelFogVolFile =
      LevelFogVolumeConversion::export(&Self::get_part::<EditableFogVolumeObject>(scene, guid)?)?;

    files.insert(
      LevelFogVolFile::FILE_NAME,
      Self::encode(|writer| fog.write::<T>(writer))?,
    );

    if let Some(file) = LevelParticleConversion::export(&Self::get_part::<EditableParticleObject>(scene, guid)?) {
      files.insert(
        LevelPsStaticFile::FILE_NAME,
        Self::encode(|writer| file.write::<T>(writer))?,
      );
    }

    if let Some(file) = LevelSoundSourceConversion::export(&Self::get_part::<EditableSoundSourceObject>(scene, guid)?) {
      files.insert(
        LevelSndStaticFile::FILE_NAME,
        Self::encode(|writer| file.write::<T>(writer))?,
      );
    }

    if let Some(file) =
      LevelSoundEnvironmentConversion::export(&Self::get_part::<EditableSoundEnvironmentObject>(scene, guid)?)?
    {
      files.insert(
        LevelSndEnvFile::FILE_NAME,
        Self::encode(|writer| file.write::<T>(writer))?,
      );
    }

    if let Some(tool) = scene.get_tool(EditableWallmarkTool::TOOL_NAME) {
      let file: LevelWallmarksFile =
        LevelWallmarkConversion::export(&EditableWallmarkTool::read_from_scene_tool::<T>(tool)?);

      if !file.slots.is_empty() {
        files.insert(
          LevelWallmarksFile::FILE_NAME,
          Self::encode(|writer| file.write::<T>(writer))?,
        );
      }
    }

    Ok(files)
  }

  /// Writes an export's files into a level directory, each staged, saying of each what it replaced. Whether files
  /// already there may be replaced, and which of the build's own it no longer writes go, is the caller's
  /// ([`LevelBuildRequest::get_outputs`](crate::LevelBuildRequest::get_outputs)).
  ///
  /// # Errors
  ///
  /// Returns an error when a file there cannot be read or one cannot be written.
  pub fn write_to_directory<K: Borrow<str> + Ord>(
    files: &BTreeMap<K, Vec<u8>>,
    directory: &Path,
  ) -> XrfResult<Vec<LevelExportedFile>> {
    let mut written: Vec<LevelExportedFile> = Vec::with_capacity(files.len());

    for (name, bytes) in files {
      let name: &str = name.borrow();
      let path: PathBuf = directory.join(name);
      let previous: LevelExportedPrevious = match std::fs::read(&path) {
        Ok(existing) if existing == *bytes => LevelExportedPrevious::Same,
        Ok(_) => LevelExportedPrevious::Changed,
        Err(error) if error.kind() == ErrorKind::NotFound => LevelExportedPrevious::New,
        Err(error) => {
          return Err(XrfError::new_read_error(format!(
            "{} was not read: {error}",
            format_path(&path)
          )));
        }
      };

      StagedFile::write_creating_directories(&path, bytes).map_err(|error| {
        XrfError::new_io_error(format!("{} was not written: {error}", format_path(&path)), error.kind())
      })?;
      written.push(LevelExportedFile {
        name: name.to_owned(),
        size: bytes.len(),
        previous,
      });
    }

    Ok(written)
  }

  /// The scene's part of tool `O` as the editor holds it once loaded ([`EditableToolObjects::into_part`]): an empty
  /// one where the scene has no such tool, its objects the groups' members of the tool's class, then the part's own.
  pub(crate) fn get_part<O: EditableToolObject>(
    scene: &EditableLevelScene,
    guid: EditableLevelGuid,
  ) -> XrfResult<EditableObjectPart<O>> {
    Ok(EditableToolObjects::<O>::read(scene, guid)?.into_part())
  }

  /// `list` with `built` before its entities, every one numbered again in order; none where both are empty.
  fn prepend_built<T: ByteOrder>(
    built: &[AlifeObject],
    list: Option<LevelSpawnFile>,
  ) -> XrfResult<Option<LevelSpawnFile>> {
    if built.is_empty() {
      return Ok(list);
    }

    let mut objects: Vec<LevelSpawnObject> = built
      .iter()
      .zip(0u32..)
      .map(|(entity, id)| LevelSpawnObject::from_alife_object::<T>(id, entity))
      .collect::<XrfResult<_>>()?;

    for (mut object, id) in list
      .map(|list| list.objects)
      .unwrap_or_default()
      .into_iter()
      .zip(u32::try_from(objects.len()).unwrap_or(u32::MAX)..)
    {
      object.id = id;
      objects.push(object);
    }

    Ok(Some(LevelSpawnFile { objects }))
  }

  /// The bytes `write` puts into a new chunk writer.
  fn encode(write: impl FnOnce(&mut ChunkWriter) -> XrfResult) -> XrfResult<Vec<u8>> {
    let mut writer: ChunkWriter = ChunkWriter::new();

    write(&mut writer)?;

    Ok(writer.buffer)
  }
}
