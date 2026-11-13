use uuid::Uuid;
use xrf_chunk::XRayByteOrder;
use xrf_error::XrfResult;
use xrf_level::{LevelAiHeader, LevelAiNode, LevelAiNodes, LevelAiVersion};
use xrf_math::{Aabb, Vector3d};

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
