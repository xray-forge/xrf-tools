use std::collections::HashMap;

use xrf_error::{XrfError, XrfResult};
use xrf_math::Vector3d;

use crate::object::decompile::geometry::ogf_surface::OgfSurface;
use crate::object::decompile::geometry::welded_face::WeldedFace;

/// The exported triangles of every surface welded back into one mesh: a point for each distinct position and bone,
/// in the order the triangles first reach them, and the triangles indexing them.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct WeldedMesh {
  pub(crate) points: Vec<Vector3d>,
  /// The bone each point belongs to, by index into the OGF's bones.
  pub(crate) point_bones: Vec<u16>,
  pub(crate) faces: Vec<WeldedFace>,
}

impl WeldedMesh {
  /// Welds every surface's triangles into one mesh.
  ///
  /// # Errors
  ///
  /// Returns an error when the points outnumber a `u32`, which an `.object` indexes them by.
  pub(crate) fn weld(surfaces: &[OgfSurface]) -> XrfResult<Self> {
    let mut points: Vec<Vector3d> = Vec::new();
    let mut point_bones: Vec<u16> = Vec::new();
    let mut point_of_key: HashMap<[u32; 4], u32> = HashMap::new();
    let mut faces: Vec<WeldedFace> = Vec::new();

    for (surface_index, surface) in surfaces.iter().enumerate() {
      for corners in &surface.triangles {
        let mut face_points: [u32; 3] = [0; 3];

        for (point, corner) in face_points.iter_mut().zip(corners) {
          *point = match point_of_key.get(&corner.get_weld_key()) {
            Some(existing) => *existing,
            None => {
              let added: u32 = u32::try_from(points.len())
                .map_err(|_| XrfError::new_invalid_error("A decompiled mesh has more points than a u32 indexes"))?;

              points.push(corner.position);
              point_bones.push(corner.bone);
              point_of_key.insert(corner.get_weld_key(), added);

              added
            }
          };
        }

        faces.push(WeldedFace {
          points: face_points,
          corners: *corners,
          surface: surface_index,
          is_two_sided: false,
        });
      }
    }

    Ok(Self {
      points,
      point_bones,
      faces,
    })
  }
}
