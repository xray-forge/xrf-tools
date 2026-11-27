use xrf_game_objects::ObjectRegistry;
use xrf_math::glam::Vec3;

/// What the game's state hands A-Life each step: the object registry, which it switches objects online and offline in,
/// the graph level the actor is on (the level played now, none while it is on none the graph holds), and where the
/// actor stands, in the engine's space.
pub struct AlifeContext<'a> {
  pub objects: &'a mut ObjectRegistry,
  pub level: Option<u8>,
  pub actor: Vec3,
}
