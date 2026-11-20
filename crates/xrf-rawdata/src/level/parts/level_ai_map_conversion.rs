use xrf_editable_level::{EditableAiMapNode, EditableAiMapParams, EditableAiMapTool};
use xrf_level::{BuildAimapFile, BuildAimapNode, BuildAimapParams, LevelAiHeader, LevelAiNode};
use xrf_math::{Aabb, AiNodePosition, EPS_L, Vector3d};

/// A level's AI map, `level.ai`, back into the AI map tool, and the tool out to the `build.aimap` the Build hands xrAI
/// (`ESceneAIMapTool::Export`).
///
/// The nodes' cells, heights, planes and links come back in the level's order, and the patch size as the cell size.
/// The generator's other settings and the editor's own take a new tool's; covers are dropped, as xrAI recomputes them.
///
/// The heights come back as the `build.aimap` held them, so that xrAI packs them into the level's again
/// ([`Self::get_tool_height`]).
pub(crate) struct LevelAiMapConversion;

impl LevelAiMapConversion {
  /// The tool of a level's AI map, `header` and its `nodes`.
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

  /// The `build.aimap` the Build exports from `tool`: each node, in the tool's order, repacked from the tool's box into
  /// the box around every node (`CalculateNodesBBox`, `PackPosition`), heights in double precision
  /// ([`Self::get_exported_height`]).
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
  /// xrAI unpacks an exported height `k` across the export's box and repacks it across the nodes' box, which keeps
  /// the export's floor and top but is `extent + EPS_L` tall (`CalculateHeight`). It stores
  /// `iFloor(k * extent / (extent + EPS_L) + EPS_L)`, a step below `k` on most nodes.
  ///
  /// The inverse searches a few heights around `level_height * size_y / extent` for one xrAI saves as it. Where two
  /// do, the higher is taken, so the top node comes back at 65535 and the export's box is the level's again. Where
  /// another xrAI build's float rounding stored a height none saves as, the nearest is kept.
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

  /// A height of the tool's box `from` repacked across the export's box `to`, as `PackPosition` does but in double
  /// precision.
  ///
  /// In single precision, rounding at tens of metres exceeds the `EPS_L` the packing adds, so the editor drops many
  /// heights a step even between equal boxes, and a decompiled tool could not export every height xrAI compiled from.
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

  /// The cell a coordinate falls in, in patches from the world origin, rounded to the nearest: float rounding moves a
  /// node a hair off its multiple of the patch size.
  fn get_cell(coordinate: f32, patch_size: f32) -> i16 {
    (coordinate / patch_size).round().clamp(-32767.0, 32767.0) as i16
  }
}
