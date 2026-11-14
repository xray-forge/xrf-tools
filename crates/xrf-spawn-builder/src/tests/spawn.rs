use uuid::Uuid;
use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, XRayByteOrder};
use xrf_error::XrfResult;
use xrf_level::{LevelAiHeader, LevelAiNode, LevelAiNodes, LevelAiVersion};
use xrf_math::{Aabb, Vector3d};
use xrf_spawn::{
  AlifeGraphPoint, GraphCrossTableCell, LevelSpawnFile, LevelSpawnObject, SpawnPacket, SpawnPacketHeader,
};

use crate::{SpawnAiMap, SpawnCrossTable, SpawnGraphPointRemoval, SpawnGraphPoints};

/// No neighbour across an edge: past the map's nodes.
const NONE: u32 = (1 << 23) - 1;

/// A flat node at cell `z` of the map's one row, at `height` in the map's ten metres, linked back and front.
fn node(z: u32, height: u16, back: u32, front: u32) -> LevelAiNode {
  LevelAiNode {
    links: [NONE, front, NONE, back],
    high_cover: [0; 4],
    low_cover: [0; 4],
    plane: Vector3d::new(0.0, 1.0, 0.0).to_compressed_normal(),
    packed_xz: z,
    packed_y: height,
    light: 0,
  }
}

/// One row of one-metre cells from z 0 to 7: a strip over cells 0 to 5 (node 4 a floor above node 3, linked to
/// nothing, and cell 4 holding node 5), and node 7 at cell 7, cut off.
fn map() -> XrfResult<SpawnAiMap> {
  let nodes: [LevelAiNode; 8] = [
    node(0, 0, NONE, 1),
    node(1, 0, 0, 2),
    node(2, 0, 1, 3),
    node(3, 0, 2, 5),
    node(3, 65535, NONE, NONE),
    node(4, 0, 3, 6),
    node(5, 0, 5, NONE),
    node(7, 0, NONE, NONE),
  ];
  let header: LevelAiHeader = LevelAiHeader {
    version: LevelAiVersion::CsCop as u32,
    count: nodes.len() as u32,
    size: 1.0,
    size_y: 10.0,
    bounds: Aabb {
      min: Vector3d::new(0.0, 0.0, 0.0),
      max: Vector3d::new(0.0, 10.0, 7.0),
    },
    guid: Uuid::nil(),
  };

  Ok(SpawnAiMap::new(
    header,
    &LevelAiNodes::from_nodes::<XRayByteOrder>(LevelAiVersion::CsCop, &nodes)?,
  ))
}

fn cell(game_vertex_id: u16, cells: u32) -> GraphCrossTableCell {
  GraphCrossTableCell {
    game_vertex_id,
    distance: cells as f32,
  }
}

#[test]
fn gives_each_node_the_nearest_graph_point_a_tie_to_the_earlier() -> XrfResult {
  let cells: Vec<GraphCrossTableCell> = SpawnCrossTable::build(&map()?, &[0, 5])?;

  assert_eq!(
    cells,
    vec![
      cell(0, 0),
      cell(0, 1),
      // Two steps from either point: the earlier keeps it.
      cell(0, 2),
      cell(1, 1),
      // Linked to nothing, and cut off: the first vertex, `u32::MAX` cells.
      cell(0, u32::MAX),
      cell(1, 0),
      cell(1, 1),
      cell(0, u32::MAX),
    ]
  );

  Ok(())
}

#[test]
fn reaches_only_through_nodes_a_point_takes() -> XrfResult {
  // Node 1 stays the first point's at one step, so the second, at node 3, reaches node 0 no more than it reaches 1.
  let cells: Vec<GraphCrossTableCell> = SpawnCrossTable::build(&map()?, &[1, 3])?;

  assert_eq!(&cells[..4], &[cell(0, 1), cell(0, 0), cell(0, 1), cell(1, 0)]);

  Ok(())
}

#[test]
fn refuses_a_point_off_the_map() -> XrfResult {
  let error: String = SpawnCrossTable::build(&map()?, &[8])
    .expect_err("expect node 8 of eight refused")
    .to_string();

  assert!(error.contains("node 8 of a map of 8"), "Unexpected error: {error}");

  Ok(())
}

#[test]
fn finds_the_node_nearest_at_or_under_a_position_in_its_cell() -> XrfResult {
  let map: SpawnAiMap = map()?;

  assert_eq!(map.find_node(&Vector3d::new(0.0, 0.5, 3.2)), Some(3));
  assert_eq!(map.find_node(&Vector3d::new(0.0, 10.5, 3.0)), Some(4));
  // Under both floors, the nearer over it.
  assert_eq!(map.find_node(&Vector3d::new(0.0, -1.0, 3.0)), Some(3));
  assert_eq!(map.find_node(&Vector3d::new(0.0, 0.0, 6.0)), None);
  assert!(!map.is_valid_position(&Vector3d::new(2.0, 0.0, 3.0)));

  Ok(())
}

/// `level.spawn` holding a graph point at each position, in order.
fn spawn_list(positions: &[Vector3d]) -> XrfResult<LevelSpawnFile> {
  let objects = positions
    .iter()
    .enumerate()
    .map(|(index, position)| {
      let packet: SpawnPacket<AlifeGraphPoint> = SpawnPacket {
        header: SpawnPacketHeader {
          id: u16::MAX,
          net_action: 1,
          section: String::from("graph_point"),
          name: format!("graph_point_{index}"),
          script_game_id: 0,
          script_rp: 254,
          position: *position,
          direction: Vector3d::ZERO,
          respawn_time: 0,
          parent_id: u16::MAX,
          phantom_id: u16::MAX,
          script_flags: 33,
          version: 128,
          game_type: u16::MAX,
          script_version: 12,
          client_data: Vec::new(),
          spawn_id: u16::MAX,
        },
        state: AlifeGraphPoint {
          connection_point_name: String::new(),
          connection_level_name: String::new(),
          location0: index as u8,
          location1: 0,
          location2: 0,
          location3: 0,
        },
      };
      let mut bytes: ChunkWriter = ChunkWriter::new();

      packet.write::<XRayByteOrder>(&mut bytes)?;

      LevelSpawnObject::read::<XRayByteOrder, _>(index as u32, &mut ChunkReader::from_bytes(&bytes.buffer)?)
    })
    .collect::<XrfResult<_>>()?;

  Ok(LevelSpawnFile { objects })
}

#[test]
fn places_graph_points_and_leaves_out_what_xrai_removes() -> XrfResult {
  let points: SpawnGraphPoints = SpawnGraphPoints::read(
    &spawn_list(&[
      Vector3d::new(0.0, 0.2, 0.1),
      Vector3d::new(0.0, 0.2, 0.11),
      Vector3d::new(0.0, 0.0, 6.0),
      Vector3d::new(0.0, 0.5, -0.3),
      Vector3d::new(0.0, 0.0, 4.0),
    ])?,
    &map()?,
  )?;

  assert_eq!(
    points
      .points
      .iter()
      .map(|point| (point.name.as_str(), point.node, point.locations[0]))
      .collect::<Vec<_>>(),
    vec![("graph_point_0", 0, 0), ("graph_point_4", 5, 4)]
  );
  assert_eq!(
    points
      .removed
      .iter()
      .map(|point| (point.name.as_str(), point.removal))
      .collect::<Vec<_>>(),
    vec![
      ("graph_point_1", SpawnGraphPointRemoval::TooClose),
      ("graph_point_2", SpawnGraphPointRemoval::OutsideMap),
      ("graph_point_3", SpawnGraphPointRemoval::SharedNode),
    ]
  );

  Ok(())
}
