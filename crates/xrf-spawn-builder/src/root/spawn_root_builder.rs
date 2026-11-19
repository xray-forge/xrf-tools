use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use sha2::{Digest, Sha256};
use uuid::Uuid;
use xrf_chunk::XRayByteOrder;
use xrf_error::{XrfError, XrfResult};
use xrf_report::Report;
use xrf_spawn::{
  AlifeBuiltFields, AlifeObject, ArtefactSpawnPoint, Patrol, SpawnALifeSpawnsChunk, SpawnArtefactSpawnsChunk,
  SpawnFile, SpawnGraphsChunk, SpawnHeaderChunk, SpawnPatrolsChunk,
};
use xrf_utils::narrow_format_size;

use crate::entity::spawn_level_changer_links::SpawnLevelChangerLinks;
use crate::entity::spawn_level_entities::SpawnLevelEntities;
use crate::entity::spawn_level_vertices::SpawnLevelVertices;
use crate::entity::spawn_patrols::SpawnPatrols;
use crate::merge::spawn_game_graph_merge::SpawnGameGraphMerge;
use crate::merge::spawn_game_graph_merger::SpawnGameGraphMerger;
use crate::merge::spawn_merge_level::SpawnMergeLevel;
use crate::merge::spawn_random::SpawnRandom;
use crate::root::spawn_actor_start::SpawnActorStart;
use crate::root::spawn_root_build::SpawnRootBuild;
use crate::root::spawn_root_level::SpawnRootLevel;
use crate::root::spawn_root_options::SpawnRootOptions;
use crate::root::spawn_root_report::SpawnRootReport;
use crate::separator::spawn_separator_check::SpawnSeparatorCheck;
use crate::separator::spawn_separator_cut::SpawnSeparatorCut;

/// The root spawn builder, `CGameSpawnConstructor`: merges the levels' game graphs, places each level's entities and
/// patrols in id order, then links level changers, lays out artefact points and places the actor.
///
/// One random stream runs from the merge's level points to the last zone's artefact points. The output is
/// deterministic where xrAI's is not: patrols sorted by name, unplaced patrol points given a vertex of their level,
/// and GUIDs derived from content.
///
/// What xrAI asserts is refused, naming the level and entity. What it lets through is reported: connections to levels
/// outside the build, and restrictors cutting an AI map apart, refused only with
/// [`SpawnRootOptions::is_separator_strict`].
pub(crate) struct SpawnRootBuilder;

impl SpawnRootBuilder {
  /// `XRAI_CURRENT_VERSION`, the spawn's version.
  pub(crate) const VERSION: u32 = 10;

  /// Builds the root spawn of `levels` as `options` asks.
  ///
  /// # Errors
  ///
  /// Returns an error when the levels do not merge ([`SpawnGameGraphMerger::merge`]), a level has no graph point or
  /// entity, an entity does not read as its class, two entities share a story id, a level changer cannot be linked, two
  /// patrols share a name, the levels hold no actor or several, the start level is not built, or, when strict, a
  /// restrictor cuts an AI map apart.
  pub(crate) fn build(levels: &[SpawnRootLevel<'_>], options: &SpawnRootOptions) -> XrfResult<SpawnRootBuild> {
    let mut ordered: Vec<&SpawnRootLevel<'_>> = levels.iter().collect();

    ordered.sort_by_key(|level| level.merge.id);
    Self::assert_graph_points(&ordered)?;

    let merges: Vec<SpawnMergeLevel<'_>> = ordered.iter().map(|level| level.merge.clone()).collect();
    let mut random: SpawnRandom = SpawnRandom::new();
    let merge: SpawnGameGraphMerge = SpawnGameGraphMerger::merge(&merges, &mut random)?;
    let mut entities: Vec<SpawnLevelEntities> = Vec::new();
    let mut patrols: Vec<Patrol> = Vec::new();
    let mut cuts: Vec<SpawnSeparatorCut> = Vec::new();
    let mut separators_took: Duration = Duration::ZERO;
    let mut first: u16 = 0;

    for (level, merge_level) in ordered.iter().zip(&merges) {
      // Every level keeps a graph point, so the merged graph holds a vertex at each level's first.
      let vertices: SpawnLevelVertices = SpawnLevelVertices {
        first,
        first_node: merge.graph.vertices[usize::from(first)].level_vertex_id,
      };

      let level_entities: SpawnLevelEntities = SpawnLevelEntities::build(level, &vertices, &mut random)?;
      let started: Instant = Instant::now();

      cuts.extend(SpawnSeparatorCheck::new(merge_level.map).find_cuts(&merge_level.name, &level_entities.objects));
      separators_took += started.elapsed();
      entities.push(level_entities);

      if let Some(game) = level.game {
        patrols.extend(SpawnPatrols::build(merge_level, game, &vertices));
      }

      first = narrow_format_size(usize::from(first) + merge_level.points.len(), "A game vertex id")?;
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
        levels_count: narrow_format_size(merges.len(), "A spawn's level count")?,
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
      report: Report::new(vec![
        SpawnRootReport::get_graph_points(&ordered),
        SpawnRootReport::get_connections(&merge.skipped),
        SpawnRootReport::get_separators(&cuts, separators_took),
      ]),
    })
  }

