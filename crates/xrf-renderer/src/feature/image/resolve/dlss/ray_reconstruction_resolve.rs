use glam::Mat4;
use xrf_dlss::DlssRayReconstructor;

use crate::frame::frame_texture::FrameTexture;

/// DLSS ray reconstruction's state: its reconstructor, the frame it writes at the output's size, whether the next frame
/// forgets the ones before, and this frame's unjittered view and projection, which its hit distances are placed
/// through.
pub struct RayReconstructionResolve {
  pub reconstructor: DlssRayReconstructor,
  pub output: FrameTexture,
  pub is_reset: bool,
  pub view: Mat4,
  pub projection: Mat4,
}
