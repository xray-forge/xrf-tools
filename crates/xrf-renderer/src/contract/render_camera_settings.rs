use serde::{Deserialize, Serialize};

/// The physical camera the frame is shown through after its resolve: a shutter that blurs what moves, a lens that
/// blurs what stands out of focus, dims its corners and splits colour towards them, and film grain. Every effect is
/// off until turned on; the rest shape each one.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderCameraSettings {
  /// Whether what moves across the frame blurs along its motion while the shutter is open.
  pub is_motion_blurred: bool,
  /// The share of a frame the shutter stays open: a half is film's 180° shutter.
  pub shutter: f32,
  /// The longest blur, as a share of the frame's height, so a fast turn smears no further.
  pub motion_limit: f32,
  /// Whether what stands nearer or farther than the focus blurs by the lens's depth of field.
  pub is_defocused: bool,
  /// The lens's aperture as an f-number: lower opens it wider and keeps less in focus.
  pub f_stop: f32,
  /// Metres from the camera the lens is focused at, which a pick in the viewer sets.
  pub focus_distance: f32,
  /// The sensor's height in millimetres, which with the view's field of view gives the lens's focal length: 24 is
  /// full-frame film's.
  pub sensor_height: f32,
  /// Whether the lens dims the frame's corners, as light reaching the film at an angle falls off.
  pub is_vignetted: bool,
  /// How much of the lens's falloff the corners show, from none to all of it.
  pub vignette_strength: f32,
  /// Whether the frame is shown with film grain, varying every frame.
  pub is_grained: bool,
  /// How strongly the grain shows, by the frame's brightness.
  pub grain_strength: f32,
  /// How many pixels across a grain is.
  pub grain_size: f32,
  /// Whether the lens splits colour towards the frame's edges, its glass bending red and blue apart.
  pub is_aberrated: bool,
  /// How far apart the colours split at the corners.
  pub aberration_strength: f32,
}

impl Default for RenderCameraSettings {
  fn default() -> Self {
    Self {
      is_motion_blurred: false,
      shutter: 0.5,
      motion_limit: 0.04,
      is_defocused: false,
      f_stop: 2.8,
      focus_distance: 10.0,
      sensor_height: 24.0,
      is_vignetted: false,
      vignette_strength: 0.5,
      is_grained: false,
      grain_strength: 0.3,
      grain_size: 1.6,
      is_aberrated: false,
      aberration_strength: 0.5,
    }
  }
}

impl RenderCameraSettings {
  /// Whether a pass after the resolve blurs the frame: by motion, or out of focus.
  pub fn is_blurred(&self) -> bool {
    self.is_motion_blurred || self.is_defocused
  }
}
