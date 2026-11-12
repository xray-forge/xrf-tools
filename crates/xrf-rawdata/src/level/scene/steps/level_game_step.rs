use byteorder::ByteOrder;
use xrf_editable_level::{EditableLevelOptions, EditableSpawnPointObject};
use xrf_error::XrfResult;
use xrf_level::{LevelEnvModFile, LevelGameFile, LevelGameRPoint};

use crate::level::level_entity_conversion::{LevelEntityConversion, LevelEntityDecompile, LevelEntitySources};
use crate::level::level_spawn_point_conversion::LevelSpawnPointConversion;
use crate::level::level_way_conversion::LevelWayConversion;
use crate::level::scene::level_decompile_options::LevelDecompileOptions;
use crate::level::scene::level_scene_draft::LevelSceneDraft;
use crate::level::scene::level_scene_sources::LevelSceneSources;
use crate::level::scene::steps::level_scene_step::LevelSceneStep;
use crate::provenance::level_scene_provenance::LevelSceneInference;

/// What `level.game`, `level.env_mod` and the draft's spawn lists give: ways, spawn points (respawn points, environment
/// modifiers and entities, from the root spawn where the draft holds one), and the game modes the level serves where
/// no `level.ltx` named them.
pub(crate) struct LevelGameStep;

impl LevelSceneStep for LevelGameStep {
  const FILES: &'static [&'static str] = &[
    LevelGameFile::FILE_NAME,
    LevelEnvModFile::FILE_NAME,
    LevelEntityConversion::SPAWN_FILE_NAME,
    LevelEntityConversion::RESPAWN_FILE_NAME,
  ];

  fn decompile<T: ByteOrder>(
    sources: &mut LevelSceneSources,
    _options: &LevelDecompileOptions,
    draft: &mut LevelSceneDraft,
  ) -> XrfResult {
    let game: Option<LevelGameFile> =
      sources.take_read(LevelGameFile::FILE_NAME, LevelGameFile::read_from_bytes::<T>)?;
    let environment: Option<LevelEnvModFile> =
      sources.take_read(LevelEnvModFile::FILE_NAME, LevelEnvModFile::read_from_bytes::<T>)?;
    let entities: LevelEntityDecompile = LevelEntityConversion::decompile::<T>(
      &LevelEntitySources {
        spawn: draft.spawn.as_ref(),
        respawn: draft.respawn.as_ref(),
        root: draft
          .root_spawn
          .as_ref()
          .map(|root| root.objects.as_slice())
          .unwrap_or_default(),
        folded: &draft.folded_entities,
      },
      &mut draft.context,
    )?;

    draft.infer_if(
      entities.entities.iter().any(|entity| entity.attached.is_some()),
      &[LevelSceneInference::EntityShapes],
    );
    draft.refused.extend(entities.refused);

    if let Some(root) = &mut draft.root_spawn {
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
      draft.add(spawns.to_scene_tool()?, Self::FILES, sources);
    }

    draft.level_options.game_type = match &draft.map {
      Some(map) => map.game_types,
      None => Self::get_game_types(respawn_points),
    };
    draft.infer_if(draft.map.is_none(), &[LevelSceneInference::GameTypes]);
    draft.add_level_source(LevelGameFile::FILE_NAME, sources);

    Ok(())
  }
}

impl LevelGameStep {
  /// The game modes the level serves: those its respawn points answer to, or single player for a level of none.
  fn get_game_types(respawn_points: &[LevelGameRPoint]) -> u16 {
    respawn_points
      .iter()
      .map(|point| point.game_types)
      .reduce(|types, point| types | point)
      .unwrap_or(EditableLevelOptions::SINGLE_PLAYER)
  }
}
