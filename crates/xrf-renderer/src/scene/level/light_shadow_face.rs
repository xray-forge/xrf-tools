use crate::camera::camera_view::CameraView;
use crate::scene::level::shadow_tile::ShadowTile;

/// One face of a light's shadow: its square of the atlas, the camera it is drawn through, and what the scene held
/// when it was last drawn, none before it is.
#[derive(Clone, Copy, Debug)]
pub struct LightShadowFace {
  pub tile: ShadowTile,
  pub view: CameraView,
  pub drawn: Option<usize>,
  /// The sway's time it was last drawn at.
  pub drawn_at: f32,
}
