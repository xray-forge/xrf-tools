use xrf_error::{XrfError, XrfResult};
use xrf_level::{BuildAimapFile, BuildAimapNode};
use xrf_math::{AiNodePosition, Vector3d};

use crate::ai::ai_grid::AiGrid;
use crate::ai::ai_map_compiler::AiMapCompiler;
use crate::ai::ai_vertex::AiVertex;

impl AiMapCompiler {
  /// The map's nodes as xrAI loads them (`xrLoad`, `xrAI/compiler_load.cpp:267`), in `build.aimap`'s order.
  ///
  /// Each cell goes through the packed form xrAI's save uses, over the map's box, and back, which is where its position
  /// lands; the height is unpacked across the box. Each plane normal is decompressed and normalised again
  /// (`Fplane::build`), as the save compresses it.
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
    let grid: AiGrid = AiGrid::new(bounds, size);

    Ok(
      aimap
        .nodes
        .iter()
        .map(|node: &BuildAimapNode| {
          let (x, z) = grid.unpack(grid.pack(f32::from(node.position.x) * size, f32::from(node.position.z) * size));

          AiVertex {
            links: node.links.map(|link| link & AiVertex::NO_LINK),
            normal: Self::normalize(&Vector3d::from_compressed_normal(node.plane)),
            position: Vector3d::new(
              x,
              AiNodePosition::unpack_height(node.position.y, bounds.min.y, bounds.max.y - bounds.min.y),
              z,
            ),
            high_cover: [0.0; 4],
            low_cover: [0.0; 4],
          }
        })
        .collect(),
    )
  }

  /// `Fvector::normalize`, in single precision: each component times `sqrt(1 / (x * x + y * y + z * z))`.
  fn normalize(vector: &Vector3d) -> Vector3d {
    let scale: f32 = (1.0 / (vector.x * vector.x + vector.y * vector.y + vector.z * vector.z)).sqrt();

    Vector3d::new(vector.x * scale, vector.y * scale, vector.z * scale)
  }
}
