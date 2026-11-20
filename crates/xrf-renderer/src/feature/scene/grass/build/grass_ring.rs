use crate::camera::camera_view::CameraView;

/// How a view's grass ring is planted: seen through a lens, its farthest bands lost where the lists run out, or flat
/// under an orthographic camera, a level map's tile planting all of its ground alike.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GrassRing {
  Lens,
  Flat,
}

impl GrassRing {
  /// The ring `camera` plants.
  pub(crate) fn of(camera: &CameraView) -> Self {
    if camera.is_orthographic() {
      Self::Flat
    } else {
      Self::Lens
    }
  }
}
