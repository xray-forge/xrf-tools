use xrf_game_objects::ObjectRegistry;
use xrf_math::glam::Vec3;

/// What the game's state hands A-Life each step: the object registry, which it switches objects online and offline in,
/// and where the actor stands, in the engine's space.
pub struct AlifeContext<'a> {
  pub objects: &'a mut ObjectRegistry,
  pub actor: Vec3,
}
