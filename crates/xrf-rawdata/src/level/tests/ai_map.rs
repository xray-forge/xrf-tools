use uuid::Uuid;
use xrf_editable_level::{EditableAiMapNode, EditableAiMapTool};
use xrf_level::{BuildAimapFile, LevelAiHeader, LevelAiNode};
use xrf_math::{Aabb, AiNodePosition, EPS_L, Vector3d};

use crate::level::parts::level_ai_map_conversion::LevelAiMapConversion;

/// A map xrAI saved over nodes from a box `floor` to `top`, with cells 0.7 across over 14 metres square, 21 cells
/// along z: its height the box's plus `EPS_L`, as `CalculateHeight` gives it.
fn header(count: u32, floor: f32, top: f32) -> LevelAiHeader {
  LevelAiHeader {
    version: 10,
    count,
    size: 0.7,
    size_y: top - floor + EPS_L,
    bounds: Aabb {
      min: Vector3d::new(-7.0, floor, 14.0),
      max: Vector3d::new(7.0, top, 28.0),
    },
    guid: Uuid::nil(),
  }
}

fn node(cell: (u32, u32), packed_y: u16, links: [u32; 4]) -> LevelAiNode {
  LevelAiNode {
    links,
    high_cover: [1, 2, 3, 4],
    low_cover: [1, 2, 3, 4],
    plane: 0x1234,
    packed_xz: cell.0 * 21 + cell.1,
    packed_y,
    light: 15,
  }
}

/// A 10 m high map's top node, stored at 65528 across the map's 10.001 m, and its floor node.
fn tool() -> EditableAiMapTool {
  LevelAiMapConversion::decompile(
    &header(2, -5.0, 5.0),
    [node((2, 3), 65528, [1, 99, 1, 5]), node((5, 0), 0, [0, 2, 0, 0])],
  )
}

/// The height xrAI stores for a `build.aimap` height: unpacked across the export's box, packed across the map's.
fn save_height(exported: u16, exported_bounds: &Aabb, header: &LevelAiHeader) -> u16 {
  let floor: f32 = exported_bounds.min.y;

  AiNodePosition::pack_height(
    AiNodePosition::unpack_height(exported, floor, exported_bounds.max.y - floor),
    header.bounds.min.y,
    header.size_y,
  )
}

/// Each node keeps its cell, counted from the world origin, the height the export gave it over the map's box, and its
/// plane; a link past the map's nodes is no link. The covers are left for xrAI to compute again.
#[test]
fn decompiles_a_level_ai_into_the_ai_map_tool() {
  let tool: EditableAiMapTool = tool();

  assert_eq!(
    tool.nodes[0],
    EditableAiMapNode {
      links: [1, EditableAiMapNode::NO_LINK, 1, EditableAiMapNode::NO_LINK],
      plane: 0x1234,
      position: AiNodePosition { x: -8, y: 65535, z: 23 },
      flags: 0,
    },
    "cell 2 is 1.4 m past the floor at -7, the eighth cell west of the origin; cell 3 along z is 16.1 m, the 23rd; \
     the top node at the top of the box"
  );
  assert_eq!(
    (
      tool.nodes[1].position.x,
      tool.nodes[1].position.y,
      tool.nodes[1].position.z,
      tool.nodes[1].links[1]
    ),
    (-5, 0, 20, EditableAiMapNode::NO_LINK)
  );
  assert_eq!((tool.bounds.min.y, tool.bounds.max.y), (-5.0, 5.0));
  assert_eq!(tool.params.patch_size, 0.7);
  assert!(
    (tool.get_node_position(&tool.nodes[0]).x + 5.6).abs() < 1e-5,
    "the node stands where the level put it"
  );
}

/// The export packs each node again across the box around every node, as `ESceneAIMapTool::Export` does, keeping its
/// cell, plane and links.
#[test]
fn exports_the_tool_as_the_editor_writes_build_aimap() {
  let tool: EditableAiMapTool = tool();
  let exported: BuildAimapFile = LevelAiMapConversion::export(&tool);

  assert_eq!(exported.version, BuildAimapFile::NARROW_LINKS_VERSION);
  assert_eq!(exported.params.patch_size, 0.7);
  assert_eq!(
    exported
      .nodes
      .iter()
      .map(|node| (
        node.position.x,
        node.position.z,
        node.position.y,
        node.links,
        node.plane
      ))
      .collect::<Vec<_>>(),
    [
      (-8, 23, 65535, tool.nodes[0].links, 0x1234),
      (-5, 20, 0, tool.nodes[1].links, 0x1234),
    ],
    "the higher node at the top of the new box, the lower at its floor"
  );
  assert_eq!((exported.bounds.min.y, exported.bounds.max.y), (-5.0, 5.0));
}

/// A level's map decompiled and exported again gives xrAI the box and heights it compiled the level from: it saves
/// every height the level stores, which sits a step below the exported one on most nodes, and the level's header
/// again. Jupiter's floor and top, over a sweep of heights.
#[test]
fn exports_the_heights_xrai_saves_into_the_level() {
  let (floor, top): (f32, f32) = (-29.987507, 46.926617);
  let exported_bounds: Aabb = Aabb {
    min: Vector3d::new(-7.0, floor, 14.0),
    max: Vector3d::new(7.0, top, 28.0),
  };
  let header: LevelAiHeader = header(0, floor, top);
  let heights: Vec<u16> = (0..=u16::MAX).step_by(13).chain([1, 2, 65534, u16::MAX]).collect();
  let stored: Vec<u16> = heights
    .iter()
    .map(|&height| save_height(height, &exported_bounds, &header))
    .collect();
  let tool: EditableAiMapTool = LevelAiMapConversion::decompile(
    &LevelAiHeader {
      count: stored.len() as u32,
      ..header
    },
    stored
      .iter()
      .enumerate()
      .map(|(index, &packed_y)| node((0, index as u32 % 21), packed_y, [u32::MAX; 4])),
  );
  let exported: BuildAimapFile = LevelAiMapConversion::export(&tool);

  assert_eq!(
    (exported.bounds.min.y, exported.bounds.max.y),
    (floor, top),
    "the box the level was compiled from"
  );
  assert_eq!(
    exported
      .nodes
      .iter()
      .zip(&stored)
      .zip(&heights)
      .map(|((node, &stored), &height)| (
        height,
        node.position.y,
        save_height(node.position.y, &exported.bounds, &header),
        stored
      ))
      .filter(|(_, _, saved, stored)| saved != stored)
      .collect::<Vec<_>>(),
    [],
    "xrAI saves the level's heights again"
  );
  assert!(
    stored
      .iter()
      .zip(&heights)
      .filter(|(stored, height)| stored != height)
      .count()
      > heights.len() * 9 / 10,
    "most stored heights sit below the exported ones, so keeping them would move the map by a step"
  );
}
