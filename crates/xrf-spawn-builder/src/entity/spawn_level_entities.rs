use xrf_chunk::XRayByteOrder;
use xrf_error::{XrfError, XrfResult};
use xrf_math::Vector3d;
use xrf_spawn::{
  AlifeBuiltFields, AlifeObject, AlifeObjectInherited, ArtefactSpawnPoint, LevelSpawnFile, SpawnSectionClasses,
};

use crate::entity::spawn_ai_locations::SpawnAiLocations;
use crate::entity::spawn_artefact_buffer::SpawnArtefactBuffer;
use crate::entity::spawn_level_vertices::SpawnLevelVertices;
use crate::map::spawn_path_search::SpawnPathSearch;
use crate::merge::spawn_merge_level::SpawnMergeLevel;
use crate::merge::spawn_random::SpawnRandom;
use crate::root::spawn_root_level::SpawnRootLevel;

/// A level's entities as `CLevelSpawnConstructor::Execute` leaves them: placed on the AI map, anomalous zones given
/// artefact points, level changers their own node.
///
/// Graph points are kept apart for the level changers that lead to this level.
pub(crate) struct SpawnLevelEntities {
  pub(crate) objects: Vec<AlifeObject>,
  /// Each object's artefact points, parallel to `objects`; only anomalous zones have some.
  pub(crate) artefacts: Vec<Option<Vec<ArtefactSpawnPoint>>>,
  pub(crate) graph_points: Vec<AlifeObject>,
}

impl SpawnLevelEntities {
  /// `y_shift_correction`, how far a position is raised before its node is looked up.
  pub(crate) const Y_SHIFT: f32 = 0.15;
  /// `eGameIDSingle`, the game type bit of objects a single player game loads.
  const SINGLE_PLAYER: u16 = 1;

  /// Builds the level's entities, the zones' shuffles drawn from `random`.
  ///
  /// # Errors
  ///
  /// Returns an error when a packet does not read as its class, or the level holds no entity (`load_objects`).
  pub(crate) fn build(
    root: &SpawnRootLevel<'_>,
    vertices: &SpawnLevelVertices,
    random: &mut SpawnRandom,
  ) -> XrfResult<Self> {
    let level: &SpawnMergeLevel<'_> = &root.merge;
    let mut entities: Self = Self::load(root.spawn, root.classes)?;

    if entities.objects.is_empty() {
      return Err(XrfError::new_invalid_error(format!(
        "Level {} holds no entity a single player game loads",
        level.name
      )));
    }
    let mut search: SpawnPathSearch = SpawnPathSearch::new(level.map);
    let mut buffer: SpawnArtefactBuffer = SpawnArtefactBuffer::default();

    // xrAI runs each step over all objects in turn; one loop matches it because only the draws share state, and they
    // keep their object order.
    for object in &mut entities.objects {
      Self::place(level, vertices, object);
      entities.artefacts.push(Self::draw_artefacts(
        level,
        vertices,
        object,
        &mut search,
        &mut buffer,
        random,
      ));

      if matches!(object.inherited, AlifeObjectInherited::SeLevelChanger(_)) {
        let node: u32 = level.map.place(&Self::raise(&object.header.position));

        Self::set_node(level, vertices, object, node);
      }
    }

    Ok(entities)
  }

  /// `load_objects`: every single player object, with graph points set apart.
  fn load(list: &LevelSpawnFile, classes: &SpawnSectionClasses) -> XrfResult<Self> {
    let mut entities: Self = Self {
      objects: Vec::new(),
      artefacts: Vec::new(),
      graph_points: Vec::new(),
    };

    for packet in &list.objects {
      let object: AlifeObject = packet.read_alife_object_by::<XRayByteOrder>(classes)?;

      if matches!(object.inherited, AlifeObjectInherited::CseAlifeGraphPoint(_)) {
        entities.graph_points.push(object);
        continue;
      }

      if object.header.game_type & Self::SINGLE_PLAYER == 0 {
        continue;
      }

      entities.objects.push(object);
    }

    Ok(entities)
  }

