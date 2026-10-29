use std::collections::BTreeMap;

/// What one shader table entry's surfaces are made of: their area on each game material's collision faces, and the area
/// with none under it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LevelSurfaceMaterial {
  /// Square metres the entry's triangles cover.
  pub area: f64,
  /// Of it, square metres lying on no collision face: grass, foliage, a prop the level compiler gave no collision.
  pub uncovered_area: f64,
  /// Square metres lying on each game material's faces, by the material's `gamemtl.xr` id.
  pub materials: BTreeMap<u16, f64>,
}

impl LevelSurfaceMaterial {
  /// Adds one triangle of `area` lying on `material`, or on none.
  pub(crate) fn add(&mut self, area: f64, material: Option<u16>) {
    self.area += area;

    match material {
      Some(material) => *self.materials.entry(material).or_default() += area,
      None => self.uncovered_area += area,
    }
  }

  /// Folds another tally of the same entry into this one.
  pub(crate) fn merge(&mut self, other: Self) {
    self.area += other.area;
    self.uncovered_area += other.uncovered_area;

    for (material, area) in other.materials {
      *self.materials.entry(material).or_default() += area;
    }
  }
}
