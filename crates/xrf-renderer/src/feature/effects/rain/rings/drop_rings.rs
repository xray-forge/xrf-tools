/// The rings the enhanced drops leave this frame, which the water and the puddles ripple by: the tile they wrap over
/// (`RainUniform::get_rings`), none without them, the metres they reach and their strength.
#[derive(Clone, Copy, Debug)]
pub(crate) struct DropRings {
  pub tile: f32,
  pub reach: f32,
  pub strength: f32,
}
