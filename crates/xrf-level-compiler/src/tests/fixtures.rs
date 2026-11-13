use uuid::Uuid;
use xrf_chunk::XRayByteOrder;
use xrf_error::XrfResult;
use xrf_level::{LevelAiHeader, LevelAiNode, LevelAiNodes, LevelAiVersion};
use xrf_math::{Aabb, Vector3d};
use xrf_spawn::{
  AlifeActor, AlifeAnomalousZone, AlifeGraphPoint, AlifeLevelChanger, AlifeObject, AlifeObjectAbstract,
  AlifeObjectAnomalyZone, AlifeObjectCustomZone, AlifeObjectInherited, AlifeObjectSpaceRestrictor, ClsId,
  LastSpawnTime, LevelSpawnFile, LevelSpawnObject, SpawnPacketHeader,
};

use crate::{SpawnAiMap, SpawnGraphPoint};

/// No neighbour across an edge: past the map's nodes.
const NONE: u32 = (1 << 23) - 1;

/// A flat map of one-metre cells over `columns` along x by `rows` along z, every cell holding a node but `holes`,
/// each linked to the nodes across its four sides, named `guid`.
pub(crate) fn grid(columns: u32, rows: u32, holes: &[(u32, u32)], guid: Uuid) -> XrfResult<SpawnAiMap> {
  let cells: Vec<(u32, u32)> = (0..columns)
    .flat_map(|x| (0..rows).map(move |z| (x, z)))
    .filter(|cell| !holes.contains(cell))
    .collect();
  let index = |x: i64, z: i64| {
    cells
      .iter()
      .position(|cell| i64::from(cell.0) == x && i64::from(cell.1) == z)
      .map_or(NONE, |index| index as u32)
  };
  let nodes: Vec<LevelAiNode> = cells
    .iter()
    .map(|&(x, z)| {
      let (x, z) = (i64::from(x), i64::from(z));

      LevelAiNode {
        links: [index(x - 1, z), index(x, z + 1), index(x + 1, z), index(x, z - 1)],
        high_cover: [0; 4],
        low_cover: [0; 4],
        plane: Vector3d::new(0.0, 1.0, 0.0).to_compressed_normal(),
        packed_xz: (x * i64::from(rows) + z) as u32,
        packed_y: 0,
        light: 0,
      }
    })
    .collect();
  let header: LevelAiHeader = LevelAiHeader {
    version: LevelAiVersion::CsCop as u32,
    count: nodes.len() as u32,
    size: 1.0,
    size_y: 1.0,
    bounds: Aabb {
      min: Vector3d::new(0.0, 0.0, 0.0),
      max: Vector3d::new((columns - 1) as f32, 1.0, (rows - 1) as f32),
    },
    guid,
  };

  Ok(SpawnAiMap::new(
    header,
    &LevelAiNodes::from_nodes::<XRayByteOrder>(LevelAiVersion::CsCop, &nodes)?,
  ))
}

/// A graph point standing on its cell's node, connecting nowhere.
pub(crate) fn point(map: &SpawnAiMap, x: f32, z: f32) -> SpawnGraphPoint {
  let position: Vector3d = Vector3d::new(x, 0.0, z);

  SpawnGraphPoint {
    name: format!("point_{x}_{z}"),
    position,
    node: map.find_node(&position).expect("a node under the point"),
    locations: [0; 4],
    connection_point_name: String::new(),
    connection_level_name: String::new(),
  }
}

/// A single player object of `section` named `name` at `position`, as `level.spawn` lists it in Call of Pripyat.
fn object(section: &str, name: &str, position: Vector3d, inherited: AlifeObjectInherited) -> XrfResult<AlifeObject> {
  Ok(AlifeObject {
    header: SpawnPacketHeader {
      id: u16::MAX,
      net_action: 1,
      section: section.to_owned(),
      name: name.to_owned(),
      script_game_id: 0,
      script_rp: 254,
      position,
      direction: Vector3d::ZERO,
      respawn_time: 0,
      parent_id: u16::MAX,
      phantom_id: u16::MAX,
      script_flags: 33,
      version: 128,
      game_type: 1,
      script_version: 12,
      client_data: Vec::new(),
      spawn_id: u16::MAX,
    },
    clsid: ClsId::from_section(section)?,
    inherited,
  })
}

