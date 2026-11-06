use xrf_error::{XrfError, XrfResult};
use xrf_math::Vector2d;
use xrf_ogf::{OgfFile, OgfVertex, OgfVertexData};

use crate::object::decompile::geometry::bone_influences::BoneInfluences;
use crate::object::decompile::geometry::ogf_corner::OgfCorner;

/// One OGF child read back as the surface it was exported from: its texture and engine shader, and its full-detail
/// triangles. The export writes one child per surface (`CExportSkeleton::SSplit`), merging surfaces that share both.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct OgfSurface {
  pub(crate) texture: String,
  pub(crate) shader: String,
  pub(crate) triangles: Vec<[OgfCorner; 3]>,
}

impl OgfSurface {
  /// Every child of `file` that draws geometry, depth first, as the export wrote them.
  ///
  /// # Errors
  ///
  /// Returns an error for a child with static vertices, which this does not decompile yet, and for an index past its
  /// child's vertices.
  pub(crate) fn collect(file: &OgfFile) -> XrfResult<Vec<Self>> {
    let mut surfaces: Vec<Self> = Vec::new();

    Self::collect_into(file, &mut surfaces)?;

    Ok(surfaces)
  }

  fn collect_into(file: &OgfFile, surfaces: &mut Vec<Self>) -> XrfResult {
    if let (Some(texture), Some(geometry)) = (&file.texture, &file.geometry) {
      let vertices: &[OgfVertex] = match &geometry.vertices {
        Some(OgfVertexData::Skinned(vertices)) => vertices,
        Some(OgfVertexData::Static(_)) => {
          return Err(XrfError::new_not_implemented_error(
            "An OGF child with static vertices is not decompiled yet; only skeletons are",
          ));
        }
        None => {
          return Err(XrfError::new_not_implemented_error(
            "An OGF child's vertex layout is not decoded",
          ));
        }
      };

      let indices: &[u16] = Self::get_full_detail_indices(file, geometry.indices.as_deref().unwrap_or_default())?;
      let corner = |index: u16| -> XrfResult<OgfCorner> {
        let vertex: &OgfVertex = vertices.get(usize::from(index)).ok_or_else(|| {
          XrfError::new_invalid_error(format!("OGF index {index} is past its {} vertices", vertices.len()))
        })?;

        Ok(OgfCorner {
          position: vertex.position,
          normal: vertex.normal,
          texture_coordinate: Vector2d::new(vertex.texture_u, vertex.texture_v),
          influences: BoneInfluences::of_links(&vertex.links),
        })
      };

      surfaces.push(Self {
        texture: texture.texture_name.clone(),
        shader: texture.shader_name.clone(),
        triangles: indices
          .as_chunks::<3>()
          .0
          .iter()
          .map(|[a, b, c]| Ok([corner(*a)?, corner(*b)?, corner(*c)?]))
          .collect::<XrfResult<_>>()?,
      });
    }

    for child in file.children.iter().flat_map(|children| &children.nested) {
      Self::collect_into(child, surfaces)?;
    }

    Ok(())
  }

  /// The indices of the most detailed sliding window where the child is progressive, `FProgressive`'s window 0, which
  /// is every triangle the export wrote; all of them otherwise.
  fn get_full_detail_indices<'a>(file: &OgfFile, indices: &'a [u16]) -> XrfResult<&'a [u16]> {
    let Some(window) = file.swi_data.as_ref().and_then(|swi| swi.windows.first()) else {
      return Ok(indices);
    };

    let start: usize = usize::try_from(window.offset)
      .map_err(|_| XrfError::new_invalid_error("A sliding window's offset is past the platform's range"))?;
    let end: usize = start + usize::from(window.num_tris) * 3;

    indices.get(start..end).ok_or_else(|| {
      XrfError::new_invalid_error(format!(
        "The full-detail sliding window reaches index {end}, past the child's {} indices",
        indices.len()
      ))
    })
  }
}
