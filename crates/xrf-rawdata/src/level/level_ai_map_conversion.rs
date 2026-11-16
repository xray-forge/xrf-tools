use xrf_editable_level::{EditableAiMapNode, EditableAiMapParams, EditableAiMapTool};
use xrf_level::{BuildAimapFile, BuildAimapNode, BuildAimapParams, LevelAiHeader, LevelAiNode};
use xrf_math::{Aabb, AiNodePosition, EPS_L, Vector3d};

/// A level's AI map, `level.ai`, back into the AI map tool (`ESceneAIMapTool`), and the tool out to the `build.aimap`
/// the Build hands xrAI (`ESceneAIMapTool::Export`, `LevelEditor/Edit/ESceneAIMapTools_Export.cpp:31`).
///
/// xrAI keeps each node's cell, height, plane and links, so they come back as the nodes, in the level's order. It keeps
/// the patch size as the cell size, but not the generator's other settings or the editor's own, which take a new
/// tool's. The covers xrAI computes are not kept, since it computes them again.
///
/// The heights come back as the `build.aimap` held them, so that xrAI packs them again into the level's: the tool's box
/// is the level's, which is the box the export wrote, and each height is one whose export xrAI saves as the level's
/// ([`Self::get_tool_height`]).
pub(crate) struct LevelAiMapConversion;

impl LevelAiMapConversion {
  /// The tool of a level's AI map of `header` and its `nodes`, decoded.
  pub(crate) fn decompile(header: &LevelAiHeader, nodes: impl IntoIterator<Item = LevelAiNode>) -> EditableAiMapTool {
    let params: EditableAiMapParams = EditableAiMapParams {
      patch_size: header.size,
      ..EditableAiMapParams::default()
    };
    let count: u32 = header.count;

    EditableAiMapTool::new(
      header.bounds,
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
            position: AiNodePosition {
              x: Self::get_cell(position.x, header.size),
              y: Self::get_tool_height(node.packed_y, header),
              z: Self::get_cell(position.z, header.size),
            },
            flags: 0,
          }
        })
        .collect(),
    )
  }

  /// The `build.aimap` the Build exports from `tool`: each node unpacked against the tool's box and packed again
  /// against the box around every node (`CalculateNodesBBox`, `PackPosition`), in the tool's order; each height moves
  /// between the boxes in double precision ([`Self::get_exported_height`]).
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
          position: AiNodePosition {
            y: Self::get_exported_height(node.position.y, &tool.bounds, &bounds),
            ..AiNodePosition::pack(position, tool.params.patch_size, &bounds)
          },
        })
        .collect(),
    }
  }

  /// The tool height whose export xrAI saves as a node's `level_height`.
  ///
  /// xrAI unpacks each exported height across the export's box, `k / 65535 * extent + floor`, and saves it packed
  /// across the nodes' own box, whose floor and top are the lowest and highest exported heights (0 and 65535, as the
  /// export packs across the box around every node), but whose height is `extent + EPS_L` (`CalculateHeight`). The
  /// level's header so keeps the export's floor and top, and the stored height is
  /// `iFloor(k * extent / (extent + EPS_L) + EPS_L)`: a step below the exported one on most nodes, since the ratio falls
  /// short of one by about `EPS_L / extent`.
  ///
  /// The inverse searches the few heights around `level_height * size_y / extent` for those xrAI saves as it; the
  /// export keeps them, its box being the tool's ([`Self::get_exported_height`]). Two heights a step apart can save as
  /// one, where the ratio's shortfall adds up to a step; the higher is taken, so that the top node comes back at 65535
  /// and the export's box is the level's again. The floor node stays at 0. Where float rounding in another build of
  /// xrAI stored a height this one cannot save, the nearest is kept.
  fn get_tool_height(level_height: u16, header: &LevelAiHeader) -> u16 {
    let floor: f32 = header.bounds.min.y;
    let extent: f32 = header.bounds.max.y - floor;

    if level_height == 0 || extent <= 0.0 {
      return 0;
    }

    let estimate: i32 = (f32::from(level_height) * header.size_y / extent).round() as i32;
    let save = |height: u16| {
      AiNodePosition::pack_height(
        AiNodePosition::unpack_height(height, floor, extent),
        floor,
        header.size_y,
      )
    };

    (estimate - 3..=estimate + 3)
      .rev()
      .filter_map(|height| u16::try_from(height).ok())
      .find(|&height| save(height) == level_height)
      .unwrap_or_else(|| estimate.clamp(0, i32::from(u16::MAX)) as u16)
  }

  /// A height of the tool's box `from` packed again across the export's box `to`, as `PackPosition` packs the node's
  /// unpacked position, but in double precision. In single precision the node's position rounds by up to half a unit
  /// in the last place, which at tens of metres is more than the `EPS_L` the packing adds, so the editor's round trip
  /// drops a height a step on many nodes even where the boxes are one, and a decompiled tool could not export every
  /// height xrAI compiled a level from.
  fn get_exported_height(height: u16, from: &Aabb, to: &Aabb) -> u16 {
    let from_floor: f64 = f64::from(from.min.y);
    let to_floor: f64 = f64::from(to.min.y);
    let to_extent: f64 = f64::from(to.max.y) - to_floor;

    if to_extent <= 0.0 {
      return 0;
    }

    let y: f64 = f64::from(height) / f64::from(u16::MAX) * (f64::from(from.max.y) - from_floor) + from_floor;

    (f64::from(u16::MAX) * (y - to_floor) / to_extent + f64::from(EPS_L))
      .floor()
      .clamp(0.0, f64::from(u16::MAX)) as u16
  }

  /// The cell a coordinate falls in, in patches from the world origin: the nearest, since a node stands on a multiple
  /// of the patch size that the map's floor and float rounding move off it by a hair.
  fn get_cell(coordinate: f32, patch_size: f32) -> i16 {
    (coordinate / patch_size).round().clamp(-32767.0, 32767.0) as i16
  }
}