  /// Refuses a level left with no graph point for its entities to stand on, naming why each point went, before the
  /// merge fails on its empty cross table.
  fn assert_graph_points(levels: &[&SpawnRootLevel<'_>]) -> XrfResult {
    let Some(level) = levels.iter().find(|level| level.merge.points.is_empty()) else {
      return Ok(());
    };

    let removed: Vec<String> = level
      .removed
      .iter()
      .map(|point| format!("{}, {}", point.name, point.removal))
      .collect();

    Err(XrfError::new_invalid_error(if removed.is_empty() {
      format!(
        "Level {} has no graph point, so no game vertex its entities can stand on",
        level.merge.name
      )
    } else {
      format!(
        "Level {} keeps no graph point, so no game vertex its entities can stand on; left out: {}",
        level.merge.name,
        removed.join("; ")
      )
    }))
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

  /// Lays out the levels' objects in order, numbered by place (`add_object`), and their zones' artefact points, each
  /// zone given its offset (`update_artefact_spawn_positions`).
  fn lay_out(entities: Vec<SpawnLevelEntities>, spawn: &mut SpawnFile) -> XrfResult {
    let objects: &mut Vec<AlifeObject> = &mut spawn.alife_spawn.objects;
    let artefacts: &mut Vec<ArtefactSpawnPoint> = &mut spawn.artefact_spawn.nodes;

    for level in entities {
      for (mut object, points) in level.objects.into_iter().zip(level.artefacts) {
        let mut fields: AlifeBuiltFields = object.get_built_fields();

        fields.spawn_id = u16::try_from(objects.len())
          .map_err(|_| XrfError::new_invalid_error("The levels hold more objects than a spawn numbers"))?;

        if let Some(points) = points {
          fields.artefact_position_offset = Some(narrow_format_size(artefacts.len(), "An artefact point offset")?);
          artefacts.extend(points);
        }

        object.set_built_fields(&fields);
        objects.push(object);
      }
    }

    spawn.header.objects_count = narrow_format_size(objects.len(), "A spawn's object count")?;

    Ok(())
  }

  /// `process_actor`: checks the build holds one actor and, where `start` names another level, moves it there through
  /// [`SpawnActorStart::find_vertex`].
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

  /// Sorts patrols by name, refusing a repeated name.
  ///
  /// xrAI writes them in name allocation order, but the engine keys them by string address on load, so the order is
  /// free.
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

  /// The spawn's GUID: the first 16 bytes of the SHA-256 of the file written with a nil GUID, so the same levels give
  /// the same spawn.
  fn derive_guid(spawn: &SpawnFile) -> XrfResult<Uuid> {
    let mut bytes: Vec<u8> = Vec::new();

    spawn.write_to::<XRayByteOrder>(&mut bytes)?;

    let digest = Sha256::digest(&bytes);
    let mut guid: [u8; 16] = [0; 16];

    guid.copy_from_slice(&digest[..16]);

    Ok(Uuid::from_bytes(guid))
  }
}
