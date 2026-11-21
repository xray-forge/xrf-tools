use std::fs;
use std::path::Path;

use uuid::{Uuid, uuid};
use xrf_chunk::ChunkWriter;
use xrf_error::XrfResult;
use xrf_math::Vector3d;
use xrf_spawn::{
  AlifeGraphPoint, AlifeObject, AlifeObjectInherited, ClsId, GraphHeader, LevelSpawnFile, LevelSpawnObject,
  SpawnALifeSpawnsChunk, SpawnArtefactSpawnsChunk, SpawnFile, SpawnGraphsChunk, SpawnHeaderChunk, SpawnPacketHeader,
  SpawnPatrolsChunk,
};

/// A root spawn holding nothing: no objects, levels, artefact points, patrols or graph vertices.
pub(crate) fn new_root_spawn() -> SpawnFile {
  let guid: Uuid = uuid!("78e55023-10b1-426f-9247-bb680e5fe0b7");

  SpawnFile {
    header: SpawnHeaderChunk {
      version: 10,
      guid,
      graph_guid: guid,
      objects_count: 0,
      levels_count: 0,
    },
    alife_spawn: SpawnALifeSpawnsChunk { objects: Vec::new() },
    artefact_spawn: SpawnArtefactSpawnsChunk { nodes: Vec::new() },
    patrols: SpawnPatrolsChunk { patrols: Vec::new() },
    graphs: SpawnGraphsChunk {
      header: GraphHeader {
        version: 10,
        vertices_count: 0,
        edges_count: 0,
        points_count: 0,
        guid,
        levels_count: 0,
      },
      levels: Vec::new(),
      vertices: Vec::new(),
      edges: Vec::new(),
      points: Vec::new(),
      cross_tables: Vec::new(),
    },
  }
}

/// An object of a level's spawn list, its class-dependent rest left opaque.
pub(crate) fn new_level_object(id: u32, section: &str, name: &str) -> LevelSpawnObject {
  LevelSpawnObject {
    id,
    // `M_SPAWN`, which every packet of a list opens with.
    net_action: 1,
    section: String::from(section),
    name: String::from(name),
    script_game_id: 0,
    script_rp: 0xFF,
    position: Vector3d::new(1.0, 2.0, 3.0),
    direction: Vector3d::ZERO,
    payload: vec![7; 12],
  }
}

/// A graph point of Call of Pripyat's spawn version, which the typed read accepts, listed under chunk `id`.
pub(crate) fn new_typed_level_object(id: u32, name: &str) -> XrfResult<LevelSpawnObject> {
  LevelSpawnObject::from_alife_object(
    id,
    &AlifeObject {
      header: SpawnPacketHeader {
        id: u16::MAX,
        net_action: 1,
        section: String::from("graph_point"),
        name: String::from(name),
        script_game_id: 0,
        script_rp: 0xFE,
        position: Vector3d::new(1.5, -2.0, 4.25),
        direction: Vector3d::ZERO,
        respawn_time: 0,
        parent_id: u16::MAX,
        phantom_id: u16::MAX,
        script_flags: SpawnPacketHeader::FLAG_SPAWN_VERSION | 1,
        version: 128,
        game_type: u16::MAX,
        script_version: 12,
        client_data: Vec::new(),
        spawn_id: u16::MAX,
      },
      clsid: ClsId::AiGraph,
      inherited: AlifeObjectInherited::CseAlifeGraphPoint(Box::new(AlifeGraphPoint {
        connection_point_name: String::new(),
        connection_level_name: String::new(),
        location0: 1,
        location1: 0,
        location2: 0,
        location3: 0,
      })),
    },
  )
}

/// Writes `objects` to `path` as a level's spawn list, answering the bytes written.
pub(crate) fn write_level_spawn_list(path: &Path, objects: Vec<LevelSpawnObject>) -> XrfResult<Vec<u8>> {
  let mut writer: ChunkWriter = ChunkWriter::new();

  LevelSpawnFile { objects }.write(&mut writer)?;

  let bytes: Vec<u8> = writer.into_bytes();

  fs::write(path, &bytes)?;

  Ok(bytes)
}
