use glam::Vec3;

/// One frame a lens flare is drawn in: towards the sun in view space and in the world, the sun's colour, how far the
/// flare has faded in, and the seconds since the last frame its visibility eases over.
#[derive(Clone, Copy, Debug)]
pub struct FlareFrame {
  pub to_sun: Vec3,
  pub toward_sun: Vec3,
  pub color: Vec3,
  pub faded: f32,
  pub delta: f32,
}
