use uuid::Uuid;
use xrf_chunk::XRayByteOrder;
use xrf_error::XrfResult;
use xrf_level::{LevelAiHeader, LevelAiNode, LevelAiNodes, LevelAiVersion};
use xrf_math::{Aabb, Vector3d};

use crate::{SpawnAiMap, SpawnCrossTable, SpawnGameEdge, SpawnGraphPoint, SpawnLevelGameGraph};

/// No neighbour across an edge: past the map's nodes.
const NONE: u32 = (1 << 23) - 1;

/// A flat map of one-metre cells over `columns` along x by `rows` along z, every cell holding a node but `holes`,
/// each linked to the nodes across its four sides.
fn grid(columns: u32, rows: u32, holes: &[(u32, u32)]) -> XrfResult<SpawnAiMap> {
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
    guid: Uuid::nil(),
  };

  Ok(SpawnAiMap::new(
    header,
    &LevelAiNodes::from_nodes::<XRayByteOrder>(LevelAiVersion::CsCop, &nodes)?,
  ))
}

/// A graph point standing on its cell's node.
fn point(map: &SpawnAiMap, x: f32, z: f32) -> SpawnGraphPoint {
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

fn build(map: &SpawnAiMap, points: &[SpawnGraphPoint]) -> XrfResult<SpawnLevelGameGraph> {
  let nodes: Vec<u32> = points.iter().map(|point| point.node).collect();

  SpawnLevelGameGraph::build(map, points, &SpawnCrossTable::build(map, &nodes)?)
}

fn targets(edges: &[SpawnGameEdge]) -> Vec<u16> {
  let mut targets: Vec<u16> = edges.iter().map(|edge| edge.target).collect();

  targets.sort_unstable();
  targets
}

#[test]
fn joins_bordering_regions_and_drops_each_triangles_longest_edge() -> XrfResult {
  let map: SpawnAiMap = grid(6, 6, &[])?;
  let points: Vec<SpawnGraphPoint> = vec![point(&map, 0.0, 0.0), point(&map, 5.0, 0.0), point(&map, 0.0, 5.0)];
  let graph: SpawnLevelGameGraph = build(&map, &points)?;

  // The three regions all border each other; the diagonal, the triangle's longest side, goes both ways.
  assert_eq!(targets(&graph.edges[0]), vec![1, 2]);
  assert_eq!(targets(&graph.edges[1]), vec![0]);
  assert_eq!(targets(&graph.edges[2]), vec![0]);

  // Open ground: each edge the straight distance across it.
  for edges in &graph.edges {
    for edge in edges {
      assert_eq!(edge.distance, 5.0);
    }
  }

  Ok(())
}

#[test]
fn weighs_a_walled_edge_by_the_path_around_the_wall() -> XrfResult {
  // A wall along x 2 from z 0 to 3, open only at z 4.
  let map: SpawnAiMap = grid(5, 5, &[(2, 0), (2, 1), (2, 2), (2, 3)])?;
  let points: Vec<SpawnGraphPoint> = vec![point(&map, 0.0, 0.0), point(&map, 4.0, 0.0)];
  let graph: SpawnLevelGameGraph = build(&map, &points)?;
  let (there, back) = (graph.edges[0][0], graph.edges[1][0]);

  assert_eq!((there.target, back.target), (1, 0));

  // Longer than the four metres straight through the wall, no longer than the twelve cells around it.
  for edge in [there, back] {
    assert!(
      edge.distance > 4.0 && edge.distance <= 12.0,
      "Unexpected weight {}",
      edge.distance
    );
  }

  Ok(())
}

#[test]
fn leaves_regions_no_link_joins_without_an_edge() -> XrfResult {
  // A wall across the whole map: the two halves' regions never border, so no edge joins them.
  let map: SpawnAiMap = grid(5, 3, &[(2, 0), (2, 1), (2, 2)])?;
  let points: Vec<SpawnGraphPoint> = vec![point(&map, 0.0, 0.0), point(&map, 4.0, 0.0)];
  let graph: SpawnLevelGameGraph = build(&map, &points)?;

  assert!(graph.edges.iter().all(Vec::is_empty));

  Ok(())
}
