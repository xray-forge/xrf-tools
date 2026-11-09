use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use byteorder::ByteOrder;
use sha2::{Digest, Sha256};
use xrf_editable_level::{
  EditableBuildParams, EditableLevelCamera, EditableLevelFile, EditableLevelGuid, EditableLevelOptions,
  EditableLevelScene, EditableSceneTool, EditableSpawnPointObject,
};
use xrf_error::{XrfError, XrfResult};
use xrf_level::{
  LevelEnvModFile, LevelFogVolFile, LevelGameFile, LevelGameRPoint, LevelPsStaticFile, LevelSndEnvFile,
  LevelSndStaticFile, LevelWallmarksFile,
};
use xrf_utils::{StagedFile, format_path};

use crate::level::level_decompile_context::LevelDecompileContext;
use crate::level::level_fog_volume_conversion::LevelFogVolumeConversion;
use crate::level::level_particle_conversion::LevelParticleConversion;
use crate::level::level_scene_exporter::LevelSceneExporter;
use crate::level::level_sound_environment_conversion::LevelSoundEnvironmentConversion;
use crate::level::level_sound_source_conversion::LevelSoundSourceConversion;
use crate::level::level_spawn_point_conversion::LevelSpawnPointConversion;
use crate::level::level_wallmark_conversion::LevelWallmarkConversion;
use crate::level::level_way_conversion::LevelWayConversion;
use crate::provenance::level_scene_provenance::{
  LevelSceneFile, LevelSceneInference, LevelSceneProvenance, LevelSceneSource,
};

/// What a level's decompile takes besides the level's directory (decision 21's one options type; the root spawn,
/// libraries, surfaces mode and output root join it as their stages land).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LevelDecompileOptions {
  /// The level's name: its scene's main file is `<name>.level`, its parts are in `<name>/`, and its level options
  /// build into and prefix with it.
  pub name: String,
  /// The front-end decompiling, such as `xrf-cli level decompile`, and its version, which the sidecar records.
  pub tool: String,
  pub tool_version: String,
}

/// A decompiled level: its scene and the provenance written beside it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LevelSceneDecompile {
  pub scene: EditableLevelScene,
  pub provenance: LevelSceneProvenance,
}

impl LevelSceneDecompile {
  /// What a write to `level_path` would replace, of the scene's main file, its part directory and its sidecar: what a
  /// front-end asks about before writing over an earlier decompile or an authored scene.
  pub fn get_existing_paths(level_path: &Path) -> Vec<PathBuf> {
    [
      level_path.to_path_buf(),
      EditableLevelScene::get_parts_directory(level_path),
      LevelSceneProvenance::sidecar_path(level_path),
    ]
    .into_iter()
    .filter(|path| path.exists())
    .collect()
  }

  /// Writes the scene's sidecar (`<level_path>.xrf.json`), then the scene: its main file at `level_path` and its parts
  /// in the directory beside it. The sidecar goes first, so a scene that fails to write leaves a stray sidecar, never a
  /// decompile taken for an authored scene.
  ///
  /// # Errors
  ///
  /// Returns an error, writing nothing, when [`EditableLevelScene::check_writable`] refuses the scene; when the sidecar
  /// cannot be written; or as [`EditableLevelScene::write_to_path`] does.
  pub fn write_to_path<T: ByteOrder>(&self, level_path: &Path) -> XrfResult {
    let sidecar: PathBuf = LevelSceneProvenance::sidecar_path(level_path);

    // Refused before the sidecar is written, so a sidecar never lands beside an authored scene left as it was.
    self.scene.check_writable(level_path)?;

    StagedFile::write_creating_directories(&sidecar, self.provenance.to_json()?.as_bytes()).map_err(|error| {
      XrfError::new_io_error(
        format!("{} was not written: {error}", format_path(&sidecar)),
        error.kind(),
      )
    })?;

    self.scene.write_to_path::<T, _>(&level_path)
  }
}

/// Decompiles a level's directory into a Level Editor scene: every file the build passes through from a part
/// (`BuilderGame.cpp`) back into its part, through one [`LevelDecompileContext`], under a scene GUID hashed from those
/// files, so decompiling the same level twice gives the same scene (decision 22). The scene holds no geometry yet.
pub struct LevelSceneDecompiler;

