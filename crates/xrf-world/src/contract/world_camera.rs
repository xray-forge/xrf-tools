use serde::{Deserialize, Serialize};

/// A camera a viewport is driven by, as its consumer describes it.
///
/// Described again from the same start, a camera keeps where it has been moved and takes only the rest.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum WorldCamera {
  /// Flies free, turned by a drag and moved by the keys, as a level is walked.
  Fly {
    /// Where the camera starts, and returns to on reset.
    position: [f32; 3],
    /// What it looks at from there.
    target: [f32; 3],
    /// Vertical field of view, in degrees.
    field_of_view: f32,
    near: f32,
    far: f32,
    /// Metres a second at a walk.
    speed: f32,
    /// Times the speed while the boost key is held.
    boost: f32,
    /// Radians of turn per CSS pixel dragged.
    sensitivity: f32,
  },
  /// Orbits a target, as a model or texture preview does.
  Orbit {
    /// Where the camera starts, and returns to on reset.
    position: [f32; 3],
    /// What the camera looks at and turns around.
    target: [f32; 3],
    /// Vertical field of view, in degrees.
    field_of_view: f32,
    near: f32,
    far: f32,
  },
}

impl WorldCamera {
  /// Where the camera starts and what it looks at.
  pub fn get_start(&self) -> ([f32; 3], [f32; 3]) {
    match *self {
      WorldCamera::Fly { position, target, .. } | WorldCamera::Orbit { position, target, .. } => (position, target),
    }
  }

  /// Vertical field of view in degrees, near and far.
  pub fn get_lens(&self) -> (f32, f32, f32) {
    match *self {
      WorldCamera::Fly {
        field_of_view,
        near,
        far,
        ..
      }
      | WorldCamera::Orbit {
        field_of_view,
        near,
        far,
        ..
      } => (field_of_view, near, far),
    }
  }
}
