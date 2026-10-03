use crate::contract::render_antialiasing::RenderAntialiasing;
use crate::frame::smaa_targets::SmaaTargets;
use crate::frame::smoothing_target::SmoothingTarget;

/// A viewport's smoothing pass while one smooths its scene as drawn: FXAA, or SMAA with its edges and weights.
pub struct LevelSmoothing {
  /// The targets' epoch it reads.
  pub epoch: u64,
  pub mode: RenderAntialiasing,
  pub target: SmoothingTarget,
  pub smaa: Option<SmaaTargets>,
  /// FXAA's one bind group, or SMAA's three.
  pub groups: Vec<wgpu::BindGroup>,
}
