use glam::Vec2;

/// The rings the enhanced drops leave this frame, which the water and the puddles ripple by: the tile they wrap over
/// (`RainUniform::get_rings`), none without them, the metres they reach, their strength, and where the drops' square
/// stands from the eye in renderer `x` and `z`, within which a ring is a landing seen.
#[derive(Clone, Copy, Debug)]
pub(crate) struct DropRings {
  pub tile: f32,
  pub reach: f32,
  pub strength: f32,
  pub aim: Vec2,
}
