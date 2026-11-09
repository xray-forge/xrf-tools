use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use byteorder::ByteOrder;
use sha2::{Digest, Sha256};
use xrf_chunk::ChunkReader;
use xrf_editable_level::{
  EditableBuildParams, EditableLevelCamera, EditableLevelFile, EditableLevelGuid, EditableLevelOptions,
  EditableLevelScene, EditableSceneTool, EditableSpawnPointObject,
};
use xrf_error::{XrfError, XrfResult};
use xrf_level::{
  LevelEnvModFile, LevelFile, LevelFogVolFile, LevelGameFile, LevelGameRPoint, LevelLightsFile, LevelPsStaticFile,
  LevelSndEnvFile, LevelSndStaticFile, LevelWallmarksFile,
};
use xrf_spawn::{AlifeObject, LevelSpawnFile, SpawnFile, SpawnLevelObjects};
use xrf_utils::{StagedFile, format_path};

use crate::level::level_decompile_context::LevelDecompileContext;
use crate::level::level_entity_conversion::{LevelEntityConversion, LevelEntityDecompile, LevelEntitySources};
use crate::level::level_fog_volume_conversion::LevelFogVolumeConversion;
use crate::level::level_glow_conversion::LevelGlowConversion;
use crate::level::level_light_conversion::LevelLightConversion;
use crate::level::level_particle_conversion::LevelParticleConversion;
use crate::level::level_sound_environment_conversion::LevelSoundEnvironmentConversion;
use crate::level::level_sound_source_conversion::LevelSoundSourceConversion;
use crate::level::level_spawn_point_conversion::LevelSpawnPointConversion;
use crate::level::level_wallmark_conversion::LevelWallmarkConversion;
use crate::level::level_way_conversion::LevelWayConversion;
use crate::provenance::level_scene_provenance::{
  LevelSceneFile, LevelSceneInference, LevelSceneProvenance, LevelSceneRefusal, LevelSceneRootSpawn, LevelSceneSource,
};
use crate::provenance::sidecar::Sidecar;

/// What a level's decompile takes besides the level's directory (decision 21's one options type; libraries, surfaces
/// mode and output root join it as their stages land).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LevelDecompileOptions {
  /// The level's name: its scene's main file is `<name>.level`, its parts are in `<name>/`, and its level options
  /// build into and prefix with it.
  pub name: String,
  /// The front-end decompiling, such as `xrf-cli level decompile`, and its version, which the sidecar records.
  pub tool: String,
  pub tool_version: String,
  /// A root spawn (`all.spawn`) whose objects on this level stand in for `level.spawn`'s entities (decision 10). The
  /// level is found in it by its directory's name.
  pub root_spawn: Option<PathBuf>,
}