  /// `correct_objects`: an object using AI locations takes the node under its raised position, moving onto it when
  /// outside its cell; any other takes the level's first vertex.
  fn place(level: &SpawnMergeLevel<'_>, vertices: &SpawnLevelVertices, object: &mut AlifeObject) {
    if !SpawnAiLocations::is_used(object) {
      let mut fields: AlifeBuiltFields = object.get_built_fields();

      fields.game_vertex_id = fields.game_vertex_id.map(|_| vertices.first);
      fields.distance = fields.distance.map(|_| 0.0);
      fields.level_vertex_id = fields.level_vertex_id.map(|_| vertices.first_node);
      object.set_built_fields(&fields);
      return;
    }

    let raised: Vector3d<f32> = Self::raise(&object.header.position);
    let node: u32 = level.map.place(&raised);

    if !level.map.is_inside(node, &raised) {
      object.move_to(level.map.get_node_position(node), object.header.direction);
    }

    Self::set_node(level, vertices, object, node);
  }

  /// `generate_artefact_spawn_positions` for one object, `None` unless an anomalous zone.
  ///
  /// The zone's node is looked up again unless its unraised position lies inside it. The nodes within its offline
  /// radius are shuffled and the first `artefact_spawn_count` become its points.
  fn draw_artefacts(
    level: &SpawnMergeLevel<'_>,
    vertices: &SpawnLevelVertices,
    object: &mut AlifeObject,
    search: &mut SpawnPathSearch,
    buffer: &mut SpawnArtefactBuffer,
    random: &mut SpawnRandom,
  ) -> Option<Vec<ArtefactSpawnPoint>> {
    let (radius, count): (f32, u16) = match &object.inherited {
      AlifeObjectInherited::CseAlifeAnomalousZone(zone) => {
        (zone.base.offline_interactive_radius, zone.base.artefact_spawn_count)
      }
      AlifeObjectInherited::CseAlifeZoneVisual(zone) => {
        (zone.base.offline_interactive_radius, zone.base.artefact_spawn_count)
      }
      _ => return None,
    };
    let position: Vector3d<f32> = object.header.position;
    let current: u32 = object.get_built_fields().level_vertex_id.unwrap_or(u32::MAX);
    let node: u32 = if (current as usize) < level.map.get_node_count() && level.map.is_inside(current, &position) {
      current
    } else {
      level.map.place(&position)
    };

    Self::set_node(level, vertices, object, node);

    let flooded: Vec<u32> = search.flood(level.map, node, radius);
    let kept: usize = buffer.fill(&flooded, |candidate| {
      Self::distance_squared(&level.map.get_node_position(candidate), &position) <= radius * radius
    });

    random.shuffle(buffer.get_mut(kept));

    Some(
      buffer
        .read(usize::from(count))
        .iter()
        .map(|node| ArtefactSpawnPoint {
          position: level.map.get_node_position(*node),
          level_vertex_id: *node,
          distance: level.cells.get(*node as usize).map_or(0.0, |cell| cell.distance),
        })
        .collect(),
    )
  }

  /// Sets an object's node and the game vertex and distance its cell gives.
  fn set_node(level: &SpawnMergeLevel<'_>, vertices: &SpawnLevelVertices, object: &mut AlifeObject, node: u32) {
    let cell = level.cells[node as usize];
    let mut fields: AlifeBuiltFields = object.get_built_fields();

    fields.game_vertex_id = fields.game_vertex_id.map(|_| vertices.first + cell.game_vertex_id);
    fields.distance = fields.distance.map(|_| cell.distance);
    fields.level_vertex_id = fields.level_vertex_id.map(|_| node);
    object.set_built_fields(&fields);
  }

  /// Whether the object is the actor, `CSE_ALifeCreatureActor`; a build holds one.
  pub(crate) fn is_actor(object: &AlifeObject) -> bool {
    matches!(
      object.inherited,
      AlifeObjectInherited::SeActor(_) | AlifeObjectInherited::ClearSkySeActor(_)
    )
  }

  /// A position raised by [`Self::Y_SHIFT`] for a node lookup.
  pub(crate) fn raise(position: &Vector3d<f32>) -> Vector3d<f32> {
    Vector3d::new(position.x, position.y + Self::Y_SHIFT, position.z)
  }

  /// `Fvector::distance_to_sqr`.
  fn distance_squared(a: &Vector3d<f32>, b: &Vector3d<f32>) -> f32 {
    let (dx, dy, dz) = (a.x - b.x, a.y - b.y, a.z - b.z);

    dx * dx + dy * dy + dz * dz
  }
}
