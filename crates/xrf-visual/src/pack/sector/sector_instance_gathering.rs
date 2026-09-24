use xrf_math::Matrix4x4;

/// The places one mesh stands, gathered while a sector is walked.
#[derive(Debug, Default)]
pub(crate) struct SectorInstanceGathering {
  pub(crate) drawables: Vec<u32>,
  pub(crate) placements: Vec<Matrix4x4>,
  /// Each place's hemisphere scale and bias, beside its transform.
  pub(crate) hemi: Vec<[f32; 2]>,
  /// Each place's impostor, or -1 for a tree no `MT_LOD` visual composes.
  pub(crate) impostors: Vec<i32>,
}
