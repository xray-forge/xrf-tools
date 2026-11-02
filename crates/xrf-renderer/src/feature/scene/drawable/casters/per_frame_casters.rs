use crate::contract::feature::lighting::render_shadow_settings::RENDER_MAX_SHADOW_CASCADES;
use crate::feature::scene::drawable::drawable_kind::DrawableKind;

/// A drawable kind's shadow casters moving every frame, as the sun's cascades draw them: over the kept layer of every
/// near cascade they land in on screen, each frame, while what stands still and sways waits its own redraws.
#[derive(Clone, Copy, Debug)]
pub struct PerFrameCasters {
  pub kind: DrawableKind,
  /// Metres from the eye they stand within, which the nearest cascades covering it are drawn with them.
  pub reach: f32,
  /// Metres they stand at most over the ground, which where they land is sampled past their reach for.
  pub standing: f32,
  /// The pass drawing them into each cascade, as the frame's timings name it.
  pub passes: &'static [&'static str; RENDER_MAX_SHADOW_CASCADES],
  /// Milliseconds a cascade's draw of them costs before the timer measured one.
  pub seed: f32,
}
