use std::collections::HashMap;

use crate::object::decompile::geometry::source::ogf_corner::OgfCorner;
use crate::object::decompile::geometry::welded_face::WeldedFace;

/// Drops the faces an export wrote again: a face of the same surface on the same points, wound the same way, with the
/// same corners, draws nothing the first did not.
///
/// `CExportObjectOGF::PrepareMESH` counts a two-sided surface's faces twice, once for each copy it writes, but counts
/// them down once per face, so it walks the surface's faces a second time and writes every face and its mirror again,
/// onto the vertices the first walk packed: SoC's `plitka_br1` writes each of its 1,320 two-sided faces four times.
/// A mirror is wound the other way, so it is never taken for a repeat; [`super::two_sided_pairs::TwoSidedPairs`] folds
/// it afterwards.
pub(crate) struct RepeatedFaces;

impl RepeatedFaces {
  pub(crate) fn drop_repeats(faces: Vec<WeldedFace>) -> Vec<WeldedFace> {
    let mut kept_by_key: HashMap<(usize, [u32; 3]), Vec<[OgfCorner; 3]>> = HashMap::new();

    faces
      .into_iter()
      .filter(|face| {
        let (points, corners) = Self::get_rotated(face);
        let kept: &mut Vec<[OgfCorner; 3]> = kept_by_key.entry((face.surface, points)).or_default();

        if kept.contains(&corners) {
          false
        } else {
          kept.push(corners);
          true
        }
      })
      .collect()
  }

  /// The face's points and corners turned to start at its lowest point, keeping their winding, so every start of one
  /// winding gives one key.
  fn get_rotated(face: &WeldedFace) -> ([u32; 3], [OgfCorner; 3]) {
    let start: usize = (0..3).min_by_key(|index| face.points[*index]).unwrap_or(0);

    (
      std::array::from_fn(|index| face.points[(start + index) % 3]),
      std::array::from_fn(|index| face.corners[(start + index) % 3]),
    )
  }
}
