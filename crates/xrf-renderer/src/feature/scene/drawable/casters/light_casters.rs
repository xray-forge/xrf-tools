use crate::feature::scene::drawable::drawable_kind::DrawableKind;

/// A drawable kind's shadow casters as the local lights' faces draw them: over a face whose light stands near the eye
/// and reaches them, drawn again as the trees' sway has a face drawn again.
#[derive(Clone, Copy, Debug)]
pub struct LightCasters {
  pub kind: DrawableKind,
  /// Metres from the eye they stand within.
  pub reach: f32,
  /// Metres from the eye a light stands within to cast them.
  pub light_distance: f32,
  /// Metres they lean at most for the wind's amplitude, which a face over them is drawn again for as a tree is.
  pub lean: f32,
}
