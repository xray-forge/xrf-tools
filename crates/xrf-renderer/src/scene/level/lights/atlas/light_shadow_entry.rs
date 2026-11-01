use crate::feature::scene::drawable::drawable_kinds::DrawableKinds;
use crate::scene::level::lights::atlas::light_shadow_set::LightShadowSet;

/// What one light's shadow holds of the atlas: the faces it lights with, and those it is drawing at another size, which
/// take their place once all are drawn.
#[derive(Clone, Debug, Default)]
pub struct LightShadowEntry {
  pub shown: Option<LightShadowSet>,
  pub next: Option<LightShadowSet>,
  /// The frame the light was last in view, which room is made from the oldest of.
  pub seen: u64,
  /// The frame the light was last kept ready out of view, standing near the camera.
  pub warmed: u64,
  /// What the scene held when the swaying places in its range were last looked for, none before they were.
  pub swaying_contents: Option<usize>,
  /// The most any swaying place in its range reaches for each metre it stands from the light: zero where none does.
  pub swaying: f32,
  /// The drawable kinds whose casters its light casts this frame (their casters stand within its reach and it stands
  /// near the eye), and the most any of them leans, in metres.
  pub casting: DrawableKinds,
  pub casting_lean: f32,
}
