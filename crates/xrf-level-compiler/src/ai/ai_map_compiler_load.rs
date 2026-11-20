use xrf_error::{XrfError, XrfResult};
use xrf_level::{BuildAimapFile, BuildAimapNode, LevelAiGrid};
use xrf_math::{AiNodePosition, Vector3d};

use crate::ai::ai_map_compiler::AiMapCompiler;
use crate::ai::ai_vertex::AiVertex;

impl AiMapCompiler {
  /// The map's nodes as xrAI loads them (`xrLoad`, `xrAI/compiler_load.cpp`), in `build.aimap`'s order.
  ///
  /// Each position goes through the save's packed form over the map's box and back, and each plane normal is
  /// decompressed and normalised again (`Fplane::build`), so both land where xrAI's do.
  pub(crate) fn load(aimap: &BuildAimapFile) -> XrfResult<Vec<AiVertex>> {
    if aimap.version != BuildAimapFile::NARROW_LINKS_VERSION {
      return Err(XrfError::new_invalid_error(format!(
        "Build AI map version {} is not the {} xrAI loads",
        aimap.version,
        BuildAimapFile::NARROW_LINKS_VERSION
      )));
    }

    if aimap.nodes.is_empty() {
      return Err(XrfError::new_invalid_error("Build AI map holds no node to compile"));
    }

    let (bounds, size) = (&aimap.bounds, aimap.params.patch_size);
    let grid: LevelAiGrid = LevelAiGrid::new(bounds, size)?;
    let count: usize = aimap.nodes.len();

    aimap
      .nodes
      .iter()
      .enumerate()
      .map(|(index, node): (usize, &BuildAimapNode)| {
        let (x, z) =
          grid.unpack(grid.compress_cell(f32::from(node.position.x) * size, f32::from(node.position.z) * size)?);
        let links: [u32; 4] = node.links.map(|link| link & AiVertex::NO_LINK);

        // The later passes walk the links by index, so one past the nodes is refused here rather than read there.
        if let Some(link) = links
          .iter()
          .find(|link| **link != AiVertex::NO_LINK && **link as usize >= count)
        {
          return Err(XrfError::new_invalid_error(format!(
            "Build AI map node {index} links to node {link}, past the {count} it holds"
          )));
        }

        Ok(AiVertex {
          links,
          normal: Vector3d::from_compressed_normal(node.plane).normalize(),
          position: Vector3d::new(
            x,
            AiNodePosition::unpack_height(node.position.y, bounds.min.y, bounds.max.y - bounds.min.y),
            z,
          ),
          high_cover: [0.0; 4],
          low_cover: [0.0; 4],
        })
      })
      .collect()
  }
}
