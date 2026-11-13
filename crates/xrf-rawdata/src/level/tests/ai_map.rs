use uuid::Uuid;
use xrf_editable_level::{EditableAiMapNode, EditableAiMapTool};
use xrf_level::{BuildAimapFile, LevelAiHeader, LevelAiNode};
use xrf_math::{Aabb, AiNodePosition, Vector3d};

use crate::level::level_ai_map_conversion::LevelAiMapConversion;

/// A map of cells 0.7 across over a box 14 metres square and 10 high, 21 cells along z.
fn header(count: u32) -> LevelAiHeader {
  LevelAiHeader {
    version: 10,
    count,
    size: 0.7,
    size_y: 10.0,
    bounds: Aabb {
      min: Vector3d::new(-7.0, -5.0, 14.0),
      max: Vector3d::new(7.0, 5.0, 28.0),
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

fn tool() -> EditableAiMapTool {
  LevelAiMapConversion::decompile(
    &header(2),
    [node((2, 3), 32768, [1, 99, 1, 5]), node((5, 0), 0, [0, 2, 0, 0])],
  )
}

/// Each node keeps its cell, counted from the world origin, its packed height over a box of the map's floor and
/// height, and its plane; a link past the map's nodes is no link. The covers are left for xrAI to compute again.
#[test]
fn decompiles_a_level_ai_into_the_ai_map_tool() {
  let tool: EditableAiMapTool = tool();

  assert_eq!(
    tool.nodes[0],
    EditableAiMapNode {
      links: [1, EditableAiMapNode::NO_LINK, 1, EditableAiMapNode::NO_LINK],
      plane: 0x1234,
      position: AiNodePosition { x: -8, y: 32768, z: 23 },
      flags: 0,
    },
    "cell 2 is 1.4 m past the floor at -7, the eighth cell west of the origin; cell 3 along z is 16.1 m, the 23rd"
  );
  assert_eq!(
    (
      tool.nodes[1].position.x,
      tool.nodes[1].position.z,
      tool.nodes[1].links[1]
    ),
    (-5, 20, EditableAiMapNode::NO_LINK)
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
  assert_eq!(exported.bounds.min.y, -5.0);
}
