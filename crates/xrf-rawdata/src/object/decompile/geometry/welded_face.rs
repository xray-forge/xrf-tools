use crate::object::decompile::geometry::ogf_corner::OgfCorner;

/// One triangle with its corners welded into the mesh's points: which points, the corners as the OGF stored them, the
/// surface it was exported in, and whether it stands for a two-sided face.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct WeldedFace {
  pub(crate) points: [u32; 3],
  pub(crate) corners: [OgfCorner; 3],
  /// Index of the OGF surface the triangle came from.
  pub(crate) surface: usize,
  /// Whether the export wrote the face twice, its copy mirrored, as it does every face of a two-sided surface.
  pub(crate) is_two_sided: bool,
}

impl WeldedFace {
  /// The corner standing on `point`, or `None` where the face does not touch it.
  pub(crate) fn get_corner_at(&self, point: u32) -> Option<&OgfCorner> {
    self
      .points
      .iter()
      .position(|it| *it == point)
      .map(|index| &self.corners[index])
  }

  /// Whether `other` is this face mirrored: the same points wound the other way, each corner's normal turned
  /// around, as `PrepareMESH` writes a two-sided face's copy.
  pub(crate) fn is_mirrored_by(&self, other: &Self) -> bool {
    let [a, b, c] = self.points;
    let is_reversed: bool = [[c, b, a], [b, a, c], [a, c, b]].contains(&other.points);

    is_reversed
      && self.points.iter().zip(&self.corners).all(|(point, corner)| {
        other
          .get_corner_at(*point)
          .is_some_and(|mirrored| corner.has_normal_opposite(mirrored))
      })
  }
}
