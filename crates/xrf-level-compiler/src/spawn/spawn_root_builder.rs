use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use sha2::{Digest, Sha256};
use uuid::Uuid;
use xrf_chunk::XRayByteOrder;
use xrf_error::{XrfError, XrfResult};
use xrf_spawn::{
  AlifeBuiltFields, AlifeObject, ArtefactSpawnPoint, Patrol, SpawnALifeSpawnsChunk, SpawnArtefactSpawnsChunk,
  SpawnFile, SpawnGraphsChunk, SpawnHeaderChunk, SpawnPatrolsChunk,
};

use crate::spawn::spawn_actor_start::SpawnActorStart;
use crate::spawn::spawn_game_graph_merge::SpawnGameGraphMerge;
use crate::spawn::spawn_game_graph_merger::SpawnGameGraphMerger;
use crate::spawn::spawn_level_changer_links::SpawnLevelChangerLinks;
use crate::spawn::spawn_level_entities::{SpawnLevelEntities, SpawnLevelVertices};
use crate::spawn::spawn_merge_level::SpawnMergeLevel;
use crate::spawn::spawn_patrols::SpawnPatrols;
use crate::spawn::spawn_random::SpawnRandom;
use crate::spawn::spawn_root_build::SpawnRootBuild;
use crate::spawn::spawn_root_level::SpawnRootLevel;
use crate::spawn::spawn_root_options::SpawnRootOptions;
use crate::spawn::spawn_root_report::SpawnRootReport;
use crate::spawn::spawn_separator_check::SpawnSeparatorCheck;
use crate::spawn::spawn_separator_cut::SpawnSeparatorCut;

/// The root spawn builder, xrAI's `CGameSpawnConstructor` (`game_spawn_constructor.cpp:24`): the levels' game graphs
/// merged, then each level's entities placed and its patrols read, the levels in id order and the random stream one
/// from the merge's level points to the last zone's artefact points; then the level changers linked, the zones'
/// artefact points laid one level after another, and the actor found and, with a start level, moved.
///
/// The output is deterministic, where xrAI's is not: patrols go sorted by name, a patrol point with no node under it
/// takes a game vertex of its level, and the spawn's GUID, like the graph's, is derived from what the file holds.
///
/// What xrAI asserts is refused, naming the level and the entity; what it lets through is reported: the connections to
/// levels outside the build, and the restrictors whose border cuts an AI map apart ([`SpawnSeparatorCheck`]), refused
/// only with [`SpawnRootOptions::is_separator_strict`].
pub struct SpawnRootBuilder;

impl SpawnRootBuilder {
  /// `XRAI_CURRENT_VERSION`, the spawn's version.
  pub const VERSION: u32 = 10;

