use xrf_math::{Vector2d, Vector3d};

/// One corner of an exported triangle as the OGF stores it: where it stands, the normal the export gave it, its
/// texture coordinate and the one bone a rigid model ties it to.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct OgfCorner {
  pub(crate) position: Vector3d,
  pub(crate) normal: Vector3d,
  pub(crate) texture_coordinate: Vector2d,
  pub(crate) bone: u16,
}

impl OgfCorner {
  /// How far two exported normals may differ, component by component, and still be one smoothed normal: the export
  /// sums a fan's face normals once per corner, starting from a different face each time, so equal normals can differ
  /// in their last bits. A hard edge differs by far more.
  pub(crate) const NORMAL_TOLERANCE: f32 = 1e-3;

  /// Whether the two corners carry one smoothed normal.
  pub(crate) fn has_normal_of(&self, other: &Self) -> bool {
    Self::is_near(&self.normal, &other.normal)
  }

  /// Whether this corner's normal is the other's turned around, as a two-sided copy's are.
  pub(crate) fn has_normal_opposite(&self, other: &Self) -> bool {
    Self::is_near(&self.normal, &-&other.normal)
  }

  /// The position's bits and the bone, which together decide a welded point: the export copied one float into every
  /// corner split from a point, and a point belongs to one bone.
  pub(crate) fn get_weld_key(&self) -> [u32; 4] {
    [
      self.position.x.to_bits(),
      self.position.y.to_bits(),
      self.position.z.to_bits(),
      u32::from(self.bone),
    ]
  }

  fn is_near(first: &Vector3d, second: &Vector3d) -> bool {
    (first.x - second.x).abs() <= Self::NORMAL_TOLERANCE
      && (first.y - second.y).abs() <= Self::NORMAL_TOLERANCE
      && (first.z - second.z).abs() <= Self::NORMAL_TOLERANCE
  }
}
