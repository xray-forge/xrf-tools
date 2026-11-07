use std::collections::HashMap;

use xrf_error::{XrfError, XrfResult};
use xrf_math::Vector3d;

use crate::object::decompile::geometry::bone_influences::BoneInfluences;
use crate::object::decompile::geometry::ogf_surface::OgfSurface;
use crate::object::decompile::geometry::welded_face::WeldedFace;

/// The exported triangles of every surface welded back into one mesh: a point for each distinct position and set of
/// bones with their weights, in the order the triangles first reach them, and the triangles indexing them.
///
/// Weights tell points apart as well as bones do: a point's weights are its own, and the export's `VPack` moves a
/// vertex onto a neighbour within a millimetre, so two source points can come back at one position, following the same
/// bones with different weights. Welded, one of them would take the other's weights.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct WeldedMesh {
  pub(crate) points: Vec<Vector3d>,
  /// The bones each point follows and their weights, by index into the OGF's bones, from the first corner reaching it.
  /// Every corner welded into a point carries alike weights.
  pub(crate) point_influences: Vec<BoneInfluences>,
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
    let mut point_influences: Vec<BoneInfluences> = Vec::new();
    // The points at each position following each set of bones, told apart by their weights.
    let mut points_of_key: HashMap<[u32; 3 + BoneInfluences::CAPACITY], Vec<u32>> = HashMap::new();
    let mut faces: Vec<WeldedFace> = Vec::new();

    for (surface_index, surface) in surfaces.iter().enumerate() {
      for corners in &surface.triangles {
        let mut face_points: [u32; 3] = [0; 3];

        for (point, corner) in face_points.iter_mut().zip(corners) {
          let candidates: &mut Vec<u32> = points_of_key.entry(corner.get_weld_key()).or_default();
          let existing: Option<u32> = candidates
            .iter()
            .copied()
            .find(|candidate| point_influences[*candidate as usize].is_alike(&corner.influences));

          *point = match existing {
            Some(existing) => existing,
            None => {
              let added: u32 = u32::try_from(points.len())
                .map_err(|_| XrfError::new_invalid_error("A decompiled mesh has more points than a u32 indexes"))?;

              points.push(corner.position);
              point_influences.push(corner.influences);
              candidates.push(added);

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
      point_influences,
      faces,
    })
  }
}
