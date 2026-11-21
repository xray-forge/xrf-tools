use uuid::Uuid;
use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;
use xrf_level::{LevelAiHeader, LevelAiNode, LevelAiNodes, LevelAiVersion};
use xrf_math::{Aabb, AiNodePosition, Vector3d};
use xrf_spawn::{
  AlifeGraphPoint, GraphCrossTableCell, LevelSpawnFile, LevelSpawnObject, SpawnPacket, SpawnPacketHeader,
  SpawnSectionClasses,
};

use crate::gamedata::spawn_level_files::SpawnLevelFiles;
use crate::graph::spawn_graph_point_removal::SpawnGraphPointRemoval;
use crate::graph::spawn_graph_points::SpawnGraphPoints;
use crate::map::spawn_ai_map::SpawnAiMap;
use crate::map::spawn_cross_table::SpawnCrossTable;

// A link past the map's nodes: no neighbour.
const NONE: u32 = (1 << 23) - 1;

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

// One row of cells: a strip over cells 0 to 5, node 4 an unlinked floor above node 3, and node 7 cut off at cell 7.
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

  SpawnAiMap::new(header, &LevelAiNodes::from_nodes(LevelAiVersion::CsCop, &nodes)?)
}

fn cell(game_vertex_id: u16, cells: u32) -> GraphCrossTableCell {
  GraphCrossTableCell {
    game_vertex_id,
    distance: cells as f32,
  }
}

