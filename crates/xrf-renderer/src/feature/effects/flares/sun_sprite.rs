use glam::Vec4;

/// The sun's sprite as the sky draws it this frame: its texture, and its colour faded in or out with its radius in
/// `w`, as the lighting's `sun_sprite` reads it.
#[derive(Clone, Debug)]
pub struct SunSprite {
  pub texture: String,
  pub look: Vec4,
}
