use xrf_math::{Vector2d, Vector3d};

use crate::object::decompile::geometry::source::bone_influences::BoneInfluences;
use crate::object::decompile::geometry::source::normal_precision::NormalPrecision;

/// One corner of a compiled triangle as an OGF, a level or a detail model stores it: where it stands, the normal the
/// export gave it and how precisely that is stored (zero where none is), its texture coordinate and the bones it
/// follows.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct OgfCorner {
  pub(crate) position: Vector3d,
  pub(crate) normal: Vector3d,
  pub(crate) normal_precision: NormalPrecision,
  pub(crate) texture_coordinate: Vector2d,
  pub(crate) influences: BoneInfluences,
}

impl OgfCorner {
  /// Whether the two corners carry one smoothed normal; where either stores none, nothing tells them apart, so they do.
  pub(crate) fn has_normal_of(&self, other: &Self) -> bool {
    self.is_near(&self.normal, &other.normal, other).unwrap_or(true)
  }

  /// Whether this corner's normal is the other's turned around, as a two-sided copy's are; where either stores none,
  /// nothing shows it is.
  pub(crate) fn has_normal_opposite(&self, other: &Self) -> bool {
    self.is_near(&self.normal, &-&other.normal, other).unwrap_or(false)
  }

  /// Whether the corner stores a normal, which a detail model's corners do not.
  pub(crate) fn has_normal(&self) -> bool {
    self.normal_precision != NormalPrecision::Unstored
  }

  /// The position's bits and the bones followed, which narrow a welded point to those standing there: the export copied
  /// one float into every corner split from a point. The weights stay out of the key, since a point exported into
  /// children of different link layouts comes back with weights rounded differently; [`BoneInfluences::is_alike`] tells
  /// the points of one key apart.
  pub(crate) fn get_weld_key(&self) -> [u32; 3 + BoneInfluences::CAPACITY] {
    let bones: [u32; BoneInfluences::CAPACITY] = self.influences.get_bone_key();

    [
      self.position.x.to_bits(),
      self.position.y.to_bits(),
      self.position.z.to_bits(),
      bones[0],
      bones[1],
      bones[2],
      bones[3],
    ]
  }

  /// Whether `first` and `second` are one normal within the coarser precision of this corner and `other`, or `None`
  /// where either stores no normal.
  fn is_near(&self, first: &Vector3d, second: &Vector3d, other: &Self) -> Option<bool> {
    let tolerance: f32 = self.normal_precision.get_tolerance_with(other.normal_precision)?;

    Some(
      (first.x - second.x).abs() <= tolerance
        && (first.y - second.y).abs() <= tolerance
        && (first.z - second.z).abs() <= tolerance,
    )
  }
}