#[test]
fn refuses_a_header_its_grid_or_nodes_cannot_follow() -> XrfResult {
  let header: LevelAiHeader = LevelAiHeader {
    count: 1,
    ..map()?.get_header().clone()
  };
  let nodes: LevelAiNodes = LevelAiNodes::from_nodes(LevelAiVersion::CsCop, &[node(0, 0, NONE, NONE)])?;
  let refused = |header: LevelAiHeader| SpawnAiMap::new(header, &nodes).is_err();

  assert!(SpawnAiMap::new(header.clone(), &nodes).is_ok());
  assert!(
    refused(LevelAiHeader { size: 0.0, ..header }),
    "a grid that unpacks nothing"
  );
  assert!(
    refused(LevelAiHeader { count: 2, ..header }),
    "a count past the nodes held"
  );

  Ok(())
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
  // Node 1 stays the first point's, so the second, at node 3, cannot reach node 0 through it.
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

/// A rough map of `columns` by `rows` 0.7 m cells off the origin, from `seed`: about a third of the cells empty, some
/// holding a second floor above the first, each node at its own height on its own tilted plane. Unlinked: only the
/// nearest-node searches read it.
fn rough_map(columns: u32, rows: u32, seed: u64) -> XrfResult<SpawnAiMap> {
  let mut state: u64 = seed;
  let mut next = move || {
    state ^= state << 13;
    state ^= state >> 7;
    state ^= state << 17;
    state
  };
  let mut nodes: Vec<LevelAiNode> = Vec::new();

  for packed in 0..columns * rows {
    let floors: u64 = match next() % 6 {
      0 | 1 => 0,
      2 => 2,
      _ => 1,
    };

    for _ in 0..floors {
      let tilt = |value: u64| (value % 100) as f32 / 250.0 - 0.2;

      nodes.push(LevelAiNode {
        links: [NONE; 4],
        high_cover: [0; 4],
        low_cover: [0; 4],
        plane: Vector3d::new(tilt(next()), 1.0, tilt(next())).to_compressed_normal(),
        packed_xz: packed,
        packed_y: (next() % 65536) as u16,
        light: 0,
      });
    }
  }

  let header: LevelAiHeader = LevelAiHeader {
    version: LevelAiVersion::CsCop as u32,
    count: nodes.len() as u32,
    size: 0.7,
    size_y: 6.0,
    bounds: Aabb {
      min: Vector3d::new(-13.3, -2.0, 7.1),
      max: Vector3d::new(-13.3 + (columns - 1) as f32 * 0.7, 4.0, 7.1 + (rows - 1) as f32 * 0.7),
    },
    guid: Uuid::nil(),
  };

  SpawnAiMap::new(header, &LevelAiNodes::from_nodes(LevelAiVersion::CsCop, &nodes)?)
}

// The search by rings must find the node the engine's walk over every node finds, wherever the position lies: over a
// node, over an empty cell, on a cell's edge, off the grid near or far, above or under the floors.
#[test]
fn finds_the_nearest_node_the_walk_over_every_node_finds() -> XrfResult {
  for (seed, columns, rows) in [
    (0x9E37_79B9_7F4A_7C15_u64, 23, 17),
    (0x2545_F491_4F6C_DD1D, 41, 9),
    (7, 1, 30),
  ] {
    let map: SpawnAiMap = rough_map(columns, rows, seed)?;
    let (min_x, min_z) = (-13.3_f32, 7.1_f32);
    let mut positions: Vec<Vector3d> = Vec::new();

    // A lattice half a cell apart, three cells past every side, at three heights; it lands on cell edges too.
    for x in -6..=(columns.cast_signed() * 2 + 6) {
      for z in -6..=(rows.cast_signed() * 2 + 6) {
        for y in [-3.0, 0.4, 5.5] {
          positions.push(Vector3d::new(
            min_x + x as f32 * 0.35 - 0.35,
            y,
            min_z + z as f32 * 0.35 - 0.35,
          ));
        }
      }
    }

    // Far off every side and corner.
    for (x, z) in [
      (-400.0, 10.0),
      (500.0, 10.0),
      (0.0, -350.0),
      (0.0, 700.0),
      (-900.0, -900.0),
      (1.0e6, 3.0),
    ] {
      positions.push(Vector3d::new(x, 1.0, z));
    }

    for position in &positions {
      assert_eq!(
        map.find_nearest_node(position),
        map.find_nearest_node_by_scan(position),
        "nearest node to {position:?} on the {columns} by {rows} map of seed {seed:#x}"
      );
    }
  }

  Ok(())
}

// The engine's `vertex_plane_y` takes its height from `vertex_position`, so both lookups must agree.
#[test]
fn takes_a_node_plane_through_its_own_position() -> XrfResult {
  // 257 is the first packed height whose division and reciprocal multiply land an ulp apart, at an extent of 10.
  let nodes: [LevelAiNode; 1] = [node(0, 257, NONE, NONE)];
  let map: SpawnAiMap = SpawnAiMap::new(
    LevelAiHeader {
      count: 1,
      ..map()?.get_header().clone()
    },
    &LevelAiNodes::from_nodes(LevelAiVersion::CsCop, &nodes)?,
  )?;
  let position: Vector3d = map.get_node_position(0);

  assert_eq!(position.y, AiNodePosition::unpack_height_as_built(257, 0.0, 10.0));
  assert_ne!(position.y, AiNodePosition::unpack_height(257, 0.0, 10.0));
  assert_eq!(map.get_plane_y(0, position.x, position.z), position.y);

  Ok(())
}

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

      packet.write(&mut bytes)?;

      LevelSpawnObject::read(index as u32, &mut ChunkReader::from_bytes(&bytes.buffer)?)
    })
    .collect::<XrfResult<_>>()?;

  Ok(LevelSpawnFile { objects })
}

#[test]
fn places_graph_points_and_leaves_out_what_xrai_removes() -> XrfResult {
  let list: LevelSpawnFile = spawn_list(&[
    Vector3d::new(0.0, 0.2, 0.1),
    Vector3d::new(0.0, 0.2, 0.11),
    Vector3d::new(0.0, 0.0, 6.0),
    Vector3d::new(0.0, 0.5, -0.3),
    Vector3d::new(0.0, 0.0, 4.0),
  ])?;
  let points: SpawnGraphPoints = SpawnGraphPoints::read(
    &SpawnLevelFiles::read_objects(&list, &SpawnSectionClasses::default())?,
    &map()?,
  );

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