/// What a restrictor or zone holds of its own before xrAI places it.
fn restrictor() -> AlifeObjectSpaceRestrictor {
  AlifeObjectSpaceRestrictor {
    base: AlifeObjectAbstract {
      game_vertex_id: u16::MAX,
      distance: 0.0,
      direct_control: 1,
      level_vertex_id: u32::MAX,
      flags: 0,
      custom_data: String::new(),
      story_id: AlifeObjectAbstract::INVALID_STORY_ID,
      spawn_story_id: AlifeObjectAbstract::INVALID_STORY_ID,
    },
    shape: Vec::new(),
    restrictor_type: 0,
  }
}

/// A graph point named `name` at `x`, `z`, connecting to point `target` of level `level`, or nowhere when empty.
pub(crate) fn graph_point_object(name: &str, x: f32, z: f32, level: &str, target: &str) -> XrfResult<AlifeObject> {
  object(
    "graph_point",
    name,
    Vector3d::new(x, 0.0, z),
    AlifeObjectInherited::CseAlifeGraphPoint(Box::new(AlifeGraphPoint {
      connection_point_name: target.to_owned(),
      connection_level_name: level.to_owned(),
      location0: 0,
      location1: 0,
      location2: 0,
      location3: 0,
    })),
  )
}

/// The actor, named `name`, at `x`, `z`.
pub(crate) fn actor_object(name: &str, x: f32, z: f32) -> XrfResult<AlifeObject> {
  let position: Vector3d = Vector3d::new(x, 0.0, z);

  object(
    "actor",
    name,
    position,
    AlifeObjectInherited::SeActor(Box::new(AlifeActor::new_mock(position))),
  )
}

/// A level changer named `name` at `x`, `z`, leading to graph point `point` of level `level`.
pub(crate) fn changer_object(name: &str, x: f32, z: f32, level: &str, point: &str) -> XrfResult<AlifeObject> {
  object(
    "level_changer",
    name,
    Vector3d::new(x, 0.0, z),
    AlifeObjectInherited::SeLevelChanger(Box::new(AlifeLevelChanger {
      base: restrictor(),
      dest_game_vertex_id: u16::MAX,
      dest_level_vertex_id: u32::MAX,
      dest_position: Vector3d::ZERO,
      dest_direction: Vector3d::ZERO,
      dest_level_name: level.to_owned(),
      dest_graph_point: point.to_owned(),
      silent_mode: 0,
      enabled: 1,
      hint: String::new(),
    })),
  )
}

/// A radiation field named `name` at `x`, `z`, keeping `count` artefact points within `radius`.
pub(crate) fn zone_object(name: &str, x: f32, z: f32, radius: f32, count: u16) -> XrfResult<AlifeObject> {
  object(
    "zone_field_radioactive",
    name,
    Vector3d::new(x, 0.0, z),
    AlifeObjectInherited::CseAlifeAnomalousZone(Box::new(AlifeAnomalousZone {
      base: AlifeObjectAnomalyZone {
        base: AlifeObjectCustomZone {
          base: restrictor(),
          max_power: 1.0,
          owner_id: u32::MAX,
          enabled_time: 0,
          disabled_time: 0,
          start_time_shift: 0,
        },
        offline_interactive_radius: radius,
        artefact_spawn_count: count,
        artefact_position_offset: 0,
      },
      last_spawn_time: LastSpawnTime::Unset,
    })),
  )
}

/// `level.spawn` listing `objects` in order.
pub(crate) fn spawn_file(objects: &[AlifeObject]) -> XrfResult<LevelSpawnFile> {
  Ok(LevelSpawnFile {
    objects: objects
      .iter()
      .enumerate()
      .map(|(index, object)| LevelSpawnObject::from_alife_object::<XRayByteOrder>(index as u32, object))
      .collect::<XrfResult<_>>()?,
  })
}
