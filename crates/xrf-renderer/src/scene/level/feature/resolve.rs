use crate::frame::fsr_targets::FsrTargets;
use crate::pass::fsr_uniform::FsrUniform;
use crate::scene::level::feature::dlss_resolve::DlssResolve;
use crate::scene::level::feature::ray_reconstruction_resolve::RayReconstructionResolve;
use crate::scene::level::feature::temporal_resolve::TemporalResolve;

/// The temporal resolve gathering a view's samples, and what it keeps from frame to frame.
pub enum Resolve {
  /// TAA.
  Temporal(Box<TemporalResolve>),
  /// FSR 2: its targets and its constants this frame.
  Fsr {
    targets: Box<FsrTargets>,
    uniform: FsrUniform,
  },
  /// DLSS.
  Dlss(Box<DlssResolve>),
  /// DLSS ray reconstruction: denoised and upscaled in one network over the frame's guides.
  RayReconstruction(Box<RayReconstructionResolve>),
}

impl Resolve {
  pub fn get_memory(&self) -> u64 {
    match self {
      Self::Temporal(temporal) => [&temporal.history, &temporal.flicker]
        .iter()
        .flat_map(|history| history.get_sides())
        .map(|side| side.get_memory())
        .sum(),
      Self::Fsr { targets, .. } => targets.get_memory(),
      Self::Dlss(resolve) => resolve.output.get_memory(),
      Self::RayReconstruction(resolve) => resolve.output.get_memory(),
    }
  }

  /// The written history becomes the one the next frame reads.
  pub fn swap(&mut self) {
    match self {
      Self::Temporal(temporal) => {
        temporal.history.swap();
        temporal.flicker.swap();
      }
      Self::Fsr { targets, .. } => targets.frames.swap(),
      Self::Dlss(resolve) => resolve.is_reset = false,
      Self::RayReconstruction(resolve) => resolve.is_reset = false,
    }
  }
}