  /// Builds the root spawn of `levels` as `options` asks.
  ///
  /// # Errors
  ///
  /// Returns an error when the levels do not merge ([`SpawnGameGraphMerger::merge`]), a level has no graph point or
  /// no entity, an entity does not read as its class, two entities share a story id, a level changer cannot be
  /// linked, two patrols share a name, the levels hold no actor or more than one, the start level is no level of the
  /// build, or, strict, a restrictor cuts an AI map apart.
  pub fn build(levels: &[SpawnRootLevel<'_>], options: &SpawnRootOptions) -> XrfResult<SpawnRootBuild> {
    let mut ordered: Vec<&SpawnRootLevel<'_>> = levels.iter().collect();

    ordered.sort_by_key(|level| level.merge.id);

    let merges: Vec<SpawnMergeLevel<'_>> = ordered.iter().map(|level| level.merge.clone()).collect();
    let mut random: SpawnRandom = SpawnRandom::new();
    let merge: SpawnGameGraphMerge = SpawnGameGraphMerger::merge(&merges, &mut random)?;
    let mut entities: Vec<SpawnLevelEntities> = Vec::new();
    let mut patrols: Vec<Patrol> = Vec::new();
    let mut cuts: Vec<SpawnSeparatorCut> = Vec::new();
    let mut separators_took: Duration = Duration::ZERO;
    let mut first: u16 = 0;

    for (level, merge_level) in ordered.iter().zip(&merges) {
      let Some(first_vertex) = merge.graph.vertices.get(usize::from(first)) else {
        return Err(XrfError::new_invalid_error(format!(
          "Level {} has no graph point, so no game vertex its entities can stand on",
          merge_level.name
        )));
      };
      let vertices: SpawnLevelVertices = SpawnLevelVertices {
        first,
        first_node: first_vertex.level_vertex_id,
      };

      let level_entities: SpawnLevelEntities = SpawnLevelEntities::build(level, &vertices, &mut random)?;
      let started: Instant = Instant::now();

      cuts.extend(SpawnSeparatorCheck::new(merge_level.map).find_cuts(&merge_level.name, &level_entities.objects));
      separators_took += started.elapsed();
      entities.push(level_entities);

      if let Some(game) = level.game {
        patrols.extend(SpawnPatrols::build(merge_level, game, &vertices));
      }

      first += merge_level.points.len() as u16;
    }

    Self::assert_story_ids(&merges, &entities)?;
    Self::assert_separators(&cuts, options)?;
    SpawnLevelChangerLinks::fill(&merges, &mut entities, &merge.graph)?;
    Self::sort_patrols(&mut patrols)?;

    let mut spawn: SpawnFile = SpawnFile {
      header: SpawnHeaderChunk {
        version: Self::VERSION,
        guid: Uuid::nil(),
        graph_guid: merge.graph.header.guid,
        objects_count: 0,
        levels_count: merges.len() as u32,
      },
      alife_spawn: SpawnALifeSpawnsChunk { objects: Vec::new() },
      artefact_spawn: SpawnArtefactSpawnsChunk { nodes: Vec::new() },
      patrols: SpawnPatrolsChunk { patrols },
      graphs: merge.graph,
    };

    Self::lay_out(entities, &mut spawn)?;
    Self::place_actor(&mut spawn, options.start.as_deref())?;
    spawn.header.guid = Self::derive_guid(&spawn)?;

    Ok(SpawnRootBuild {
      spawn,
      report: SpawnRootReport::build(&merge.skipped, &cuts, separators_took),
    })
  }

  /// `add_story_object`: a story id names one entity of the whole build.
  fn assert_story_ids(levels: &[SpawnMergeLevel<'_>], entities: &[SpawnLevelEntities]) -> XrfResult {
    let mut owners: BTreeMap<u32, String> = BTreeMap::new();

    for (level, level_entities) in levels.iter().zip(entities) {
      for object in &level_entities.objects {
        let Some(story_id) = object.inherited.get_abstract().and_then(|base| base.get_story_id()) else {
          continue;
        };

        let owner: String = format!("{} on level {}", object.header.name, level.name);

        if let Some(other) = owners.insert(story_id, owner.clone()) {
          return Err(XrfError::new_invalid_error(format!(
            "Story id {story_id} names both {other} and {owner}"
          )));
        }
      }
    }

    Ok(())
  }

  /// Refuses a build whose restrictors cut an AI map apart, where the options ask for it.
  fn assert_separators(cuts: &[SpawnSeparatorCut], options: &SpawnRootOptions) -> XrfResult {
    match cuts.first().filter(|_| options.is_separator_strict) {
      Some(cut) => Err(XrfError::new_invalid_error(format!(
        "Restrictors cut an AI map apart, {} in all; on level {}: {}",
        cuts.len(),
        cut.level,
        SpawnRootReport::describe(cut)
      ))),
      None => Ok(()),
    }
  }

  /// The levels' objects one level after another, each numbered by its place (`add_object`), and their zones' artefact
  /// points likewise, each zone told where its own begin (`update_artefact_spawn_positions`).
  fn lay_out(entities: Vec<SpawnLevelEntities>, spawn: &mut SpawnFile) -> XrfResult {
    let objects: &mut Vec<AlifeObject> = &mut spawn.alife_spawn.objects;
    let artefacts: &mut Vec<ArtefactSpawnPoint> = &mut spawn.artefact_spawn.nodes;

    for level in entities {
      for (mut object, points) in level.objects.into_iter().zip(level.artefacts) {
        let mut fields: AlifeBuiltFields = object.get_built_fields();

        fields.spawn_id = u16::try_from(objects.len())
          .map_err(|_| XrfError::new_invalid_error("The levels hold more objects than a spawn numbers"))?;

        if let Some(points) = points {
          fields.artefact_position_offset = Some(Self::narrow(artefacts.len(), "artefact points")?);
          artefacts.extend(points);
        }

        object.set_built_fields(&fields);
        objects.push(object);
      }
    }

    spawn.header.objects_count = Self::narrow(objects.len(), "objects")?;

    Ok(())
  }

  fn narrow(count: usize, what: &str) -> XrfResult<u32> {
    u32::try_from(count)
      .map_err(|_| XrfError::new_invalid_error(format!("The levels hold more {what} than a spawn counts")))
  }

  /// `process_actor`: the one actor of the build, moved to the start level's first vertex a search from its own
  /// reaches where `start` names another level than its own.
  fn place_actor(spawn: &mut SpawnFile, start: Option<&str>) -> XrfResult {
    let graph: &SpawnGraphsChunk = &spawn.graphs;
    let mut actors = spawn
      .alife_spawn
      .objects
      .iter_mut()
      .filter(|object| SpawnLevelEntities::is_actor(object));
    let actor: &mut AlifeObject = actors
      .next()
      .ok_or_else(|| XrfError::new_invalid_error("The levels hold no actor"))?;

    if let Some(other) = actors.next() {
      return Err(XrfError::new_invalid_error(format!(
        "The levels hold more than one actor: {} and {}",
        actor.header.name, other.header.name
      )));
    }

    let Some(start) = start else {
      return Ok(());
    };
    let Some(level) = graph.levels.iter().find(|level| level.name == start) else {
      return Err(XrfError::new_invalid_error(format!(
        "The start level {start} is not a level of the build"
      )));
    };
    let mut fields = actor.get_built_fields();
    let Some(from) = fields.game_vertex_id else {
      return Ok(());
    };

    if graph.vertices[usize::from(from)].level_id == level.id {
      return Ok(());
    }

    let Some(to) = SpawnActorStart::find_vertex(graph, from, level.id) else {
      return Err(XrfError::new_invalid_error(format!(
        "The start level {start} has no game vertex"
      )));
    };
    let vertex = &graph.vertices[usize::from(to)];

    fields.game_vertex_id = Some(to);
    fields.level_vertex_id = Some(vertex.level_vertex_id);
    actor.set_built_fields(&fields);
    actor.move_to(vertex.level_point, actor.header.direction);

    Ok(())
  }

  /// Patrols by name, where xrAI writes them in the order its name strings were allocated; the engine files them by
  /// the string's address on load, so the order changes nothing it does.
  fn sort_patrols(patrols: &mut [Patrol]) -> XrfResult {
    patrols.sort_by(|a, b| a.name.as_bytes().cmp(b.name.as_bytes()));

    match patrols.windows(2).find(|pair| pair[0].name == pair[1].name) {
      Some(pair) => Err(XrfError::new_invalid_error(format!(
        "Two patrols are named {}",
        pair[0].name
      ))),
      None => Ok(()),
    }
  }

  /// The spawn's GUID: the first 16 bytes of the SHA-256 of the file written with a nil one, so the same levels give
  /// the same spawn and a changed spawn another GUID.
  fn derive_guid(spawn: &SpawnFile) -> XrfResult<Uuid> {
    let mut bytes: Vec<u8> = Vec::new();

    spawn.write_to::<XRayByteOrder>(&mut bytes)?;

    let digest = Sha256::digest(&bytes);
    let mut guid: [u8; 16] = [0; 16];

    guid.copy_from_slice(&digest[..16]);

    Ok(Uuid::from_bytes(guid))
  }
}
