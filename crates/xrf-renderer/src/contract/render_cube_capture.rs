use serde::{Deserialize, Serialize};

/// Pixels a cubemap's face spans on a side at most.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub const RENDER_CUBE_MOST_PIXELS: u32 = 2048;

/// Times over a cubemap's face is drawn at most.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub const RENDER_CUBE_MOST_SUPERSAMPLING: u32 = 4;

/// A cubemap asked of a viewport: the six faces around where its camera stands, world-aligned in the engine's order
/// `+x -x +y -y +z -z`, each `size` pixels square at exactly 90 degrees.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderCubeCapture {
  /// A face's pixels on a side.
  pub size: u32,
  /// Times over on each side a face is drawn before it is reduced to its pixels.
  pub supersampling: u32,
  /// Frames each face is drawn from where it stands before the one kept, for what builds over frames to settle.
  pub settle_frames: u32,
}
