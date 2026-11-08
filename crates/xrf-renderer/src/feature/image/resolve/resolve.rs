use crate::feature::image::resolve::dlss::dlss_resolve::DlssResolve;
use crate::feature::image::resolve::dlss::ray_reconstruction_resolve::RayReconstructionResolve;
use crate::feature::image::resolve::fsr::fsr_uniform::FsrUniform;
use crate::feature::image::resolve::fsr::targets::fsr_targets::FsrTargets;
use crate::feature::image::resolve::temporal::temporal_resolve::TemporalResolve;

/// The temporal resolve gathering a view's samples, and what it keeps from frame to frame.
pub enum Resolve {
  /// TAA.
  Temporal(Box<TemporalResolve>),
  /// FSR: its targets and its constants this frame.
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
  /// Drops what it gathered, so its next frame resolves the frame alone, as after a cut: TAA's last camera, which its
  /// history is read by, FSR's frames counted from none, and DLSS's reset.
  pub fn reset(&mut self) {
    match self {
      Self::Temporal(temporal) => temporal.previous = None,
      Self::Fsr { targets, .. } => targets.frames.restart(),
      Self::Dlss(resolve) => resolve.is_reset = true,
      Self::RayReconstruction(resolve) => resolve.is_reset = true,
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
