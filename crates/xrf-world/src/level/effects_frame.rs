use std::collections::HashMap;

use glam::{Mat4, Vec3};
use xrf_renderer::AmbientGust;

/// What a frame of a level's effects gives its scene besides its posts: the wind the ambient effects blow, how much of
/// each campfire's idle light shows by its id, and where each object motion has its object in engine space and how fast
/// it moves, by the motion's name.
#[derive(Default)]
pub struct EffectsFrame {
  pub gust: AmbientGust,
  pub campfire_shares: HashMap<u16, f32>,
  pub motions: HashMap<String, (Mat4, Vec3)>,
}
