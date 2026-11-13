use xrf_chunk::XRayByteOrder;
use xrf_error::XrfResult;
use xrf_math::Vector3d;
use xrf_spawn::{AlifeBuiltFields, AlifeObject, AlifeObjectInherited, ArtefactSpawnPoint, LevelSpawnFile};

use crate::spawn::spawn_ai_locations::SpawnAiLocations;
use crate::spawn::spawn_artefact_buffer::SpawnArtefactBuffer;
use crate::spawn::spawn_merge_level::SpawnMergeLevel;
use crate::spawn::spawn_path_search::SpawnPathSearch;
use crate::spawn::spawn_random::SpawnRandom;
use crate::spawn::spawn_root_level::SpawnRootLevel;

/// A level's entities as xrAI's `CLevelSpawnConstructor::Execute` (`level_spawn_constructor.cpp:600`) leaves them:
/// loaded from `level.spawn`, placed on the AI map, each anomalous zone given its artefact points, each level changer
/// its own node; its graph points kept apart for the level changers that lead here.
pub(crate) struct SpawnLevelEntities {
  pub(crate) objects: Vec<AlifeObject>,
  /// Each object's artefact points, beside it in `objects`; an anomalous zone's alone has some.
  pub(crate) artefacts: Vec<Option<Vec<ArtefactSpawnPoint>>>,
  pub(crate) graph_points: Vec<AlifeObject>,
}

/// What the per-level pass needs of the merged graph: where the level's vertices start and the first one's node.
pub(crate) struct SpawnLevelVertices {
  pub(crate) first: u16,
  pub(crate) first_node: u32,
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
  /// Returns an error when a packet does not read as its class.
  pub(crate) fn build(
    root: &SpawnRootLevel<'_>,
    vertices: &SpawnLevelVertices,
    random: &mut SpawnRandom,
  ) -> XrfResult<Self> {
    let level: &SpawnMergeLevel<'_> = &root.merge;
    let mut entities: Self = Self::load(root.spawn)?;
    let mut search: SpawnPathSearch = SpawnPathSearch::new(level.map);
    let mut buffer: SpawnArtefactBuffer = SpawnArtefactBuffer::default();

    // xrAI places every object, then draws every zone's points, then looks the level changers up; only the draws
    // share state, the stream and the buffer, and they keep their order object by object.
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

  /// `load_objects`: every packet but graph points, which go apart, and objects a single player game does not load.
  fn load(list: &LevelSpawnFile) -> XrfResult<Self> {
    let mut entities: Self = Self {
      objects: Vec::new(),
      artefacts: Vec::new(),
      graph_points: Vec::new(),
    };

    for packet in &list.objects {
      let object: AlifeObject = packet.read_alife_object::<XRayByteOrder>()?;

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

  /// `correct_objects`: an object using AI locations takes the node under its raised position and moves onto it where
  /// it stands outside its cell; any other takes the level's first vertex.
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

  /// `generate_artefact_spawn_positions` for one object, if an anomalous zone: its node looked up again unraised, from
  /// the one it has; the nodes flooded within its offline radius and lying within it, shuffled, the first of them its
  /// artefact points.
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

  /// Whether the object is the actor, `CSE_ALifeCreatureActor`, of which a build holds one.
  pub(crate) fn is_actor(object: &AlifeObject) -> bool {
    matches!(
      object.inherited,
      AlifeObjectInherited::SeActor(_) | AlifeObjectInherited::ClearSkySeActor(_)
    )
  }

  /// A position raised by [`Self::Y_SHIFT`], where xrAI looks a node up from.
  pub(crate) fn raise(position: &Vector3d<f32>) -> Vector3d<f32> {
    Vector3d::new(position.x, position.y + Self::Y_SHIFT, position.z)
  }

  /// `Fvector::distance_to_sqr`.
  fn distance_squared(a: &Vector3d<f32>, b: &Vector3d<f32>) -> f32 {
    let (dx, dy, dz) = (a.x - b.x, a.y - b.y, a.z - b.z);

    dx * dx + dy * dy + dz * dz
  }
}
