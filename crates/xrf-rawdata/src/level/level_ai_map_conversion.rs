use xrf_editable_level::{EditableAiMapNode, EditableAiMapParams, EditableAiMapTool};
use xrf_level::{BuildAimapFile, BuildAimapNode, BuildAimapParams, LevelAiHeader, LevelAiNode};
use xrf_math::{Aabb, Vector3d};

/// A level's AI map, `level.ai`, back into the AI map tool (`ESceneAIMapTool`), and the tool out to the `build.aimap`
/// the Build hands xrAI (`ESceneAIMapTool::Export`, `LevelEditor/Edit/ESceneAIMapTools_Export.cpp:31`).
///
/// xrAI keeps each node's cell, height, plane and links, so they come back as the nodes, in the level's order and
/// packed against a box whose floor and height are the map's, so each height is the one stored. It keeps the patch size
/// as the cell size, but not the generator's other settings or the editor's own, which take a new tool's. The covers
/// xrAI computes are not kept, since it computes them again.
pub(crate) struct LevelAiMapConversion;

impl LevelAiMapConversion {
  /// The tool of a level's AI map of `header` and its `nodes`, decoded.
  pub(crate) fn decompile(header: &LevelAiHeader, nodes: impl IntoIterator<Item = LevelAiNode>) -> EditableAiMapTool {
    let params: EditableAiMapParams = EditableAiMapParams {
      patch_size: header.size,
      ..EditableAiMapParams::default()
    };
    let bounds: Aabb = Aabb {
      min: header.bounds.min,
      max: Vector3d::new(
        header.bounds.max.x,
        header.bounds.min.y + header.size_y,
        header.bounds.max.z,
      ),
    };
    let count: u32 = header.count;

    EditableAiMapTool::new(
      bounds,
      params,
      nodes
        .into_iter()
        .map(|node: LevelAiNode| {
          let position: Vector3d = header.get_node_position(&node);

          EditableAiMapNode {
            links: node
              .links
              .map(|link| if link < count { link } else { EditableAiMapNode::NO_LINK }),
            plane: node.plane,
            x: Self::get_cell(position.x, header.size),
            z: Self::get_cell(position.z, header.size),
            y: node.packed_y,
            flags: 0,
          }
        })
        .collect(),
    )
  }

  /// The `build.aimap` the Build exports from `tool`: each node unpacked against the tool's box and packed again
  /// against the box around every node (`CalculateNodesBBox`, `PackPosition`), in the tool's order.
  pub(crate) fn export(tool: &EditableAiMapTool) -> BuildAimapFile {
    let positions: Vec<Vector3d> = tool.nodes.iter().map(|node| tool.get_node_position(node)).collect();
    let bounds: Aabb = positions.iter().fold(
      Aabb {
        min: Vector3d::new(f32::MAX, f32::MAX, f32::MAX),
        max: Vector3d::new(f32::MIN, f32::MIN, f32::MIN),
      },
      |mut bounds, position| {
        bounds.include_point(position);
        bounds
      },
    );

    BuildAimapFile {
      version: BuildAimapFile::NARROW_LINKS_VERSION,
      bounds,
      params: BuildAimapParams {
        patch_size: tool.params.patch_size,
        test_height: tool.params.test_height,
        can_up: tool.params.can_up,
        can_down: tool.params.can_down,
      },
      nodes: tool
        .nodes
        .iter()
        .zip(&positions)
        .map(|(node, position)| BuildAimapNode {
          links: node.links,
          plane: node.plane,
          x: Self::pack_axis(position.x / tool.params.patch_size),
          y: Self::pack_height(position.y, &bounds),
          z: Self::pack_axis(position.z / tool.params.patch_size),
        })
        .collect(),
    }
  }

  /// The cell a coordinate falls in, in patches from the world origin: the nearest, since a node stands on a multiple
  /// of the patch size that the map's floor and float rounding move off it by a hair.
  fn get_cell(coordinate: f32, patch_size: f32) -> i16 {
    (coordinate / patch_size).round().clamp(-32767.0, 32767.0) as i16
  }

  /// `PackPosition`'s cell, `iFloor(coordinate / patch + EPS_L)` clamped to a signed 16 bits.
  fn pack_axis(cells: f32) -> i16 {
    (cells + xrf_math::EPS_L).floor().clamp(-32767.0, 32767.0) as i16
  }

  /// `PackPosition`'s height, `iFloor(65535 * (y - floor) / extent + EPS_L)` clamped to 16 bits; the floor for a box
  /// of no height, where the editor would divide by zero.
  fn pack_height(y: f32, bounds: &Aabb) -> u16 {
    let extent: f32 = bounds.max.y - bounds.min.y;

    if extent <= 0.0 {
      return 0;
    }

    (65535.0 * (y - bounds.min.y) / extent + xrf_math::EPS_L)
      .floor()
      .clamp(0.0, 65535.0) as u16
  }
}
