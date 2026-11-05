use glam::Vec3;

/// A strike's thunder: the bolt's section, where it struck from in engine space, and how far from the view.
#[derive(Clone, Debug, PartialEq)]
pub struct StrikeThunder {
  pub bolt: String,
  pub position: Vec3,
  pub distance: f32,
}
