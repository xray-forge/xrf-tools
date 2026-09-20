use xrf_math::Matrix4x4;

/// The places one mesh stands, gathered while a sector is walked.
#[derive(Debug, Default)]
pub(crate) struct SectorInstanceGathering {
  pub(crate) drawables: Vec<u32>,
  pub(crate) placements: Vec<Matrix4x4>,
}
