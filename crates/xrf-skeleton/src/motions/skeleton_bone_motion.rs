use xrf_math::Vector3d;

use crate::motions::Quaternion;

/// One bone's animation within one motion, dequantised into renderer-ready values.
#[derive(Clone, Debug, PartialEq)]
pub struct SkeletonBoneMotion {
  pub rotations: Vec<Quaternion>,
  pub translations: Vec<Vector3d>,
}

impl SkeletonBoneMotion {
  /// The rotation at a frame, clamped to what the stream holds.
  pub fn get_rotation(&self, frame: usize) -> Quaternion {
    self.rotations[frame.min(self.rotations.len().saturating_sub(1))]
  }

  /// The translation at a frame, clamped to what the stream holds.
  pub fn get_translation(&self, frame: usize) -> &Vector3d {
    &self.translations[frame.min(self.translations.len().saturating_sub(1))]
  }
}
