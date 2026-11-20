use glam::Mat4;
use xrf_dlss::DlssRayReconstructor;

use crate::feature::image::resolve::dlss::ngx_evaluation::NgxEvaluation;
use crate::frame::targets::frame_texture::FrameTexture;

/// A view's DLSS ray reconstructor and what it writes.
pub(crate) struct RayReconstructionResolve {
  pub reconstructor: DlssRayReconstructor,
  /// The frame it writes, at the output's size.
  pub output: FrameTexture,
  /// Whether the next frame forgets the ones before.
  pub is_reset: bool,
  /// This frame's unjittered view and projection, which its hit distances are placed through.
  pub view: Mat4,
  pub projection: Mat4,
  pub evaluation: NgxEvaluation,
}
