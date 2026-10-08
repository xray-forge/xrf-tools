use xrf_renderer_core::{GraphTexture, UniformBinding};

use crate::frame::view_target_handles::ViewTargetHandles;
use crate::pass::lighting_uniform::LightingUniform;
use crate::pass::present_uniform::PresentUniform;

/// What a view's frame leaves for its window's composition: its targets as the frame's graph imported them, the frame
/// it shows (the upscaled one, or the scene), and its present and lighting uniforms as the frame pushed them.
#[derive(Clone, Copy)]
pub struct SceneOutput {
  pub targets: ViewTargetHandles,
  pub shown: GraphTexture,
  pub present: UniformBinding<PresentUniform>,
  pub lighting: UniformBinding<LightingUniform>,
  /// The indirect light a debug view shows, a texel of none where it is not gathered.
  pub indirect_light: GraphTexture,
  /// The reflections a debug view shows, a texel marked untraced where they are not traced.
  pub reflections: GraphTexture,
}
