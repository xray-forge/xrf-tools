/// The sun's sprite a lens flare draws in the sky, `CLensFlareDescriptor::m_Source`.
#[derive(Clone, Debug, PartialEq)]
pub struct RenderSunSprite {
  /// By reference, as a keyframe's textures are.
  pub texture: String,
  /// Half its side, as a share of the distance it is drawn at.
  pub radius: f32,
  /// `sun_ignore_color`: drawn white rather than in the sun's colour.
  pub is_colorless: bool,
}
