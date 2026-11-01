use glam::Vec3;

use crate::scene::level::lights::asks::light_in_view::LightInView;
use crate::scene::level::lights::asks::lights_frame::LightsFrame;

/// What a view asks of its scene's local lights this frame, picked by its camera (`LevelLights::pick`): the lights it
/// sees, nearest first, as many as its records hold, and the shadowed ones out of view kept ready near the eye, with
/// the frame it lights them and has their faces drawn by. The scene settles every view's asks for faces at once
/// (`LevelLights::settle`); the view then writes its lights from its own (`LevelLights::write`).
pub struct LightAsks {
  pub frame: LightsFrame,
  /// Where the camera looks, which a spot facing it asks bigger faces by.
  pub forward: Vec3,
  /// The lights it sees, nearest first, as many as its records hold.
  pub in_view: Vec<LightInView>,
  /// The shadowed lights out of view the eye stands near, nearest first, their faces kept ready for a turn.
  pub ready: Vec<LightInView>,
  /// The lights it sees past those its records hold.
  pub excess: u32,
}