impl LevelSceneDecompiler {
  /// The files of a level's directory the decompile reads, the ones [`LevelSceneExporter`] writes back, in the order
  /// their bytes are hashed into the scene's GUID.
  pub const SOURCE_FILES: [&'static str; 7] = LevelSceneExporter::FILE_NAMES;

  /// The owner the scene's `[level_tag]` names.
  pub const OWNER: &'static str = "xrf";

  /// The scene of the level in `directory`: a part per tool whose compiled file the directory holds (the spawn tool's
  /// from `level.game` and `level.env_mod`), and the main file its name and those files give.
  ///
  /// # Errors
  ///
  /// Returns an error when the directory holds none of [`Self::SOURCE_FILES`], one cannot be read or is not the file
  /// its name says, or a conversion refuses what it holds.
  pub fn decompile<T: ByteOrder>(directory: &Path, options: &LevelDecompileOptions) -> XrfResult<LevelSceneDecompile> {
    let sources: BTreeMap<&'static str, Vec<u8>> = Self::read_sources(directory)?;
    let mut context: LevelDecompileContext = LevelDecompileContext::new(Self::get_guid(&sources));
    let mut tools: Vec<(EditableSceneTool, Vec<&'static str>)> = Vec::new();
    let mut inferred: Vec<LevelSceneInference> = vec![
      LevelSceneInference::ObjectNames,
      LevelSceneInference::LevelOptions,
      LevelSceneInference::GameTypes,
      LevelSceneInference::BuildParameters,
      LevelSceneInference::Camera,
    ];
    let bytes = |name: &str| sources.get(name).cloned();

    if let Some(bytes) = bytes(LevelPsStaticFile::FILE_NAME) {
      let file: LevelPsStaticFile = LevelPsStaticFile::read_from_bytes::<T>(bytes)?;

      tools.push((
        LevelParticleConversion::decompile(&file, &mut context)?.to_scene_tool()?,
        vec![LevelPsStaticFile::FILE_NAME],
      ));
    }

    if let Some(bytes) = bytes(LevelSndStaticFile::FILE_NAME) {
      let file: LevelSndStaticFile = LevelSndStaticFile::read_from_bytes::<T>(bytes)?;

      inferred.push(LevelSceneInference::SoundSourceDistances);
      tools.push((
        LevelSoundSourceConversion::decompile(&file, &mut context)?.to_scene_tool()?,
        vec![LevelSndStaticFile::FILE_NAME],
      ));
    }

    if let Some(bytes) = bytes(LevelSndEnvFile::FILE_NAME) {
      let file: LevelSndEnvFile = LevelSndEnvFile::read_from_bytes::<T>(bytes)?;

      tools.push((
        LevelSoundEnvironmentConversion::decompile(&file, &mut context)?.to_scene_tool()?,
        vec![LevelSndEnvFile::FILE_NAME],
      ));
    }

    if let Some(bytes) = bytes(LevelFogVolFile::FILE_NAME) {
      let file: LevelFogVolFile = LevelFogVolFile::read_from_bytes::<T>(bytes)?;

      tools.push((
        LevelFogVolumeConversion::decompile(&file, &mut context)?.to_scene_tool()?,
        vec![LevelFogVolFile::FILE_NAME],
      ));
    }

    if let Some(bytes) = bytes(LevelWallmarksFile::FILE_NAME) {
      let file: LevelWallmarksFile = LevelWallmarksFile::read_from_bytes::<T>(bytes)?;

      inferred.push(LevelSceneInference::WallmarkBrush);
      tools.push((
        LevelWallmarkConversion::decompile(&file)?.to_scene_tool::<T>(context.guid)?,
        vec![LevelWallmarksFile::FILE_NAME],
      ));
    }

    let game: Option<LevelGameFile> = bytes(LevelGameFile::FILE_NAME)
      .map(LevelGameFile::read_from_bytes::<T>)
      .transpose()?;
    let environment: Option<LevelEnvModFile> = bytes(LevelEnvModFile::FILE_NAME)
      .map(LevelEnvModFile::read_from_bytes::<T>)
      .transpose()?;

    if let Some(game) = &game {
      tools.push((
        LevelWayConversion::decompile(&game.ways, &mut context)?.to_scene_tool()?,
        vec![LevelGameFile::FILE_NAME],
      ));
    }

    let respawn_points = game.as_ref().map(|game| game.rpoints.as_slice()).unwrap_or_default();

    if game.is_some() || environment.is_some() {
      let empty: LevelEnvModFile = LevelEnvModFile {
        version: EditableSpawnPointObject::VERSION,
        modifiers: Vec::new(),
      };
      let spawn_sources: Vec<&'static str> = [LevelGameFile::FILE_NAME, LevelEnvModFile::FILE_NAME]
        .into_iter()
        .filter(|name| sources.contains_key(name))
        .collect();

      inferred.push(LevelSceneInference::EntitiesNotDecompiled);
      tools.push((
        LevelSpawnPointConversion::decompile(respawn_points, environment.as_ref().unwrap_or(&empty), &mut context)?
          .to_scene_tool()?,
        spawn_sources,
      ));
    }

    tools.sort_by(|(first, _), (second, _)| first.name.cmp(&second.name));

    let files: Vec<LevelSceneFile> = std::iter::once(LevelSceneFile {
      written: format!("{}.{}", options.name, EditableLevelFile::EXTENSION),
      sources: Vec::new(),
    })
    .chain(tools.iter().map(|(tool, sources)| LevelSceneFile {
      written: format!("{}/{}", options.name, tool.get_file_name(0)),
      sources: sources.iter().map(|name| (*name).to_owned()).collect(),
    }))
    .collect();
    let level: EditableLevelFile = EditableLevelFile::new(
      &context.guid,
      &EditableLevelOptions::new(&options.name, Self::get_game_types(respawn_points)),
      &EditableBuildParams {
        weld_distance: 0.0,
        ..EditableBuildParams::DEFAULT
      },
      &EditableLevelCamera::default(),
      Self::OWNER,
      0,
    );

    Ok(LevelSceneDecompile {
      scene: EditableLevelScene {
        level,
        tools: tools.into_iter().map(|(tool, _)| tool).collect(),
      },
      provenance: LevelSceneProvenance::new(
        options.name.clone(),
        options.tool.clone(),
        options.tool_version.clone(),
        sources
          .iter()
          .map(|(name, bytes)| LevelSceneSource {
            file: (*name).to_owned(),
            sha256: Sha256::digest(bytes).iter().map(|byte| format!("{byte:02x}")).collect(),
          })
          .collect(),
        files,
        inferred,
      ),
    })
  }

  /// The source files the directory holds, by name.
  fn read_sources(directory: &Path) -> XrfResult<BTreeMap<&'static str, Vec<u8>>> {
    let mut sources: BTreeMap<&'static str, Vec<u8>> = BTreeMap::new();

    for name in Self::SOURCE_FILES {
      let path: PathBuf = directory.join(name);

      if path.is_file() {
        sources.insert(
          name,
          std::fs::read(&path)
            .map_err(|error| XrfError::new_read_error(format!("{} was not read: {error}", format_path(&path))))?,
        );
      }
    }

    if sources.is_empty() {
      return Err(XrfError::new_not_found_error(format!(
        "{} holds none of the level files a scene decompiles from",
        format_path(directory)
      )));
    }

    Ok(sources)
  }

  /// The scene's GUID: the first 16 bytes of the SHA-256 of each source's name, a zero, its length and its bytes, in
  /// [`Self::SOURCE_FILES`]' order.
  fn get_guid(sources: &BTreeMap<&'static str, Vec<u8>>) -> EditableLevelGuid {
    let mut hash = Sha256::new();

    for name in Self::SOURCE_FILES {
      if let Some(bytes) = sources.get(name) {
        hash.update(name.as_bytes());
        hash.update([0]);
        hash.update((bytes.len() as u64).to_le_bytes());
        hash.update(bytes);
      }
    }

    let digest = hash.finalize();
    let half = |offset: usize| {
      let mut bytes: [u8; 8] = [0; 8];

      bytes.copy_from_slice(&digest[offset..offset + 8]);
      u64::from_le_bytes(bytes)
    };

    EditableLevelGuid {
      g0: half(0),
      g1: half(8),
    }
  }

  /// The game modes the level serves: those its respawn points answer to, or single player for a level of none.
  fn get_game_types(respawn_points: &[LevelGameRPoint]) -> u16 {
    respawn_points
      .iter()
      .map(|point| point.game_types)
      .reduce(|types, point| types | point)
      .unwrap_or(EditableLevelOptions::SINGLE_PLAYER)
  }
}