/// A root spawn read for one level: what the sidecar records of it, and its objects on the level.
struct LevelRootSpawnSource {
  record: LevelSceneRootSpawn,
  objects: Vec<AlifeObject>,
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
  /// The files of a level's directory the decompile reads, in the order their bytes are hashed into the scene's GUID:
  /// the compiled bundle (`level`, for its glows and dynamic lights) and the level compiler's `build.lights`, then the
  /// files [`crate::LevelSceneExporter::FILE_NAMES`] the build writes from the scene's parts and the export writes back.
  pub const SOURCE_FILES: [&'static str; 11] = [
    LevelFile::FILE_NAME,
    LevelLightsFile::FILE_NAME,
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

  /// The owner the scene's `[level_tag]` names.
  pub const OWNER: &'static str = "xrf";

  /// The scene of the level in `directory`: a part per tool whose compiled file the directory holds (the spawn tool's
  /// from `level.game`, `level.env_mod` and the spawn lists), and the main file its name and those files give.
  ///
  /// # Errors
  ///
  /// Returns an error when the directory holds none of [`Self::SOURCE_FILES`], one cannot be read or is not the file
  /// its name says, the root spawn given cannot be read or holds no objects chunk or graph, or a conversion refuses what
  /// it holds.
  pub fn decompile<T: ByteOrder>(directory: &Path, options: &LevelDecompileOptions) -> XrfResult<LevelSceneDecompile> {
    let mut sources: LevelSceneSources = LevelSceneSources::read(directory)?;
    let mut root: Option<LevelRootSpawnSource> = options
      .root_spawn
      .as_deref()
      .map(|path| Self::read_root_spawn::<T>(path, directory))
      .transpose()?;
    let guid: EditableLevelGuid = Self::get_guid(
      &sources.files,
      root
        .as_ref()
        .filter(|root| !root.objects.is_empty())
        .map(|root| root.record.sha256.as_str()),
    );
    let mut draft: LevelSceneDraft = LevelSceneDraft::new(guid, &options.name);

    Self::decompile_bundle::<T>(&mut sources, &mut draft)?;
    Self::decompile_companions::<T>(&mut sources, &mut draft)?;

    let game_types: u16 = Self::decompile_game::<T>(&mut sources, root.as_mut(), &mut draft)?;

    Ok(draft.finish(options, game_types, &sources, root.map(|root| root.record)))
  }

  /// The objects of the root spawn at `path` standing on the level `directory` is named after, by their game vertices.
  fn read_root_spawn<T: ByteOrder>(path: &Path, directory: &Path) -> XrfResult<LevelRootSpawnSource> {
    let bytes: Vec<u8> = std::fs::read(path)
      .map_err(|error| XrfError::new_read_error(format!("{} was not read: {error}", format_path(path))))?;
    let sha256: String = Sidecar::get_sha256_hex(&bytes);
    let level: &str = directory
      .file_name()
      .and_then(|name| name.to_str())
      .ok_or_else(|| XrfError::new_invalid_error(format!("{} names no level", format_path(directory))))?;
    let chunks = ChunkReader::from_vec(bytes)?.read_children()?;
    let objects: SpawnLevelObjects = SpawnFile::read_level_objects_from_chunks::<T, _>(&chunks, level)?;

    Ok(LevelRootSpawnSource {
      record: LevelSceneRootSpawn {
        file: format_path(path).to_string(),
        sha256,
        objects: objects.objects.len(),
        unread: objects.skipped.len(),
        disagreements: Vec::new(),
        update_packets: Vec::new(),
      },
      objects: objects.objects,
    })
  }

  /// The parts the compiled bundle and the level compiler's light list give: glows, lights, and the light qualities of
  /// the level options.
  fn decompile_bundle<T: ByteOrder>(sources: &mut LevelSceneSources, draft: &mut LevelSceneDraft) -> XrfResult {
    let Some(level) = sources.take(LevelFile::FILE_NAME) else {
      return Ok(());
    };
    let level: LevelFile = LevelFile::read_from_bytes::<T>(level)?;
    let lights: Option<LevelLightsFile> = sources
      .take(LevelLightsFile::FILE_NAME)
      .map(LevelLightsFile::read_from_bytes::<T>)
      .transpose()?;

    if let Some(lights) = &lights {
      let options: &mut EditableLevelOptions = &mut draft.level_options;

      options.light_hemi_quality = LevelLightConversion::get_hemi_quality(lights).unwrap_or(options.light_hemi_quality);
      options.light_sun_quality = LevelLightConversion::get_sun_quality(lights).unwrap_or(options.light_sun_quality);
    }

    let glows = LevelGlowConversion::decompile(&level, &mut draft.context)?;

    draft.infer_if(
      !glows.objects.is_empty(),
      &[LevelSceneInference::GlowTransformsAndFlags],
    );
    draft.add(glows.to_scene_tool()?, &[LevelFile::FILE_NAME], sources);

    let lights = LevelLightConversion::decompile(lights.as_ref(), &level, &mut draft.context);

    draft.infer_if(
      !lights.objects.is_empty(),
      &[
        LevelSceneInference::LightBrightness,
        LevelSceneInference::LightFuzzyPoints,
        LevelSceneInference::LightControls,
      ],
    );
    draft.add(
      lights.to_scene_tool()?,
      &[LevelFile::FILE_NAME, LevelLightsFile::FILE_NAME],
      sources,
    );

    Ok(())
  }

  /// The parts of the files the build writes each from one part: particles, sounds, sound environments, fog volumes
  /// and wall marks.
  fn decompile_companions<T: ByteOrder>(sources: &mut LevelSceneSources, draft: &mut LevelSceneDraft) -> XrfResult {
    if let Some(bytes) = sources.take(LevelPsStaticFile::FILE_NAME) {
      let part =
        LevelParticleConversion::decompile(&LevelPsStaticFile::read_from_bytes::<T>(bytes)?, &mut draft.context)?;

      draft.add(part.to_scene_tool()?, &[LevelPsStaticFile::FILE_NAME], sources);
    }

    if let Some(bytes) = sources.take(LevelSndStaticFile::FILE_NAME) {
      let part =
        LevelSoundSourceConversion::decompile(&LevelSndStaticFile::read_from_bytes::<T>(bytes)?, &mut draft.context)?;

      draft.infer_if(!part.objects.is_empty(), &[LevelSceneInference::SoundSourceDistances]);
      draft.add(part.to_scene_tool()?, &[LevelSndStaticFile::FILE_NAME], sources);
    }

    if let Some(bytes) = sources.take(LevelSndEnvFile::FILE_NAME) {
      let part =
        LevelSoundEnvironmentConversion::decompile(&LevelSndEnvFile::read_from_bytes::<T>(bytes)?, &mut draft.context)?;

      draft.add(part.to_scene_tool()?, &[LevelSndEnvFile::FILE_NAME], sources);
    }

    if let Some(bytes) = sources.take(LevelFogVolFile::FILE_NAME) {
      let part =
        LevelFogVolumeConversion::decompile(&LevelFogVolFile::read_from_bytes::<T>(bytes)?, &mut draft.context)?;

      draft.add(part.to_scene_tool()?, &[LevelFogVolFile::FILE_NAME], sources);
    }

    if let Some(bytes) = sources.take(LevelWallmarksFile::FILE_NAME) {
      let tool = LevelWallmarkConversion::decompile(&LevelWallmarksFile::read_from_bytes::<T>(bytes)?)?;

      let scene_tool: EditableSceneTool = tool.to_scene_tool::<T>(draft.context.guid)?;

      draft.infer_if(!tool.slots.is_empty(), &[LevelSceneInference::WallmarkBrush]);
      draft.add(scene_tool, &[LevelWallmarksFile::FILE_NAME], sources);
    }

    Ok(())
  }

  /// The parts `level.game`, `level.env_mod` and the spawn lists give, ways and spawn points, and the game modes the
  /// respawn points serve. Entities are decompiled first, so that no name generated for another spawn point takes
  /// theirs; from the root spawn where it holds the level, its disagreements and update packets recorded with it.
  fn decompile_game<T: ByteOrder>(
    sources: &mut LevelSceneSources,
    root: Option<&mut LevelRootSpawnSource>,
    draft: &mut LevelSceneDraft,
  ) -> XrfResult<u16> {
    let game: Option<LevelGameFile> = sources
      .take(LevelGameFile::FILE_NAME)
      .map(LevelGameFile::read_from_bytes::<T>)
      .transpose()?;
    let environment: Option<LevelEnvModFile> = sources
      .take(LevelEnvModFile::FILE_NAME)
      .map(LevelEnvModFile::read_from_bytes::<T>)
      .transpose()?;
    let spawn_list = |sources: &mut LevelSceneSources, name: &str| {
      sources.take(name).map(LevelSpawnFile::read_from_bytes::<T>).transpose()
    };
    let spawn: Option<LevelSpawnFile> = spawn_list(sources, LevelEntityConversion::SPAWN_FILE_NAME)?;
    let respawn: Option<LevelSpawnFile> = spawn_list(sources, LevelEntityConversion::RESPAWN_FILE_NAME)?;
    let entities: LevelEntityDecompile = LevelEntityConversion::decompile::<T>(
      &LevelEntitySources {
        spawn: spawn.as_ref(),
        respawn: respawn.as_ref(),
        root: root.as_deref().map(|root| root.objects.as_slice()).unwrap_or_default(),
      },
      &mut draft.context,
    )?;

    draft.infer_if(
      entities.entities.iter().any(|entity| entity.attached.is_some()),
      &[LevelSceneInference::EntityShapes],
    );
    draft.refused.extend(entities.refused);

    if let Some(root) = root {
      root.record.disagreements = entities.disagreements;
      root.record.update_packets = entities.update_packets;
    }

    if let Some(game) = &game {
      let ways = LevelWayConversion::decompile(&game.ways, &mut draft.context)?;

      draft.add(ways.to_scene_tool()?, &[LevelGameFile::FILE_NAME], sources);
    }

    let respawn_points: &[LevelGameRPoint] = game.as_ref().map(|game| game.rpoints.as_slice()).unwrap_or_default();

    if game.is_some() || environment.is_some() || !entities.entities.is_empty() {
      let environment: LevelEnvModFile = environment.unwrap_or(LevelEnvModFile {
        version: EditableSpawnPointObject::VERSION,
        modifiers: Vec::new(),
      });
      let mut spawns = LevelSpawnPointConversion::decompile(respawn_points, &environment, &mut draft.context)?;

      spawns.objects.extend(entities.entities);
      draft.add(
        spawns.to_scene_tool()?,
        &[
          LevelGameFile::FILE_NAME,
          LevelEnvModFile::FILE_NAME,
          LevelEntityConversion::SPAWN_FILE_NAME,
          LevelEntityConversion::RESPAWN_FILE_NAME,
        ],
        sources,
      );
    }

    Ok(Self::get_game_types(respawn_points))
  }

  /// The scene's GUID: the first 16 bytes of the SHA-256 of each source's name, a zero, its length and its bytes, in
  /// [`Self::SOURCE_FILES`]' order, then the hash of the root spawn the entities came from, where they did.
  fn get_guid(sources: &BTreeMap<&'static str, Vec<u8>>, root_sha256: Option<&str>) -> EditableLevelGuid {
    let mut hash = Sha256::new();

    for name in Self::SOURCE_FILES {
      if let Some(bytes) = sources.get(name) {
        hash.update(name.as_bytes());
        hash.update([0]);
        hash.update((bytes.len() as u64).to_le_bytes());
        hash.update(bytes);
      }
    }

    if let Some(root_sha256) = root_sha256 {
      hash.update(root_sha256.as_bytes());
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

/// The source files a level's directory holds: their bytes until a decompile takes them, and what the sidecar records
/// of each, hashed as they were read.
struct LevelSceneSources {
  files: BTreeMap<&'static str, Vec<u8>>,
  recorded: Vec<LevelSceneSource>,
}

impl LevelSceneSources {
  /// The files of [`LevelSceneDecompiler::SOURCE_FILES`] the directory holds.
  fn read(directory: &Path) -> XrfResult<Self> {
    let mut files: BTreeMap<&'static str, Vec<u8>> = BTreeMap::new();

    for name in LevelSceneDecompiler::SOURCE_FILES {
      let path: PathBuf = directory.join(name);

      if path.is_file() {
        files.insert(
          name,
          std::fs::read(&path)
            .map_err(|error| XrfError::new_read_error(format!("{} was not read: {error}", format_path(&path))))?,
        );
      }
    }

    if files.is_empty() {
      return Err(XrfError::new_not_found_error(format!(
        "{} holds none of the level files a scene decompiles from",
        format_path(directory)
      )));
    }

    let recorded: Vec<LevelSceneSource> = files
      .iter()
      .map(|(name, bytes)| LevelSceneSource {
        file: (*name).to_owned(),
        sha256: Sidecar::get_sha256_hex(bytes),
      })
      .collect();

    Ok(Self { files, recorded })
  }

  /// The bytes of `name`, handed over once, or none where the directory holds no such file.
  fn take(&mut self, name: &str) -> Option<Vec<u8>> {
    self.files.remove(name)
  }

  /// Whether the directory held `name`, taken or not.
  fn holds(&self, name: &str) -> bool {
    self.recorded.iter().any(|source| source.file == name)
  }
}

/// A scene being decompiled: the context its parts share, its level options so far, each tool with the source files it
/// came from, and what was inferred.
struct LevelSceneDraft {
  context: LevelDecompileContext,
  level_options: EditableLevelOptions,
  tools: Vec<(EditableSceneTool, Vec<&'static str>)>,
  inferred: Vec<LevelSceneInference>,
  refused: Vec<LevelSceneRefusal>,
}

impl LevelSceneDraft {
  /// A scene of `guid` named `name`, holding no tool yet, with the inferences every decompiled scene makes.
  fn new(guid: EditableLevelGuid, name: &str) -> Self {
    Self {
      context: LevelDecompileContext::new(guid),
      level_options: EditableLevelOptions::new(name, EditableLevelOptions::SINGLE_PLAYER),
      tools: Vec::new(),
      inferred: vec![
        LevelSceneInference::ObjectNames,
        LevelSceneInference::LevelOptions,
        LevelSceneInference::GameTypes,
        LevelSceneInference::BuildParameters,
        LevelSceneInference::Camera,
      ],
      refused: Vec::new(),
    }
  }

  /// Records `inferences` where a part holds what they were made for.
  fn infer_if(&mut self, condition: bool, inferences: &[LevelSceneInference]) {
    if condition {
      self.inferred.extend_from_slice(inferences);
    }
  }

  /// Adds a tool decompiled from those of `files` the directory held.
  fn add(&mut self, tool: EditableSceneTool, files: &[&'static str], sources: &LevelSceneSources) {
    self
      .tools
      .push((tool, files.iter().copied().filter(|name| sources.holds(name)).collect()));
  }

  /// The scene and its provenance: its tools sorted by name, and a main file serving `game_types`, whose game types
  /// come from the respawn points and light qualities from the generated sky and sun.
  fn finish(
    mut self,
    options: &LevelDecompileOptions,
    game_types: u16,
    sources: &LevelSceneSources,
    root_spawn: Option<LevelSceneRootSpawn>,
  ) -> LevelSceneDecompile {
    self
      .tools
      .sort_by(|(first, _), (second, _)| first.name.cmp(&second.name));

    let files: Vec<LevelSceneFile> = std::iter::once(LevelSceneFile {
      written: format!("{}.{}", options.name, EditableLevelFile::EXTENSION),
      sources: [LevelLightsFile::FILE_NAME, LevelGameFile::FILE_NAME]
        .into_iter()
        .filter(|name| sources.holds(name))
        .map(str::to_owned)
        .collect(),
    })
    .chain(self.tools.iter().map(|(tool, files)| LevelSceneFile {
      written: format!("{}/{}", options.name, tool.get_file_name(0)),
      sources: files.iter().map(|name| (*name).to_owned()).collect(),
    }))
    .collect();
    let level: EditableLevelFile = EditableLevelFile::new(
      &self.context.guid,
      &EditableLevelOptions {
        game_type: game_types,
        ..self.level_options
      },
      &EditableBuildParams {
        weld_distance: 0.0,
        ..EditableBuildParams::DEFAULT
      },
      &EditableLevelCamera::default(),
      LevelSceneDecompiler::OWNER,
      0,
    );

    LevelSceneDecompile {
      scene: EditableLevelScene {
        level,
        tools: self.tools.into_iter().map(|(tool, _)| tool).collect(),
      },
      provenance: LevelSceneProvenance {
        sources: sources.recorded.clone(),
        files,
        inferred: self.inferred,
        refused: self.refused,
        root_spawn,
        ..LevelSceneProvenance::new(options.name.clone(), options.tool.clone(), options.tool_version.clone())
      },
    }
  }
}
