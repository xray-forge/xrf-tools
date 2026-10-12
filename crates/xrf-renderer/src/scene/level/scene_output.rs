use xrf_renderer_core::{GraphTexture, UniformBinding};

use crate::frame::view_target_handles::ViewTargetHandles;
use crate::pass::lighting_uniform::LightingUniform;
use crate::pass::present_uniform::PresentUniform;

/// What a view's frame leaves for its window's composition: its targets as the frame's graph imported them, its
/// finished frame and the image it shows, and its present and lighting uniforms as the frame pushed them.
#[derive(Clone, Copy)]
pub struct SceneOutput {
  pub targets: ViewTargetHandles,
  /// The finished frame at the drawn size: the scene, or the target the smoothing or the temporal resolve wrote.
  pub scene: GraphTexture,
  /// The image the window shows: the upscaled or sharpened one, or the finished frame.
  pub shown: GraphTexture,
  pub present: UniformBinding<PresentUniform>,
  pub lighting: UniformBinding<LightingUniform>,
  /// The indirect light a debug view shows, a texel of none where it is not gathered.
  pub indirect_light: GraphTexture,
  /// The reflections a debug view shows, a texel marked untraced where they are not traced.
  pub reflections: GraphTexture,
  /// How the reflections' rays ended, which a debug view shows; a texel of no ray where they are not traced.
  pub reflection_ends: GraphTexture,
  /// The bloom the present lays over the frame: the enhanced bloom's where it draws, the engine's target otherwise.
  pub bloom: GraphTexture,
}
